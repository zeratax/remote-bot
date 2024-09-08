use crate::configuration::Config;
use chrono::{Duration as ChronoDuration, Utc};
use serenity::all::{ResolvedOption, ResolvedValue};
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::prelude::CommandOptionType;

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

        return format!("⏰ Alarm set for localtime: {}", alarm_time.format("%H:%M"));
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
}
