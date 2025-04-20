pub mod commands;
pub mod configuration;
pub mod constants;

use crate::configuration::Config;
use colored::*;
use remote_bot_shared::state::AppState;
use serenity::{
    async_trait,
    builder::*,
    model::{application::*, gateway::Ready},
    prelude::*,
};
use std::sync::Arc;

pub struct Handler {
    pub config: Config,
    pub app_state: Arc<AppState>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let username = &command.user.name;
            println!(
                "{} used the {} command",
                username.green(),
                command.data.name.as_str().cyan(),
            );

            let content = match command.data.name.as_str() {
                "ping" => Some(commands::ping::run(&command.data.options())),
                "wallpaper" => {
                    if let Err(why) = command
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Defer(
                                CreateInteractionResponseMessage::new()
                                    .flags(InteractionResponseFlags::EPHEMERAL),
                            ),
                        )
                        .await
                    {
                        println!("Failed to defer response: {why}");
                        return;
                    }

                    let content =
                        commands::wallpaper::run(&command, &self.config, &self.app_state).await;

                    if let Err(why) = command
                        .edit_response(&ctx.http, EditInteractionResponse::new().content(content))
                        .await
                    {
                        println!("Failed to edit response: {why}");
                    }

                    None
                }
                "Set as Wallpaper" => {
                    if let Err(why) = command
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Defer(
                                CreateInteractionResponseMessage::new()
                                    .flags(InteractionResponseFlags::EPHEMERAL),
                            ),
                        )
                        .await
                    {
                        println!("Failed to defer response: {why}");
                        return;
                    }

                    let content =
                        commands::message_context::run(&command, &self.config, &self.app_state)
                            .await;

                    if let Err(why) = command
                        .edit_response(&ctx.http, EditInteractionResponse::new().content(content))
                        .await
                    {
                        println!("Failed to edit response: {why}");
                    }

                    None
                }
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new()
                    .content(content)
                    .flags(InteractionResponseFlags::EPHEMERAL);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let bot_id = ready.user.id;
        println!(
            "Invite me with this link: https://discord.com/oauth2/authorize?client_id={bot_id}"
        );

        let _ = Command::set_global_commands(
            &ctx.http,
            vec![
                commands::ping::register(),
                commands::alarm::register(),
                commands::wallpaper::register(),
                commands::message_context::register(),
            ],
        )
        .await;
    }
}

pub async fn run_bot(app_state: Arc<AppState>, config: Config) {
    let handler = Handler {
        config: config.clone(),
        app_state,
    };

    let mut client = Client::builder(&config.discord_token, GatewayIntents::empty())
        .event_handler(handler)
        .await
        .expect("Error creating client");

    if let Err(e) = client.start().await {
        eprintln!("Discord client error: {:?}", e);
    }
}
