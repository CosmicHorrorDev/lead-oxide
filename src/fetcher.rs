use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::opts::Opts;

#[derive(Clone, Debug)]
pub struct Fetcher {
    last_fetched: Arc<Mutex<Instant>>,
    opts: Opts,
    proxies: Vec<IpAddr>,
}

impl Fetcher {
    fn new(last_fetched: Arc<Mutex<Instant>>, opts: Opts) -> Self {
        Self {
            last_fetched,
            opts,
            proxies: Vec::new(),
        }
    }

    pub fn try_get(&mut self, amount: usize) -> Result<&[IpAddr], ()> {
        // Yes the API says 1 second delay, but I was still occasionally getting rate limited,
        // and 1.05 seconds was also causing problems, so 1.1 is the new delay.
        const DELAY: Duration = Duration::from_millis(1100);

        if self.proxies.len() >= amount {
            // If there's enough in the current list then just go ahead and fulfill
            Ok(&self.proxies[..amount])
        } else {
            // Otherwise we need to lock and request the api
            // TODO: don't just blindly unwrap here later
            let mut last_fetched = self.last_fetched.lock().unwrap();

            // Delay to prevent rate limiting
            let delta = Instant::now().duration_since(*last_fetched);
            if delta < DELAY {
                thread::sleep(delta);
            }

            // TODO: actually request the api

            // Update the request time
            *last_fetched = Instant::now();
            todo!();
        }
    }

    pub fn drain(self) -> Vec<IpAddr> {
        self.proxies
    }
}

#[derive(Debug)]
pub struct Session {
    last_fetched: Arc<Mutex<Instant>>,
}

impl Session {
    pub fn new() -> Self {
        Session {
            // FIXME: this doesn't make sense to start with now since it will block the first
            //        request for 1 second unnecessarily
            last_fetched: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn spawn_fetcher(&self, opts: Opts) -> Fetcher {
        Fetcher::new(self.last_fetched.clone(), opts)
    }
}
