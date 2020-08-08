use std::fmt::Display;
use std::net::Ipv4Addr;
use std::str::FromStr;

use chrono::naive::NaiveDateTime;

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
pub struct Response {
    pub data: Vec<Proxy>,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Proxy {
    pub ip: Ipv4Addr,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub port: u16,
    pub country: String,
    #[serde(deserialize_with = "deserialize_date")]
    pub last_checked: NaiveDateTime,
    #[serde(rename = "proxy_level")]
    pub level: Level,
    #[serde(rename = "type")]
    pub protocol: Protocol,
    #[serde(rename = "speed", deserialize_with = "deserialize_from_str")]
    pub time_to_connect: u8,
    #[serde(rename = "support")]
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

fn deserialize_date<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let time_fmt = "%Y-%m-%d %H:%M:%S";
    let s: String = Deserialize::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, time_fmt).map_err(de::Error::custom)
}

// TODO: convert these back to bool
#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Supports {
    pub https: u8,
    pub get: u8,
    pub post: u8,
    pub cookies: u8,
    pub referer: u8,
    #[serde(rename = "user_agent")]
    pub forwards_user_agent: u8,
    #[serde(rename = "google")]
    pub connects_to_google: u8,
}
