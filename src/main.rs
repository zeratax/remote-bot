mod commands;
mod configuration;
mod constants;

use crate::configuration::Config;

use config::Config as AppConfig;
use serenity::async_trait;
use serenity::builder::{
    CreateInteractionResponse, CreateInteractionResponseMessage, EditInteractionResponse,
};
use serenity::model::application::{Command, Interaction, InteractionResponseFlags};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tokio::task;
use warp::Filter;

pub struct Handler {
    pub config: Config,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
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
                        commands::wallpaper::run(&command.data.options(), &self.config).await;

                    if let Err(why) = command
                        .edit_response(&ctx.http, EditInteractionResponse::new().content(content))
                        .await
                    {
                        println!("Failed to edit response: {why}");
                    }

                    None
                }
                "alarm" => Some(commands::alarm::run(&command.data.options(), &self.config).await),
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
        let invite_link = format!("https://discord.com/oauth2/authorize?client_id={}", bot_id);

        println!("Invite me with this link: {}", invite_link);

        let _ = Command::set_global_commands(
            &ctx.http,
            vec![
                commands::ping::register(),
                commands::alarm::register(),
                commands::wallpaper::register(),
            ],
        )
        .await;
    }
}

#[tokio::main]
async fn main() {
    let settings = AppConfig::builder()
        // Add in `./Settings.toml`
        .add_source(config::File::with_name("settings"))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `REMOTE_BOT_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("REMOTE_BOT"))
        .build()
        .expect("Expected a settings file!")
        .try_deserialize::<Config>()
        .expect("Failed to deserialize settings");

    let web_server_task = task::spawn(async {
        let wallpaper = warp::path("wallpaper").and(warp::fs::file(constants::WALLPAPER_PATH));
        let alarm = warp::path("alarm").and(warp::fs::file(constants::ALARM_PATH));

        let routes = wallpaper.or(alarm);

        warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
    });

    let token = &settings.discord_token;

    let handler = Handler {
        config: settings.clone(),
    };

    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(handler)
        .await
        .expect("Error creating client");

    tokio::select! {
        _ = web_server_task => {
            println!("Web server has stopped.");
        },
        // Start a single shard, and start listening to events.
        //
        // Shards will automatically attempt to reconnect, and will perform exponential backoff until
        // it reconnects.
        result = client.start() => {
            if let Err(why) = result {
                println!("Discord client error: {why:?}");
            }
        },
    }
}
