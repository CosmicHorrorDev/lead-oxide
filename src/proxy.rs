use std::{net::SocketAddrV4, time::Duration};

use crate::types::{Level, Protocol};

use chrono::NaiveDateTime;
use iso_country::Country;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Response {
    pub data: Vec<RawProxy>,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct RawProxy {
    #[serde(rename = "ipPort")]
    socket: SocketAddrV4,
    country: Country,
    last_checked: String,
    #[serde(rename = "proxy_level")]
    level: Level,
    #[serde(rename = "type")]
    protocol: Protocol,
    #[serde(rename = "speed")]
    time_to_connect: String,
    #[serde(rename = "support")]
    supports: RawSupports,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
struct RawSupports {
    https: Option<u8>,
    get: Option<u8>,
    post: Option<u8>,
    cookies: Option<u8>,
    referer: Option<u8>,
    #[serde(rename = "user_agent")]
    forwards_user_agent: Option<u8>,
    #[serde(rename = "google")]
    connects_to_google: Option<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Proxy {
    pub socket: SocketAddrV4,
    pub country: Country,
    pub last_checked: NaiveDateTime,
    pub level: Level,
    pub protocol: Protocol,
    pub time_to_connect: Duration,
    pub supports: Supports,
}

impl From<RawProxy> for Proxy {
    fn from(raw: RawProxy) -> Self {
        let last_checked = NaiveDateTime::parse_from_str(&raw.last_checked, "%F %T")
            .expect("The API returned an invalid time");

        let secs_to_connect = raw
            .time_to_connect
            .parse()
            .expect("The API returned an invalid int");
        let time_to_connect = Duration::from_secs(secs_to_connect);

        Self {
            socket: raw.socket,
            country: raw.country,
            last_checked,
            level: raw.level,
            protocol: raw.protocol,
            time_to_connect,
            supports: Supports::from(raw.supports),
        }
    }
}

pub fn proxies_from_json(json: &str) -> Result<Vec<Proxy>, serde_json::Error> {
    let resp: Response = serde_json::from_str(json)?;
    Ok(resp
        .data
        .into_iter()
        .map(Proxy::from)
        // Just to play it safe we filter out any results with an incorrect country field. We could
        // be smarter and only use this in the presence of a blocklist if this causes issues.
        .filter(|Proxy { country, .. }| match country {
            Country::Unspecified => false,
            _ => true,
        })
        .collect())
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Supports {
    pub https: bool,
    pub get: bool,
    pub post: bool,
    pub cookies: bool,
    pub referer: bool,
    pub forwards_user_agent: bool,
    pub connects_to_google: bool,
}

impl From<RawSupports> for Supports {
    fn from(raw: RawSupports) -> Self {
        let parse_field = |field| match field {
            Some(val) => val == 1,
            // null is assumed to be false just to play it safe
            None => false,
        };

        Self {
            https: parse_field(raw.https),
            get: parse_field(raw.get),
            post: parse_field(raw.post),
            cookies: parse_field(raw.cookies),
            referer: parse_field(raw.referer),
            forwards_user_agent: parse_field(raw.forwards_user_agent),
            connects_to_google: parse_field(raw.connects_to_google),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn deserialization() -> Result<(), serde_json::Error> {
        // TODO: make this a platform independent path
        let raw_response = include_str!("test_data/response.json");
        let proxies = proxies_from_json(raw_response)?;

        let date = NaiveDate::from_ymd(2020, 12, 13);

        // The proxy with an empty country field got filtered out
        let ideal = vec![
            Proxy {
                socket: "67.225.164.154:80".parse().unwrap(),
                country: Country::US,
                last_checked: NaiveDateTime::new(date, NaiveTime::from_hms(20, 6, 41)),
                level: Level::Elite,
                protocol: Protocol::Http,
                time_to_connect: Duration::from_secs(10),
                supports: Supports {
                    get: true,
                    post: true,
                    cookies: true,
                    referer: true,
                    forwards_user_agent: true,
                    ..Supports::default()
                },
            },
            Proxy {
                socket: "35.181.4.4:80".parse().unwrap(),
                country: Country::US,
                last_checked: NaiveDateTime::new(date, NaiveTime::from_hms(20, 10, 11)),
                level: Level::Elite,
                protocol: Protocol::Http,
                time_to_connect: Duration::from_secs(1),
                supports: Supports {
                    forwards_user_agent: true,
                    ..Supports::default()
                },
            },
            Proxy {
                socket: "89.24.76.185:32842".parse().unwrap(),
                country: Country::CZ,
                last_checked: NaiveDateTime::new(date, NaiveTime::from_hms(20, 1, 52)),
                level: Level::Elite,
                protocol: Protocol::Socks5,
                time_to_connect: Duration::from_secs(18),
                supports: Supports {
                    get: true,
                    post: true,
                    cookies: true,
                    referer: true,
                    forwards_user_agent: true,
                    ..Supports::default()
                },
            },
            Proxy {
                socket: "125.99.120.166:40390".parse().unwrap(),
                country: Country::IN,
                last_checked: NaiveDateTime::new(date, NaiveTime::from_hms(20, 10, 11)),
                level: Level::Elite,
                protocol: Protocol::Socks4,
                time_to_connect: Duration::from_secs(14),
                supports: Supports {
                    get: true,
                    post: true,
                    cookies: true,
                    referer: true,
                    forwards_user_agent: true,
                    ..Supports::default()
                },
            },
        ];

        for (i, (parsed, desired)) in proxies.iter().zip(ideal.iter()).enumerate() {
            eprintln!("Checking proxy {}", i);
            assert_eq!(parsed, desired);
        }
        assert_eq!(proxies, ideal);

        Ok(())
    }
}
