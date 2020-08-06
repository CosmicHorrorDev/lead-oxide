use std::default::Default;
use std::net::IpAddr;
use std::num::NonZeroU16;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::types::{Level, Protocol};

#[derive(Clone, Default, Debug)]
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
    pub fn api_key(&mut self, api_key: String) -> &mut Self {
        self.api_key = Some(api_key);
        self
    }

    pub fn level(&mut self, level: Level) -> &mut Self {
        self.level = Some(level);
        self
    }

    pub fn protocol(&mut self, protocol: Protocol) -> &mut Self {
        self.protocol = Some(protocol);
        self
    }

    pub fn country(&mut self, country: String) -> &mut Self {
        self.countries.push(country);
        self
    }

    pub fn countries<I>(&mut self, countries: I) -> &mut Self
    where
        I: Iterator<Item = String>,
    {
        self.countries = countries.collect();
        self
    }

    pub fn not_country(&mut self, not_country: String) -> &mut Self {
        self.not_countries.push(not_country);
        self
    }

    pub fn not_countries<I>(&mut self, not_countries: I) -> &mut Self
    where
        I: Iterator<Item = String>,
    {
        self.not_countries = not_countries.collect();
        self
    }

    pub fn last_checked(&mut self, last_checked: Duration) -> &mut Self {
        self.last_checked = Some(last_checked);
        self
    }

    pub fn port(&mut self, port: NonZeroU16) -> &mut Self {
        self.port = Some(port);
        self
    }

    pub fn time_to_connect(&mut self, time_to_connect: Duration) -> &mut Self {
        self.time_to_connect = Some(time_to_connect);
        self
    }

    pub fn cookies(&mut self, cookies: bool) -> &mut Self {
        self.cookies = Some(cookies);
        self
    }

    pub fn connects_to_google(&mut self, connects_to_google: bool) -> &mut Self {
        self.connects_to_google = Some(connects_to_google);
        self
    }

    pub fn https(&mut self, https: bool) -> &mut Self {
        self.https = Some(https);
        self
    }

    pub fn post(&mut self, post: bool) -> &mut Self {
        self.post = Some(post);
        self
    }

    pub fn referer(&mut self, referer: bool) -> &mut Self {
        self.referer = Some(referer);
        self
    }

    pub fn forwards_user_agent(&mut self, forwards_user_agent: bool) -> &mut Self {
        self.forwards_user_agent = Some(forwards_user_agent);
        self
    }
}

#[derive(Clone, Debug)]
pub struct Fetcher {
    last_fetched: Arc<Instant>,
    opts: Opts,
    proxies: Vec<IpAddr>,
}

impl Fetcher {
    fn new(last_fetched: Arc<Instant>, opts: Opts) -> Self {
        Self {
            last_fetched,
            opts,
            proxies: Vec::new(),
        }
    }

    pub fn try_get(&mut self, amount: u16) -> Result<Vec<IpAddr>, ()> {
        todo!()
    }

    pub fn drain(self) -> Vec<IpAddr> {
        self.proxies
    }
}

#[derive(Debug)]
pub struct Session {
    last_fetched: Arc<Instant>,
}

impl Session {
    pub fn new() -> Self {
        Session {
            last_fetched: Arc::new(Instant::now()),
        }
    }

    pub fn spawn_fetcher(&self, opts: Opts) -> Fetcher {
        Fetcher::new(self.last_fetched.clone(), opts)
    }
}
