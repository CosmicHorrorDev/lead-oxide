use std::{convert::TryFrom, num::NonZeroU16};

use crate::{
    errors::ParamError,
    types::{Countries, LastChecked, Level, Protocol, TimeToConnect},
};

use serde::Serialize;
use serde_repr::Serialize_repr;

// TODO: allow for multiple things being specified on the different things that accept it?
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OptsBuilder {
    api_key: Option<String>,
    level: Option<Level>,
    protocol: Option<Protocol>,
    countries: Option<Countries>,
    last_checked: Option<LastChecked>,
    port: Option<NonZeroU16>,
    time_to_connect: Option<TimeToConnect>,
    cookies: Option<bool>,
    connects_to_google: Option<bool>,
    https: Option<bool>,
    post: Option<bool>,
    referer: Option<bool>,
    forwards_user_agent: Option<bool>,
}

impl OptsBuilder {
    pub fn api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self
    }

    // TODO: is there terminology used to define this?
    pub fn level(mut self, level: Level) -> Self {
        self.level = Some(level);
        self
    }

    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = Some(protocol);
        self
    }

    pub fn countries(mut self, countries: Countries) -> Self {
        self.countries = Some(countries);
        self
    }

    pub fn last_checked(mut self, last_checked: LastChecked) -> Self {
        self.last_checked = Some(last_checked);
        self
    }

    pub fn port(mut self, port: NonZeroU16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn time_to_connect(mut self, time_to_connect: TimeToConnect) -> Self {
        self.time_to_connect = Some(time_to_connect);
        self
    }

    pub fn cookies(mut self, cookies: bool) -> Self {
        self.cookies = Some(cookies);
        self
    }

    pub fn connects_to_google(mut self, connects_to_google: bool) -> Self {
        self.connects_to_google = Some(connects_to_google);
        self
    }

    pub fn https(mut self, https: bool) -> Self {
        self.https = Some(https);
        self
    }

    pub fn post(mut self, post: bool) -> Self {
        self.post = Some(post);
        self
    }

    pub fn referer(mut self, referer: bool) -> Self {
        self.referer = Some(referer);
        self
    }

    pub fn forwards_user_agent(mut self, forwards_user_agent: bool) -> Self {
        self.forwards_user_agent = Some(forwards_user_agent);
        self
    }

    // TODO: move this error to only the values that could actually fail
    pub fn try_build(self) -> Result<Opts, ParamError> {
        Opts::try_from(self)
    }
}

#[derive(Serialize_repr, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Limit {
    Free = 5,
    Premium = 20,
}

impl Default for Limit {
    fn default() -> Self {
        Self::Free
    }
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Format {
    // Techically txt is also allowed, but this library only uses json
    Json,
}

impl Default for Format {
    fn default() -> Self {
        Self::Json
    }
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct Opts {
    #[serde(rename = "api")]
    api_key: Option<String>,
    level: Option<Level>,
    #[serde(rename = "type")]
    protocol: Option<Protocol>,
    // An enpty country list is essentially `None`
    #[serde(flatten, skip_serializing_if = "Countries::is_empty")]
    countries: Countries,
    #[serde(rename = "last_check")]
    last_checked: Option<u64>,
    // Note: using a port of 0 will return any port from the api :silly:
    port: Option<NonZeroU16>,
    #[serde(rename = "speed")]
    time_to_connect: Option<u64>,
    cookies: Option<bool>,
    #[serde(rename = "google")]
    connects_to_google: Option<bool>,
    https: Option<bool>,
    post: Option<bool>,
    referer: Option<bool>,
    #[serde(rename = "user_agent")]
    forwards_user_agent: Option<bool>,
    pub(crate) limit: Limit,
    format: Format,
}

impl Opts {
    pub fn builder() -> OptsBuilder {
        OptsBuilder::default()
    }

    pub(crate) fn is_premium(&self) -> bool {
        self.api_key.is_some()
    }
}

impl TryFrom<OptsBuilder> for Opts {
    type Error = ParamError;

    fn try_from(builder: OptsBuilder) -> Result<Self, Self::Error> {
        Ok(Self {
            api_key: builder.api_key.clone(),
            level: builder.level,
            protocol: builder.protocol,
            countries: builder.countries.unwrap_or_default(),
            last_checked: builder
                .last_checked
                .map(|last_checked| last_checked.value().as_secs() / 60),
            port: builder.port,
            time_to_connect: builder
                .time_to_connect
                .map(|time_to_connect| time_to_connect.value().as_secs()),
            cookies: builder.cookies,
            connects_to_google: builder.connects_to_google,
            https: builder.https,
            post: builder.post,
            referer: builder.referer,
            forwards_user_agent: builder.forwards_user_agent,
            limit: match builder.api_key {
                Some(_) => Limit::Premium,
                None => Limit::Free,
            },
            format: Format::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    use iso_country::Country;

    #[test]
    fn url_serialization() -> Result<(), serde_urlencoded::ser::Error> {
        let check_equal_params = |opts, expected: &[&str]| {
            // Convert `opts` to a url and sort the values
            let url = serde_urlencoded::to_string(&opts)?;
            let mut params: Vec<_> = url.split('&').map(String::from).collect();
            params.sort();

            // Sort the `expected` values
            let mut expected = expected.to_vec();
            expected.sort();

            assert_eq!(params, expected);

            Ok(())
        };

        // Base `Opts`
        check_equal_params(Opts::default(), &["format=json", "limit=5"])?;
        // Using a key will up the limit
        check_equal_params(
            Opts::builder().api_key("<key>").try_build().unwrap(),
            &["api=%3Ckey%3E", "format=json", "limit=20"],
        )?;
        // Empty countries list won't be included (api seems to work with an empty list, but I don't
        // want to rely on this behavior
        check_equal_params(
            Opts::builder()
                .countries(Countries::default())
                .try_build()
                .unwrap(),
            &["format=json", "limit=5"],
        )?;
        // Kitchen sink
        check_equal_params(
            Opts::builder()
                .api_key("<key>")
                .level(Level::Elite)
                .protocol(Protocol::Socks4)
                .countries(Countries::block().countries(&[Country::CH, Country::ES]))
                .last_checked(LastChecked::try_from(Duration::from_secs(60 * 10)).unwrap())
                .time_to_connect(TimeToConnect::try_from(Duration::from_secs(10)).unwrap())
                .port(NonZeroU16::new(8080).unwrap())
                .cookies(true)
                .connects_to_google(false)
                .https(true)
                .post(false)
                .referer(true)
                .forwards_user_agent(false)
                .try_build()
                .unwrap(),
            &[
                // Automatic
                "limit=20",
                "format=json",
                // Key
                "api=%3Ckey%3E",
                // Enums
                "level=elite",
                "type=socks4",
                "not_country=CH%2CES",
                // Durations
                "last_check=10",
                "speed=10",
                // NonZero
                "port=8080",
                // Bools
                "cookies=true",
                "google=false",
                "https=true",
                "post=false",
                "referer=true",
                "user_agent=false",
            ],
        )
    }
}
