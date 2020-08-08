use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::constants;
use crate::errors::ApiError;
use crate::opts::Opts;
use crate::types::{Proxy, Response};

#[derive(Clone, Debug)]
pub struct Fetcher {
    last_fetched: Arc<Mutex<Instant>>,
    opts: Opts,
    proxies: Vec<Proxy>,
}

impl Fetcher {
    fn new(last_fetched: Arc<Mutex<Instant>>, opts: Opts) -> Self {
        Self {
            last_fetched,
            opts,
            proxies: Vec::new(),
        }
    }

    // TODO: double check names for proxy stuff and use those
    pub fn try_get(&mut self, amount: usize) -> Result<Vec<Proxy>, ApiError> {
        // Yes the API says 1 second delay, but I was still occasionally getting rate limited,
        // and 1.05 seconds was also causing problems, so 1.1 is the new delay.
        const DELAY: Duration = Duration::from_millis(1100);

        if self.proxies.len() >= amount {
            // If there's enough in the current list then just go ahead and fulfill
            Ok(self.proxies.split_off(amount))
        } else {
            // Otherwise we need to lock and request the api
            let params = serde_url_params::to_string(&self.opts).expect(&format!(
                "Failed to serialize url, please raise an issue to adress this: {}",
                constants::REPO_URI
            ));
            let mut request = ureq::get(constants::API_URI).query_str(&params).build();

            // TODO: don't just blindly unwrap here later, need to check if it was poisioned and
            //       then default to now?
            let mut last_fetched = self.last_fetched.lock().unwrap();

            while self.proxies.len() < amount {
                // Delay to prevent rate limiting
                let delta = Instant::now().duration_since(*last_fetched);
                if delta < DELAY {
                    thread::sleep(DELAY - delta);
                }

                let resp = request.call();
                // Update the request time
                *last_fetched = Instant::now();

                let mut proxies = self.validate(resp)?;
                self.proxies.append(&mut proxies);
            }

            Ok(self.proxies.split_off(amount))
        }
    }

    fn validate(&self, resp: ureq::Response) -> Result<Vec<Proxy>, ApiError> {
        if resp.ok() {
            let resp_str = resp
                .into_string()
                .expect("Failed converting response to string");

            match serde_json::from_str::<Response>(&resp_str) {
                Ok(response) => Ok(response.data),
                Err(_) => Err(ApiError::from(resp_str)),
            }
        } else {
            Err(ApiError::from(resp))
        }
    }

    pub fn drain(self) -> Vec<Proxy> {
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
