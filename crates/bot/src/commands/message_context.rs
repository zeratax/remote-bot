use std::path::PathBuf;

use crate::commands::util::image::handle_set_wallpaper_from_url;
use crate::configuration::Config;
use crate::constants::{self};

use remote_bot_shared::state::AppState;
use serenity::builder::CreateCommand;
use serenity::model::application::CommandInteraction;

pub async fn run(command: &CommandInteraction, config: &Config, app_state: &AppState) -> String {
    fn get_image_url(command: &CommandInteraction) -> Option<&str> {
        let resolved = &command.data.resolved;
        let messages = &resolved.messages;
        let (_, message) = messages.iter().next()?;

        for attachment in &message.attachments {
            if let Some(content_type) = &attachment.content_type {
                if content_type.starts_with("image/") {
                    return Some(&attachment.url);
                }
            }
        }

        for embed in &message.embeds {
            if let Some(image) = &embed.image {
                return Some(&image.url);
            } else if let Some(thumbnail) = &embed.thumbnail {
                return Some(&thumbnail.url);
            }
        }

        None
    }

    let wallpaper_dir = &PathBuf::from(constants::WALLPAPER_DIR);
    if let Some(url) = get_image_url(command) {
        let set_by = command.user.name.clone();
        match handle_set_wallpaper_from_url(url, wallpaper_dir, &set_by, app_state, config).await {
            Ok(_) => "🖼️ Wallpaper will soon be changed!".to_string(),
            Err(err) => format!("Failed to process image: {}", err),
        }
    } else {
        "No images found in this message.".to_string()
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("Set as Wallpaper")
        .kind(serenity::model::application::CommandType::Message)
        .add_integration_type(serenity::all::InstallationContext::Guild)
        .add_integration_type(serenity::all::InstallationContext::User)
        .add_context(serenity::all::InteractionContext::Guild)
        .add_context(serenity::all::InteractionContext::BotDm)
        .add_context(serenity::all::InteractionContext::PrivateChannel)
}
