use serenity::builder::CreateCommand;
use serenity::model::application::ResolvedOption;

pub fn run(_options: &[ResolvedOption]) -> String {
    "Hey, I'm alive!".to_string()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("ping")
        .description("Ping me!")
        .add_integration_type(serenity::all::InstallationContext::Guild)
        .add_integration_type(serenity::all::InstallationContext::User)
        .add_context(serenity::all::InteractionContext::Guild)
        .add_context(serenity::all::InteractionContext::BotDm)
        .add_context(serenity::all::InteractionContext::PrivateChannel)
}
