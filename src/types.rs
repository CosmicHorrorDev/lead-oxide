use std::fmt::Display;
use std::net::IpAddr;
use std::str::FromStr;

use chrono::{DateTime, Utc};

use serde::{de, Deserialize, Deserializer, Serialize};

#[derive(Serialize, Clone, Debug, PartialEq)]
pub enum Countries {
    #[serde(rename = "countries")]
    AllowList(Vec<String>),
    #[serde(rename = "not_countries")]
    BlockList(Vec<String>),
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Anonymous,
    Elite,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Http,
    Socks4,
    Socks5,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Proxy {
    pub ip_port: IpAddr,
    pub country: String,
    #[serde(rename = "last_check", deserialize_with = "deserialize_from_str")]
    pub last_checked: DateTime<Utc>,
    pub level: Level,
    #[serde(rename = "type")]
    pub protocol: Protocol,
    #[serde(rename = "speed")]
    pub time_to_connect: u8,
    pub supports: Supports,
}

// Fallback for anything that doesn't implement `Deserialize`
fn deserialize_from_str<'de, S, D>(deserializer: D) -> Result<S, D::Error>
where
    S: FromStr,
    S::Err: Display,
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    S::from_str(&s).map_err(de::Error::custom)
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Supports {
    pub https: bool,
    pub get: bool,
    pub post: bool,
    pub cookies: bool,
    pub referer: bool,
    #[serde(rename = "user_agent")]
    pub forwards_user_agent: bool,
    #[serde(rename = "google")]
    pub connects_to_google: bool,
}
