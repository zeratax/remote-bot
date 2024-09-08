use chrono::FixedOffset;
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_fixed_offset")]
    pub timezone: FixedOffset,
    pub discord_token: String,
    pub recipient_email: String,
    pub sender_domain: String,
    pub smtp_password: String,
    pub smtp_server: String,
    pub smtp_username: String,
}

fn deserialize_fixed_offset<'de, D>(deserializer: D) -> Result<FixedOffset, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    FixedOffset::from_str(&s).map_err(serde::de::Error::custom)
}
