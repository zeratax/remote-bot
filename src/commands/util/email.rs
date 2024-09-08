use crate::configuration::Config;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

pub async fn send_email(
    config: &Config,
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
