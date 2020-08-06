use std::net::IpAddr;
use std::str::FromStr;

use chrono::{DateTime, Utc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Anonymous,
    Elite,
}

impl FromStr for Level {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "anonymous" => Ok(Self::Anonymous),
            "elite" => Ok(Self::Elite),
            _ => todo!(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protocol {
    Http,
    Socks4,
    Socks5,
}

impl FromStr for Protocol {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "http" => Ok(Self::Http),
            "socks4" => Ok(Self::Socks4),
            "socks5" => Ok(Self::Socks5),
            _ => todo!(),
        }
    }
}

pub struct Proxy {
    pub ip_port: IpAddr,
    pub country: String,
    pub last_checked: DateTime<Utc>,
    // TODO: rename
    pub level: Level,
    // TODO: rename
    pub protocol: Protocol,
    // TODO: rename
    pub time_to_connect: u8,
    pub supports: Supports,
}

pub struct Supports {
    pub https: bool,
    pub get: bool,
    pub post: bool,
    pub cookies: bool,
    pub referer: bool,
    pub user_agent: bool,
    pub google: bool,
}
