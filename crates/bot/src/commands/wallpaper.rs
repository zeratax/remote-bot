use remote_bot_shared::state::AppState;
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{
    CommandInteraction, CommandOptionType, ResolvedOption, ResolvedValue,
};
use std::path::PathBuf;

use crate::configuration::Config;
use crate::constants;

use super::util::email::EmailConfig;
use super::util::image::handle_set_wallpaper_from_url;

pub async fn run(command: &CommandInteraction, config: &Config, app_state: &AppState) -> String {
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
            let wallpaper_dir = &PathBuf::from(constants::WALLPAPER_DIR);
            let set_by = command.user.name.clone();
            return match handle_set_wallpaper_from_url(
                &attachment.url,
                wallpaper_dir,
                &set_by,
                app_state,
                &EmailConfig::from(config.clone()),
            )
            .await
            {
                Ok(_) => "🖼️ Wallpaper will soon be changed!".to_string(),
                Err(err) => format!("Failed to process image: {}", err).to_string(),
            };
        }
        return "Please upload an image file (not a video or gif)".to_string();
    }
    return "Please provide a valid attachment".to_string();
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
