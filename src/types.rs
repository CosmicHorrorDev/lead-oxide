use std::{fmt::Display, net::Ipv4Addr, str::FromStr};

use chrono::naive::NaiveDateTime;

use serde::{de, Deserialize, Deserializer, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Allow,
    Block,
}

impl Default for Action {
    fn default() -> Self {
        Self::Block
    }
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub enum Countries {
    #[serde(rename = "country")]
    AllowList(String),
    #[serde(rename = "not_country")]
    BlockList(String),
}

// TODO: as far as I know this only takes 2 letter lang codes, so maybe adding try build is for the
// best. This also allows us to switch the builder to store the list in a single internal Vec
// TODO: there's probably some resource that provides the 2 letter lang codes. Look around
// TODO: provide a `countries` method for specifying multiple at once
// TODO: does passing in an empty list of countries serialize correctly?
impl Countries {
    #[must_use]
    pub fn allow() -> CountriesBuilder {
        CountriesBuilder::new(Action::Allow)
    }

    #[must_use]
    pub fn block() -> CountriesBuilder {
        CountriesBuilder::new(Action::Block)
    }

    #[must_use]
    pub fn allowlist(countries: &[&str]) -> Self {
        Self::allow().countries(countries).build()
    }

    #[must_use]
    pub fn blocklist(countries: &[&str]) -> Self {
        Self::block().countries(countries).build()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::AllowList(countries) => countries.is_empty(),
            Self::BlockList(countries) => countries.is_empty(),
        }
    }
}

impl Default for Countries {
    fn default() -> Self {
        CountriesBuilder::default().build()
    }
}

impl From<CountriesBuilder> for Countries {
    fn from(builder: CountriesBuilder) -> Self {
        let CountriesBuilder { list, action } = builder;

        match action {
            Action::Allow => Self::AllowList(list.join(",")),
            Action::Block => Self::BlockList(list.join(",")),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CountriesBuilder {
    list: Vec<String>,
    action: Action,
}

impl CountriesBuilder {
    #[must_use]
    fn new(action: Action) -> Self {
        Self {
            list: Vec::new(),
            action,
        }
    }

    #[must_use]
    pub fn country(mut self, country: &str) -> Self {
        self.list.push(country.to_string());
        self
    }

    #[must_use]
    pub fn countries(mut self, countries: &[&str]) -> Self {
        for country in countries {
            self = self.country(country);
        }

        self
    }

    #[must_use]
    pub fn build(self) -> Countries {
        Countries::from(self)
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

// TODO: Is this needed? Can we just pull out "data" directly somehow?
// Note: Interal api only
#[doc(hidden)]
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
