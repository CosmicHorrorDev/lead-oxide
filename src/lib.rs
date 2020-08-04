use std::default::Default;
use std::net::IpAddr;
use std::num::NonZeroU16;
use std::sync::Arc;
use std::time::{Duration, Instant};

use derive_builder::Builder;

// TODO: should this be https?
const API_URI: &str = "http://pubproxy.com/api/proxy?";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Anonymous,
    Elite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protocol {
    Http,
    Socks4,
    Socks5,
}

#[derive(Builder, Clone, Default, Debug)]
#[builder(setter(strip_option))]
#[builder(default)]
pub struct Opts {
    api_key: Option<String>,
    level: Option<Level>,
    protocol: Option<Protocol>,
    countries: Vec<String>,
    not_countries: Vec<String>,
    last_checked: Option<Duration>,
    #[builder(setter(into))]
    port: Option<NonZeroU16>,
    time_to_connect: Option<Duration>,
    cookies: Option<bool>,
    connects_to_google: Option<bool>,
    https: Option<bool>,
    post: Option<bool>,
    referer: Option<bool>,
    forwards_user_agent: Option<bool>,
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
