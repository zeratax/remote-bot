use crate::commands::util::email;
use crate::configuration::Config;
use crate::constants;

use chrono::{Duration as ChronoDuration, Utc};
use serenity::all::{ResolvedOption, ResolvedValue};
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::prelude::CommandOptionType;
use std::path::Path;
use tokio::fs;

pub async fn run(options: &[ResolvedOption<'_>], config: &Config) -> String {
    if let (
        Some(ResolvedOption {
            value: ResolvedValue::Integer(amount),
            ..
        }),
        Some(ResolvedOption {
            value: ResolvedValue::String(unit),
            ..
        }),
    ) = (options.get(0), options.get(1))
    {
        let duration = match *unit {
            "min" | "minutes" => ChronoDuration::minutes(*amount),
            "hours" => ChronoDuration::hours(*amount),
            _ => return "Invalid time unit! Use 'min' or 'hours'.".to_string(),
        };

        let alarm_time = Utc::now().with_timezone(&config.timezone) + duration;

        let path = Path::new(constants::ALARM_PATH);
        if let Err(e) = write_to_file(path, alarm_time.format("%H:%M").to_string()).await {
            return format!("Error saving alarm: {}", e);
        }

        let subject = "Alarm Created";
        let body = "New alarm created!";
        let sender = "alarm";
        if let Err(e) = email::send_email(config, &subject, &body, &sender).await {
            return format!("Error sending email: {}", e);
        }

        return format!("⏰ Alarm set for localtime: {}", alarm_time.format("%H:%M"));
    }
    "Invalid input! Please provide a valid amount and unit.".to_string()
}

async fn write_to_file(path: &Path, content: String) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        if !fs::try_exists(dir).await.unwrap_or(false) {
            fs::create_dir_all(dir)
                .await
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
    }
    fs::write(path, content)
        .await
        .map_err(|e| format!("Failed to write alarm to file: {}", e))
}

// Register the command
pub fn register() -> CreateCommand {
    CreateCommand::new("alarm")
        .description("Set an alarm")
        .add_option(
            CreateCommandOption::new(CommandOptionType::Integer, "amount", "The amount of time")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "unit",
                "The time unit (min or hours)",
            )
            .required(true)
            .add_string_choice("minutes", "min")
            .add_string_choice("hours", "hours"),
        )
        .add_integration_type(serenity::all::InstallationContext::Guild)
        .add_integration_type(serenity::all::InstallationContext::User)
        .add_context(serenity::all::InteractionContext::Guild)
        .add_context(serenity::all::InteractionContext::BotDm)
        .add_context(serenity::all::InteractionContext::PrivateChannel)
}
