use crate::commands::util::email;
use crate::configuration::Config;
use crate::constants;

use colored::*;
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{
    CommandInteraction, CommandOptionType, ResolvedOption, ResolvedValue,
};
use std::path::PathBuf;
use tokio::fs;
use url::Url;

pub async fn run(command: &CommandInteraction, config: &Config) -> String {
    let options = &command.data.options();
    if let Some(ResolvedOption {
        value: ResolvedValue::Attachment(attachment),
        ..
    }) = options.first()
    {
        if attachment
            .content_type
            .as_deref()
            .unwrap_or("")
            .starts_with("image/")
        {
            let saved_path = match download_and_save_image(&attachment.url).await {
                Ok(path) => match std::fs::canonicalize(&path) {
                    Ok(absolute_path) => absolute_path,
                    Err(error) => return format!("Failed to get absolute path: {}", error),
                },
                Err(error) => return format!("Failed to save image: {}", error),
            };

            let username = &command.user.name;
            let subject = "Wallpaper Updated";
            let body = format!("New wallpaper from {}!", username);
            let sender = "wallpaper";
            if let Err(e) = email::send_email(config, &subject, &body, &sender).await {
                return format!("Error sending email: {}", e);
            }

            println!(
                "{} set image to: {}",
                username.green(),
                saved_path.display().to_string().cyan()
            );

            return "🖼️ Wallpaper will soon be changed!".to_string();
        }
        return "Please upload an image file (not a video or gif)".to_string();
    }
    return "Please provide a valid attachment".to_string();
}

async fn download_and_save_image(url: &str) -> Result<PathBuf, String> {
    let parsed_url = Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;
    let filename = parsed_url
        .path_segments()
        .and_then(|segments| segments.last())
        .ok_or_else(|| "Failed to extract filename from URL".to_string())?;

    let save_dir: &PathBuf = &PathBuf::from(constants::WALLPAPER_DIR);
    let save_path = save_dir.join(filename);

    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to download image: {}", e))?;

    let content = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read image content: {}", e))?;

    if !fs::try_exists(save_dir).await.unwrap_or(false) {
        fs::create_dir_all(save_dir)
            .await
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    fs::write(&save_path, &content)
        .await
        .map_err(|e| format!("Failed to write image to file: {}", e))?;

    let wallpaper_path = &PathBuf::from(constants::WALLPAPER_PATH);
    fs::copy(&save_path, wallpaper_path).await.map_err(|e| {
        format!(
            "Failed to copy image to {}: {}",
            wallpaper_path.to_str().unwrap(),
            e
        )
    })?;

    Ok(save_path)
}

pub fn register() -> CreateCommand {
    CreateCommand::new("wallpaper")
        .description("Set wallpaper for moji")
        .add_option(
            CreateCommandOption::new(CommandOptionType::Attachment, "attachment", "An image")
                .required(true),
        )
        .add_integration_type(serenity::all::InstallationContext::Guild)
        .add_integration_type(serenity::all::InstallationContext::User)
        .add_context(serenity::all::InteractionContext::Guild)
        .add_context(serenity::all::InteractionContext::BotDm)
        .add_context(serenity::all::InteractionContext::PrivateChannel)
}
