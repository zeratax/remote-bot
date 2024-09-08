use chrono::FixedOffset;
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_fixed_offset")]
    pub timezone: FixedOffset, // Now we will manually deserialize this field
    pub discord_token: String,
    pub recipient_email: String,
    pub sender_email: String,
    pub smtp_password: String,
    pub smtp_server: String,
    pub smtp_username: String,
}

// Custom deserialization function for FixedOffset
fn deserialize_fixed_offset<'de, D>(deserializer: D) -> Result<FixedOffset, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?; // Deserialize the string first
    FixedOffset::from_str(&s).map_err(serde::de::Error::custom) // Try to parse it using FixedOffset::from_str
}
