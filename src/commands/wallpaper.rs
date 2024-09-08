use crate::commands::util::email;
use crate::configuration::Config;
use crate::constants;
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};
use std::path::Path;
use tokio::fs;

pub async fn run(options: &[ResolvedOption<'_>], config: &Config) -> String {
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
            let save_path = Path::new(constants::WALLPAPER_PATH);

            if let Err(e) = download_and_save_image(&attachment.url, &save_path).await {
                return format!("Failed to save image: {}", e);
            }

            let subject = "Wallpaper Updated";
            let body = "New Wallpaper accessible!";
            if let Err(e) = email::send_email(config, subject, &body).await {
                return format!("Error sending email: {}", e);
            }

            return "Wallpaper will soon be changed!".to_string();
        }
        return "Please upload an image file (not a video or gif)".to_string();
    }
    return "Please provide a valid attachment".to_string();
}

async fn download_and_save_image(url: &str, path: &Path) -> Result<(), String> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to download image: {}", e))?;

    let content = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read image content: {}", e))?;

    if let Some(dir) = path.parent() {
        if !fs::try_exists(dir).await.unwrap_or(false) {
            fs::create_dir_all(dir)
                .await
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
    }
    fs::write(path, content)
        .await
        .map_err(|e| format!("Failed to write image to file: {}", e))
}

pub fn register() -> CreateCommand {
    CreateCommand::new("wallpaper")
        .description("Set wallpaper for moji")
        .add_option(
            CreateCommandOption::new(CommandOptionType::Attachment, "attachment", "An image")
                .required(true),
        )
}
