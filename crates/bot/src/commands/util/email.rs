use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::configuration::Config;

pub struct EmailConfig {
    smtp_username: String,
    smtp_password: String,
    smtp_server: String,
    sender_domain: String,
    recipient_email: String,
}

impl From<Config> for EmailConfig {
    fn from(config: Config) -> Self {
        EmailConfig {
            smtp_username: config.smtp_username,
            smtp_password: config.smtp_password,
            smtp_server: config.smtp_server,
            sender_domain: config.sender_domain,
            recipient_email: config.recipient_email,
        }
    }
}

pub async fn send_email(
    config: &EmailConfig,
    subject: &str,
    body: &str,
    sender: &str,
) -> Result<(), String> {
    let sender = format!("{}@{}", sender, config.sender_domain);
    let email = Message::builder()
        .from(sender.parse().unwrap())
        .to(config.recipient_email.parse().unwrap())
        .subject(subject)
        .body(body.to_string())
        .map_err(|e| format!("Failed to build email: {}", e))?;

    let creds = Credentials::new(config.smtp_username.clone(), config.smtp_password.clone());

    let mailer = SmtpTransport::relay(&config.smtp_server)
        .map_err(|e| format!("Failed to connect to SMTP server: {}", e))?
        .credentials(creds)
        .build();

    mailer
        .send(&email)
        .map_err(|e| format!("Failed to send email: {}", e))?;

    Ok(())
}
