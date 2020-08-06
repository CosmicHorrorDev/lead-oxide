use std::default::Default;
use std::num::NonZeroU16;
use std::time::Duration;

use crate::types::{Level, Protocol};

// TODO: need to do validation on different params since some are mutually exclusive or bounded
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Opts {
    api_key: Option<String>,
    level: Option<Level>,
    protocol: Option<Protocol>,
    countries: Vec<String>,
    not_countries: Vec<String>,
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

impl Opts {
    pub fn api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self
    }

    pub fn get_api_key(&self) -> Option<&String> {
        self.api_key.as_ref()
    }

    pub fn level(mut self, level: Level) -> Self {
        self.level = Some(level);
        self
    }

    pub fn get_level(&self) -> Option<Level> {
        self.level
    }

    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = Some(protocol);
        self
    }

    pub fn get_protocol(&self) -> Option<Protocol> {
        self.protocol
    }

    pub fn country(mut self, country: &str) -> Self {
        self.countries.push(country.to_string());
        self
    }

    pub fn countries<I>(mut self, countries: I) -> Self
    where
        I: Iterator<Item = String>,
    {
        self.countries = countries.collect();
        self
    }

    pub fn get_countries(&self) -> &[String] {
        self.countries.as_ref()
    }

    pub fn not_country(mut self, not_country: &str) -> Self {
        self.not_countries.push(not_country.to_string());
        self
    }

    pub fn not_countries<I>(mut self, not_countries: I) -> Self
    where
        I: Iterator<Item = String>,
    {
        self.not_countries = not_countries.collect();
        self
    }

    pub fn get_not_countries(&self) -> &[String] {
        self.not_countries.as_ref()
    }

    pub fn last_checked(mut self, last_checked: Duration) -> Self {
        self.last_checked = Some(last_checked);
        self
    }

    pub fn get_last_checked(&self) -> Option<Duration> {
        self.last_checked
    }

    pub fn port(mut self, port: NonZeroU16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn get_port(&self) -> Option<NonZeroU16> {
        self.port
    }

    pub fn time_to_connect(mut self, time_to_connect: Duration) -> Self {
        self.time_to_connect = Some(time_to_connect);
        self
    }

    pub fn get_time_to_connect(&self) -> Option<Duration> {
        self.time_to_connect
    }

    pub fn cookies(mut self, cookies: bool) -> Self {
        self.cookies = Some(cookies);
        self
    }

    pub fn get_cookies(&self) -> Option<bool> {
        self.cookies
    }

    pub fn connects_to_google(mut self, connects_to_google: bool) -> Self {
        self.connects_to_google = Some(connects_to_google);
        self
    }

    pub fn get_connects_to_google(&self) -> Option<bool> {
        self.connects_to_google
    }

    pub fn https(mut self, https: bool) -> Self {
        self.https = Some(https);
        self
    }

    pub fn get_https(&self) -> Option<bool> {
        self.https
    }

    pub fn post(mut self, post: bool) -> Self {
        self.post = Some(post);
        self
    }

    pub fn get_post(&self) -> Option<bool> {
        self.post
    }

    pub fn referer(mut self, referer: bool) -> Self {
        self.referer = Some(referer);
        self
    }

    pub fn get_referer(&self) -> Option<bool> {
        self.referer
    }

    pub fn forwards_user_agent(mut self, forwards_user_agent: bool) -> Self {
        self.forwards_user_agent = Some(forwards_user_agent);
        self
    }

    pub fn get_forwards_user_agent(&self) -> Option<bool> {
        self.forwards_user_agent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opts_builder() {
        let opts = Opts::default().country("US").country("CA");
        assert_eq!(opts.get_countries(), &["US", "CA"]);

        let opts = Opts::default()
            .api_key("<key>")
            .level(Level::Elite)
            .protocol(Protocol::Socks4)
            .not_countries(vec![String::from("ZH"), String::from("ES")].into_iter())
            .last_checked(Duration::new(60 * 10, 0))
            .port(NonZeroU16::new(8080).unwrap())
            .time_to_connect(Duration::new(10, 0))
            .cookies(true)
            .connects_to_google(false)
            .https(true)
            .post(false)
            .referer(true)
            .forwards_user_agent(false);

        assert_eq!(
            opts,
            Opts {
                api_key: Some(String::from("<key>")),
                level: Some(Level::Elite),
                protocol: Some(Protocol::Socks4),
                countries: Vec::default(),
                not_countries: vec![String::from("ZH"), String::from("ES")],
                last_checked: Some(Duration::new(60 * 10, 0)),
                port: Some(NonZeroU16::new(8080).unwrap()),
                time_to_connect: Some(Duration::new(10, 0)),
                cookies: Some(true),
                connects_to_google: Some(false),
                https: Some(true),
                post: Some(false),
                referer: Some(true),
                forwards_user_agent: Some(false),
            }
        );
    }
}
