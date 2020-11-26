use std::{convert::TryFrom, num::NonZeroU16, time::Duration};

use crate::{
    errors::ParamError,
    types::{Countries, Level, Protocol},
};

use serde::Serialize;
use serde_repr::Serialize_repr;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OptsBuilder {
    api_key: Option<String>,
    level: Option<Level>,
    protocol: Option<Protocol>,
    countries: Option<Countries>,
    last_checked: Option<Duration>,
    port: Option<NonZeroU16>,
    time_to_connect: Option<Duration>,
    cookies: Option<bool>,
    connects_to_google: Option<bool>,
    https: Option<bool>,
    post: Option<bool>,
    referer: Option<bool>,
    forwards_user_agent: Option<bool>,
}

impl OptsBuilder {
    #[must_use]
    pub fn api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self
    }

    #[must_use]
    pub fn level(mut self, level: Level) -> Self {
        self.level = Some(level);
        self
    }

    #[must_use]
    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = Some(protocol);
        self
    }

    #[must_use]
    pub fn countries(mut self, countries: Countries) -> Self {
        self.countries = Some(countries);
        self
    }

    #[must_use]
    pub fn last_checked(mut self, last_checked: Duration) -> Self {
        self.last_checked = Some(last_checked);
        self
    }

    #[must_use]
    pub fn port(mut self, port: NonZeroU16) -> Self {
        self.port = Some(port);
        self
    }

    #[must_use]
    pub fn time_to_connect(mut self, time_to_connect: Duration) -> Self {
        self.time_to_connect = Some(time_to_connect);
        self
    }

    #[must_use]
    pub fn cookies(mut self, cookies: bool) -> Self {
        self.cookies = Some(cookies);
        self
    }

    #[must_use]
    pub fn connects_to_google(mut self, connects_to_google: bool) -> Self {
        self.connects_to_google = Some(connects_to_google);
        self
    }

    #[must_use]
    pub fn https(mut self, https: bool) -> Self {
        self.https = Some(https);
        self
    }

    #[must_use]
    pub fn post(mut self, post: bool) -> Self {
        self.post = Some(post);
        self
    }

    #[must_use]
    pub fn referer(mut self, referer: bool) -> Self {
        self.referer = Some(referer);
        self
    }

    #[must_use]
    pub fn forwards_user_agent(mut self, forwards_user_agent: bool) -> Self {
        self.forwards_user_agent = Some(forwards_user_agent);
        self
    }

    pub fn try_build(self) -> Result<Opts, ParamError> {
        Opts::try_from(self)
    }
}

#[derive(Serialize_repr, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Limit {
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
pub enum Format {
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
    #[serde(flatten)]
    countries: Option<Countries>,
    #[serde(rename = "last_check")]
    last_checked: Option<u64>,
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
    // Note: Limit is only used publicly for testing. It is not considered part of the public api
    #[doc(hidden)]
    pub limit: Limit,
    format: Format,
}

impl Opts {
    #[must_use]
    pub fn builder() -> OptsBuilder {
        OptsBuilder::default()
    }
}

impl TryFrom<OptsBuilder> for Opts {
    type Error = ParamError;

    fn try_from(builder: OptsBuilder) -> Result<Self, Self::Error> {
        // TODO: can the newtype pattern be used here to make the bounds checking nicer?
        let bounds_check = |val, param_name, bounds: (Duration, Duration)| {
            match val {
                // Check that duration is within bounds if it exists
                Some(duration) if duration < bounds.0 || duration > bounds.1 => {
                    Err(Self::Error::OutOfBounds {
                        param: String::from(param_name),
                        bounds,
                        value: duration,
                    })
                }
                _ => Ok(()),
            }
        };

        bounds_check(
            builder.last_checked,
            "last_checked",
            (Duration::from_secs(60), Duration::from_secs(60 * 60)),
        )?;

        bounds_check(
            builder.time_to_connect,
            "time_to_connect",
            (Duration::from_secs(1), Duration::from_secs(60)),
        )?;

        Ok(Self {
            api_key: builder.api_key.clone(),
            level: builder.level,
            protocol: builder.protocol,
            countries: builder.countries,
            last_checked: builder.last_checked.map(|duration| duration.as_secs() / 60),
            port: builder.port,
            time_to_connect: builder.time_to_connect.map(|duration| duration.as_secs()),
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

    #[test]
    fn bounds_checking() {
        // Check the bounds
        let bad_opts = Opts::builder()
            .time_to_connect(Duration::from_secs(0))
            .try_build();
        assert!(bad_opts.is_err());

        let bad_opts = Opts::builder()
            .time_to_connect(Duration::from_secs(61))
            .try_build();
        assert!(bad_opts.is_err());

        let bad_opts = Opts::builder()
            .last_checked(Duration::from_secs(1000 * 60 + 1))
            .try_build();
        assert!(bad_opts.is_err());

        let bad_opts = Opts::builder()
            .last_checked(Duration::from_secs(0))
            .try_build();
        assert!(bad_opts.is_err());
    }

    #[test]
    fn url_serialization() -> Result<(), serde_urlencoded::ser::Error> {
        let split_and_sort = |s: String| {
            let mut pieces: Vec<_> = s.split('&').map(String::from).collect();
            pieces.sort();
            pieces
        };

        let check_equal_params = |opts, expected: &[&str]| {
            // Convert `opts` to a url and sort the values
            let url = serde_urlencoded::to_string(&opts)?;
            let params = split_and_sort(url);

            // Sort the `expected` values
            let mut expected = expected.to_vec();
            expected.sort();

            assert_eq!(params, expected);

            Ok(())
        };

        // Now to test a variety of Opts
        check_equal_params(Opts::default(), &["format=json", "limit=5"])?;
        check_equal_params(
            Opts::builder().api_key("<key>").try_build().unwrap(),
            &["api=%3Ckey%3E", "format=json", "limit=20"],
        )?;
        check_equal_params(
            Opts::builder()
                .api_key("<key>")
                .level(Level::Elite)
                .protocol(Protocol::Socks4)
                .countries(Countries::block().country("ZH").country("ES"))
                .last_checked(Duration::from_secs(60 * 10))
                .time_to_connect(Duration::from_secs(10))
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
                "not_countries=ZH%2CES",
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
