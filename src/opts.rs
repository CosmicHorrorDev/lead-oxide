use std::{convert::TryFrom, num::NonZeroU16, time::Duration};

use crate::{
    errors::ParamError,
    types::{Countries, Level, Protocol},
};

use serde::Serialize;
use serde_repr::Serialize_repr;

// TODO: global "pub with_prefix" annotation is waiting on next `getset` release
// TODO: do other builders typically provide getters? Is there a use case?
#[derive(getset::Getters, Clone, Debug, Default, PartialEq)]
pub struct OptsBuilder {
    #[get = "pub with_prefix"]
    api_key: Option<String>,
    #[get = "pub with_prefix"]
    level: Option<Level>,
    #[get = "pub with_prefix"]
    protocol: Option<Protocol>,
    #[get = "pub with_prefix"]
    countries: Option<Countries>,
    #[get = "pub with_prefix"]
    last_checked: Option<Duration>,
    #[get = "pub with_prefix"]
    port: Option<NonZeroU16>,
    #[get = "pub with_prefix"]
    time_to_connect: Option<Duration>,
    #[get = "pub with_prefix"]
    cookies: Option<bool>,
    #[get = "pub with_prefix"]
    connects_to_google: Option<bool>,
    #[get = "pub with_prefix"]
    https: Option<bool>,
    #[get = "pub with_prefix"]
    post: Option<bool>,
    #[get = "pub with_prefix"]
    referer: Option<bool>,
    #[get = "pub with_prefix"]
    forwards_user_agent: Option<bool>,
}

impl OptsBuilder {
    pub fn api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self
    }

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

    pub fn last_checked(mut self, last_checked: Duration) -> Self {
        self.last_checked = Some(last_checked);
        self
    }

    pub fn port(mut self, port: NonZeroU16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn time_to_connect(mut self, time_to_connect: Duration) -> Self {
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

#[derive(getset::Getters, Serialize, Clone, Debug, Default, PartialEq)]
#[get = "pub with_prefix"]
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
    limit: Limit,
    format: Format,
}

impl Opts {
    pub fn builder() -> OptsBuilder {
        OptsBuilder::default()
    }

    fn new(
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
    ) -> Self {
        Self {
            api_key: api_key.clone(),
            level,
            protocol,
            countries,
            last_checked: last_checked.map(|duration| duration.as_secs() / 60),
            port,
            time_to_connect: time_to_connect.map(|duration| duration.as_secs()),
            cookies,
            connects_to_google,
            https,
            post,
            referer,
            forwards_user_agent,
            limit: match api_key {
                Some(_) => Limit::Premium,
                None => Limit::Free,
            },
            format: Format::default(),
        }
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
            (Duration::from_secs(1 * 60), Duration::from_secs(60 * 60)),
        )?;

        bounds_check(
            builder.time_to_connect,
            "time_to_connect",
            (Duration::from_secs(1), Duration::from_secs(60)),
        )?;

        Ok(Opts::new(
            builder.api_key,
            builder.level,
            builder.protocol,
            builder.countries,
            builder.last_checked,
            builder.port,
            builder.time_to_connect,
            builder.cookies,
            builder.connects_to_google,
            builder.https,
            builder.post,
            builder.referer,
            builder.forwards_user_agent,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opts_builder() {
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

        // TODO: this is done twice, is there a standard way of de-deuplicating a value like this?
        // Check full param listing
        let opts = Opts::builder()
            .api_key("<key>")
            .level(Level::Elite)
            .protocol(Protocol::Socks4)
            .countries(Countries::block().country("ZH").country("ES"))
            .last_checked(Duration::new(60 * 10, 0))
            .port(NonZeroU16::new(8080).unwrap())
            .time_to_connect(Duration::new(10, 0))
            .cookies(true)
            .connects_to_google(false)
            .https(true)
            .post(false)
            .referer(true)
            .forwards_user_agent(false)
            .try_build()
            .unwrap();

        assert_eq!(
            opts,
            Opts {
                api_key: Some(String::from("<key>")),
                level: Some(Level::Elite),
                protocol: Some(Protocol::Socks4),
                countries: Some(Countries::BlockList(String::from("ZH,ES"))),
                last_checked: Some(10),
                port: Some(NonZeroU16::new(8080).unwrap()),
                time_to_connect: Some(10),
                cookies: Some(true),
                connects_to_google: Some(false),
                https: Some(true),
                post: Some(false),
                referer: Some(true),
                forwards_user_agent: Some(false),
                limit: Limit::Premium,
                format: Format::Json
            }
        );
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
