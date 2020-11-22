use std::{
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use crate::{constants, errors::ApiError, opts::Opts, types::Proxy};

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

    pub fn try_get(&mut self, amount: usize) -> Result<Vec<Proxy>, ApiError> {
        if self.proxies.len() >= amount {
            // If there's enough in the current list then just go ahead and fulfill without locking
            Ok(self.proxies.split_off(self.proxies.len() - amount))
        } else {
            // Otherwise we need to lock and request the api
            let mut request = self.request_builder();

            let mut last_fetched = match self.last_fetched.lock() {
                Ok(last_fetched) => last_fetched,
                Err(err) => {
                    // If the lock was poisoned then play it safe and reset the timer
                    let mut poisioned = err.into_inner();
                    *poisioned = Instant::now();
                    poisioned
                }
            };

            while self.proxies.len() < amount {
                // Delay to prevent rate limiting
                let delta = Instant::now().duration_since(*last_fetched);
                if delta < constants::DELAY {
                    thread::sleep(constants::DELAY - delta);
                }

                let mut proxies = self.fetch(&mut request)?;
                self.proxies.append(&mut proxies);

                // Update the request time
                *last_fetched = Instant::now();
            }

            Ok(self.proxies.split_off(self.proxies.len() - amount))
        }
    }

    fn request_builder(&self) -> ureq::Request {
        let params = serde_url_params::to_string(&self.opts).expect(&format!(
            "Failed to serialize url, please raise an issue to address this: {}",
            constants::REPO_URI
        ));
        ureq::get(constants::API_URI).query_str(&params).build()
    }

    #[cfg(not(test))]
    fn fetch(&self, request: &mut ureq::Request) -> Result<Vec<Proxy>, ApiError> {
        let resp = request.call();

        if resp.ok() {
            let resp_str = resp
                .into_string()
                .expect("Failed converting response to string");

            match serde_json::from_str::<crate::types::Response>(&resp_str) {
                Ok(response) => Ok(response.data),
                Err(_) => Err(ApiError::from(resp_str)),
            }
        } else {
            Err(ApiError::from(resp))
        }
    }

    // TODO: is there a better way to mock the api response? It would be nice to test that errors
    //       get interpreted right too
    #[cfg(test)]
    fn fetch(&self, _not_needed: &mut ureq::Request) -> Result<Vec<Proxy>, ApiError> {
        Ok(std::iter::repeat(Proxy {
            ip: std::net::Ipv4Addr::new(1, 2, 3, 4),
            port: 4321,
            country: String::from("CA"),
            last_checked: chrono::naive::NaiveDate::from_ymd(2020, 1, 1).and_hms(1, 1, 1),
            level: crate::types::Level::Anonymous,
            protocol: crate::types::Protocol::Http,
            time_to_connect: 21,
            supports: crate::types::Supports::default(),
        })
        .take(*self.opts.limit() as usize)
        .collect())
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
        Self {
            // Start far enough back to avoid delay
            last_fetched: Arc::new(Mutex::new(Instant::now() - constants::DELAY)),
        }
    }

    pub fn spawn_fetcher(&self, opts: Opts) -> Fetcher {
        Fetcher::new(self.last_fetched.clone(), opts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // XXX: need to handle clearing the env var key once that's added in
    const FREE_LIMIT: usize = 5;
    const PREMIUM_LIMIT: usize = 20;

    mod functionality {
        use super::*;

        #[test]
        fn api_key() {
            let session = Session::new();

            let mut fetcher =
                session.spawn_fetcher(Opts::builder().api_key("<key>").try_build().unwrap());

            let single = fetcher.try_get(1).unwrap();
            let the_rest = fetcher.drain();

            // Total returned from one fetch using an api key should be 20
            assert_eq!(PREMIUM_LIMIT, single.len() + the_rest.len());
        }

        #[test]
        fn methods() {
            let session = Session::new();

            let mut fetcher = session.spawn_fetcher(Opts::default());

            let single = fetcher.try_get(1).unwrap();
            assert_eq!(single.len(), 1);

            let triple = fetcher.try_get(3).unwrap();
            assert_eq!(triple.len(), 3);

            let the_rest = fetcher.drain();
            assert_eq!(the_rest.len(), FREE_LIMIT - single.len() - triple.len());
        }
    }

    mod delays {
        use super::*;

        use std::time::Duration;

        const TEN_MILLISEC: Duration = Duration::from_millis(10);

        // Helper function for ensuring runtime of a `FnOnce`
        fn time_it<F, T>(f: F, (expected, delta): (Duration, Duration)) -> T
        where
            F: FnOnce() -> T,
        {
            let start = Instant::now();

            let result = f();

            let end = Instant::now();
            let elapsed = end.duration_since(start);
            assert!(elapsed >= (expected - delta) && elapsed <= (expected + delta));

            result
        }

        #[test]
        fn single_fetcher() {
            let mut fetcher = time_it(
                || {
                    let session = Session::new();
                    let mut fetcher = session.spawn_fetcher(Opts::default());

                    // 5 proxies is returned with no API key so 6 will force 2 calls
                    let _ = fetcher.try_get(FREE_LIMIT + 1);

                    fetcher
                },
                // delay +/- 10ms
                (constants::DELAY, TEN_MILLISEC),
            );

            // Since there are still proxies in the internal list there should be no delay here
            time_it(
                || {
                    let _ = fetcher.try_get(FREE_LIMIT - 1);
                },
                // 10ms +/- 10ms
                (TEN_MILLISEC, TEN_MILLISEC),
            );
        }

        #[test]
        fn multiple_fetchers() {
            // Multiple fetchers should still have the delays coordinated
            time_it(
                || {
                    let session = Session::new();
                    let mut fetcher1 = session.spawn_fetcher(Opts::default());
                    let mut fetcher2 = session.spawn_fetcher(Opts::default());

                    let _ = fetcher1.try_get(1);
                    let _ = fetcher2.try_get(1);
                },
                // delay +/- 10ms
                (constants::DELAY, TEN_MILLISEC),
            );
        }

        #[test]
        fn mutliple_threads() {
            // Multiple fetchers should still have the delays coordinated across threads
            time_it(
                || {
                    let session = Session::new();
                    let mut fetcher1 = session.spawn_fetcher(Opts::default());
                    let mut fetcher2 = session.spawn_fetcher(Opts::default());

                    let handle1 = thread::spawn(move || {
                        let _ = fetcher1.try_get(1);
                    });
                    let handle2 = thread::spawn(move || {
                        let _ = fetcher2.try_get(1);
                    });

                    handle1.join().expect("Failed to join thread");
                    handle2.join().expect("Failed to join thread");
                },
                // delay +/- 10ms
                (constants::DELAY, TEN_MILLISEC),
            );
        }
    }
}
