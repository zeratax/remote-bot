use chrono::{Duration, Utc};
use serenity::all::{ResolvedOption, ResolvedValue};
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::CommandInteraction;
use serenity::model::prelude::CommandOptionType;

use crate::commands::util::email::{EmailConfig, send_email};
use crate::configuration::Config;

pub async fn run(command: &CommandInteraction, config: &Config) -> String {
    let options = &command.data.options();
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
            "min" | "minutes" => Duration::minutes(*amount),
            "hours" => Duration::hours(*amount),
            _ => return "Invalid time unit! Use 'min' or 'hours'.".to_string(),
        };

        let alarm_time = Utc::now().with_timezone(&config.timezone) + duration;
        let alarm_time_formatted = alarm_time.format("%H:%M");

        let username = &command.user.name;
        let subject = "Alarm Created";
        let body = format!("Alarm set by {}: {}", username, alarm_time_formatted);
        let sender = "alarm";
        if let Err(e) =
            send_email(&EmailConfig::from(config.clone()), &subject, &body, &sender).await
        {
            return format!("Error sending email: {}", e);
        }

        return format!("⏰ Alarm set for localtime: {}", alarm_time_formatted);
    }
    "Invalid input! Please provide a valid amount and unit.".to_string()
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
