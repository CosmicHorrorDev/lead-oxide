use std::{fmt::Display, net::Ipv4Addr, str::FromStr};

use chrono::naive::NaiveDateTime;

use serde::{de, Deserialize, Deserializer, Serialize};

#[derive(Serialize, Clone, Debug, PartialEq)]
pub enum Countries {
    #[serde(rename = "countries")]
    AllowList(String),
    #[serde(rename = "not_countries")]
    BlockList(String),
}

impl Countries {
    #[must_use]
    pub fn allow() -> Self {
        Self::AllowList(String::new())
    }

    #[must_use]
    pub fn block() -> Self {
        Self::BlockList(String::new())
    }

    #[must_use]
    pub fn country(self, country: &str) -> Self {
        let smart_join = |list: String, new| {
            if list.is_empty() {
                String::from(new)
            } else {
                format!("{},{}", list, new)
            }
        };

        match self {
            Self::AllowList(list) => Self::AllowList(smart_join(list, country)),
            Self::BlockList(list) => Self::BlockList(smart_join(list, country)),
        }
    }
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
    // TODO: Combine this and the port number for a socketaddr? How to handle this
    pub ip: Ipv4Addr,
    #[serde(deserialize_with = "deserialize_from_str")]
    // TODO: switch to non-zero u16
    pub port: u16,
    pub country: String,
    #[serde(deserialize_with = "deserialize_date")]
    pub last_checked: NaiveDateTime,
    #[serde(rename = "proxy_level")]
    pub level: Level,
    #[serde(rename = "type")]
    pub protocol: Protocol,
    #[serde(rename = "speed", deserialize_with = "deserialize_from_str")]
    // TODO: switch to duration
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

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub struct Supports {
    // TODO: is there a better way to handle this deserialization?
    #[serde(deserialize_with = "deserialize_bool")]
    pub https: bool,
    #[serde(deserialize_with = "deserialize_bool")]
    pub get: bool,
    #[serde(deserialize_with = "deserialize_bool")]
    pub post: bool,
    #[serde(deserialize_with = "deserialize_bool")]
    pub cookies: bool,
    #[serde(deserialize_with = "deserialize_bool")]
    pub referer: bool,
    #[serde(rename = "user_agent", deserialize_with = "deserialize_bool")]
    pub forwards_user_agent: bool,
    #[serde(rename = "google", deserialize_with = "deserialize_bool")]
    pub connects_to_google: bool,
}

fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let byte: u8 = Deserialize::deserialize(deserializer)?;
    Ok(byte == 1)
}
