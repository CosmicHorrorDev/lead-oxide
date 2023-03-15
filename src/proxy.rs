//! [`Proxy`][Proxy]s represent information about the proxies returned by
//! [`Fetcher`][crate::fetcher::Fetcher].

use std::{net::SocketAddrV4, time::Duration};

use crate::{
    constants::REPO_URI,
    types::{Level, Protocol},
};

use chrono::NaiveDateTime;
use iso_country::Country;
use serde::{de::Deserializer, Deserialize};

/// Internal
#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Response {
    pub data: Vec<RawProxy>,
}

/// Internal
#[derive(Deserialize, Clone, Debug, PartialEq)]
struct RawProxy {
    #[serde(rename = "ipPort")]
    socket: SocketAddrV4,
    #[serde(deserialize_with = "ignore_bad_countries")]
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

// Sometimes country codes other than iso 3166-1 are returned so switch those to unspecified
/// Internal
fn ignore_bad_countries<'de, D>(deserializer: D) -> Result<Country, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).or(Ok(Country::Unspecified))
}

/// Internal
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

/// All the information representing a proxy.
///
/// Typically most people will likely only use the `socket` value, but this contains all the
/// information on a proxy.
#[derive(Clone, Debug, PartialEq, Eq)]
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
            .unwrap_or_else(|_| {
                panic!(
                    "The API returned an invalid time. Please raise an issue to address this at {}",
                    REPO_URI
                )
            });

        let secs_to_connect = raw.time_to_connect.parse().unwrap_or_else(|_| {
            panic!(
                "The API returned an invalid int. Please raise an issue to address this at {}",
                REPO_URI
            )
        });
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

/// Internal
pub(crate) fn proxies_from_json(json: &str) -> Result<Vec<Proxy>, serde_json::Error> {
    let resp: Response = serde_json::from_str(json)?;
    Ok(resp
        .data
        .into_iter()
        .map(Proxy::from)
        // Just to play it safe we filter out any results with an incorrect country field. We could
        // be smarter and only use this in the presence of a blocklist if this causes issues. Just
        // to note this is typically less than 10% or responses.
        .filter(|&Proxy { country, .. }| country != Country::Unspecified)
        .collect())
}

/// Represents all the attributes that the [`Proxy`][Proxy] supports.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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

    use chrono::NaiveDate;
    use std::{fs, path::Path};

    #[test]
    fn deserialization() -> Result<(), serde_json::Error> {
        // Just some setup
        let sample_file = Path::new("tests").join("samples").join("response.json");
        let raw_response = fs::read_to_string(&sample_file).expect("Can't open the response file");

        // And now onto testing
        let proxies = proxies_from_json(&raw_response)?;

        let date = NaiveDate::from_ymd_opt(2020, 12, 13).unwrap();

        let common = Proxy {
            socket: "1.2.3.4:1234".parse().unwrap(),
            country: Country::US,
            last_checked: date.and_hms_opt(0, 0, 0).unwrap(),
            level: Level::Elite,
            protocol: Protocol::Http,
            time_to_connect: Duration::from_secs(0),
            supports: Supports {
                get: true,
                post: true,
                cookies: true,
                referer: true,
                forwards_user_agent: true,
                ..Supports::default()
            },
        };

        // The proxy with an empty country field got filtered out
        let ideal = vec![
            Proxy {
                socket: "67.225.164.154:80".parse().unwrap(),
                last_checked: date.and_hms_opt(20, 6, 41).unwrap(),
                time_to_connect: Duration::from_secs(10),
                ..common
            },
            Proxy {
                socket: "35.181.4.4:80".parse().unwrap(),
                last_checked: date.and_hms_opt(20, 10, 11).unwrap(),
                time_to_connect: Duration::from_secs(1),
                supports: Supports {
                    forwards_user_agent: true,
                    ..Supports::default()
                },
                ..common
            },
            Proxy {
                socket: "89.24.76.185:32842".parse().unwrap(),
                country: Country::CZ,
                last_checked: date.and_hms_opt(20, 1, 52).unwrap(),
                protocol: Protocol::Socks5,
                time_to_connect: Duration::from_secs(18),
                ..common
            },
            Proxy {
                socket: "125.99.120.166:40390".parse().unwrap(),
                country: Country::IN,
                last_checked: date.and_hms_opt(20, 10, 11).unwrap(),
                protocol: Protocol::Socks4,
                time_to_connect: Duration::from_secs(14),
                ..common
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
