use std::convert::TryFrom;
use std::default::Default;
use std::num::NonZeroU16;
use std::time::Duration;

use crate::errors::ParamError;
use crate::types::{Countries, Level, Protocol};

use serde::Serialize;

#[derive(getset::Getters, Clone, Debug, Default, PartialEq)]
pub struct OptsBuilder {
    // TODO: look into why this doesn't work on the struct itself
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

#[derive(Serialize, Clone, Debug, PartialEq)]
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
    limit: u8,
    format: String,
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
        Opts {
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
                Some(_) => 20,
                None => 5,
            },
            format: String::from("json"),
        }
    }
}

impl Default for Opts {
    fn default() -> Self {
        Opts {
            api_key: Option::default(),
            level: Option::default(),
            protocol: Option::default(),
            countries: Option::default(),
            last_checked: Option::default(),
            port: Option::default(),
            time_to_connect: Option::default(),
            cookies: Option::default(),
            connects_to_google: Option::default(),
            https: Option::default(),
            post: Option::default(),
            referer: Option::default(),
            forwards_user_agent: Option::default(),
            limit: 5,
            format: String::from("json"),
        }
    }
}

impl TryFrom<OptsBuilder> for Opts {
    type Error = ParamError;

    fn try_from(builder: OptsBuilder) -> Result<Self, Self::Error> {
        let bounds_check = |val: Option<Duration>,
                            param_name: &str,
                            bounds: (Duration, Duration)|
         -> Result<(), Self::Error> {
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
            (Duration::new(1 * 60, 0), Duration::new(60 * 60, 0)),
        )?;

        bounds_check(
            builder.time_to_connect,
            "time_to_connect",
            (Duration::new(1, 0), Duration::new(60, 0)),
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
            .time_to_connect(Duration::new(0, 0))
            .try_build();
        assert!(bad_opts.is_err());

        let bad_opts = Opts::builder()
            .time_to_connect(Duration::new(61, 0))
            .try_build();
        assert!(bad_opts.is_err());

        let bad_opts = Opts::builder()
            .last_checked(Duration::new(1000 * 60 + 1, 0))
            .try_build();
        assert!(bad_opts.is_err());

        let bad_opts = Opts::builder()
            .last_checked(Duration::new(0, 0))
            .try_build();
        assert!(bad_opts.is_err());

        // Check full param listing
        let opts = Opts::builder()
            .api_key("<key>")
            .level(Level::Elite)
            .protocol(Protocol::Socks4)
            .countries(Countries::BlockList(vec![
                String::from("ZH"),
                String::from("ES"),
            ]))
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
                countries: Some(Countries::BlockList(vec![
                    String::from("ZH"),
                    String::from("ES")
                ])),
                last_checked: Some(10),
                port: Some(NonZeroU16::new(8080).unwrap()),
                time_to_connect: Some(10),
                cookies: Some(true),
                connects_to_google: Some(false),
                https: Some(true),
                post: Some(false),
                referer: Some(true),
                forwards_user_agent: Some(false),
                limit: 20,
                format: String::from("json")
            }
        );
    }
}