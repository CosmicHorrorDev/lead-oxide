use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;

use crate::opts::Opts;

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
