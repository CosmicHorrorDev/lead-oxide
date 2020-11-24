use std::{
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use crate::{constants, errors::ApiError, opts::Opts, types::Proxy};

#[derive(Clone, Debug)]
pub struct Fetcher {
    last_fetched: Arc<Mutex<Instant>>,
    // TODO: store the url so we don't have to keep rebuilding it? It won't change while the fetcher
    // is running
    opts: Opts,
    proxies: Vec<Proxy>,
}

impl Fetcher {
    #[must_use]
    pub fn oneshot() -> Self {
        Self::oneshot_with(Opts::default())
    }

    #[must_use]
    pub fn oneshot_with(opts: Opts) -> Self {
        Session::new().fetcher_with(opts)
    }

    #[must_use]
    fn new(last_fetched: Arc<Mutex<Instant>>, opts: Opts) -> Self {
        Self {
            last_fetched,
            opts,
            proxies: Vec::new(),
        }
    }

    #[must_use]
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

    #[must_use]
    fn request_builder(&self) -> ureq::Request {
        let params = serde_urlencoded::to_string(&self.opts).expect(&format!(
            "Failed to serialize url, please raise an issue to address this: {}",
            constants::REPO_URI
        ));
        ureq::get(constants::API_URI).query_str(&params).build()
    }

    #[must_use]
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
    //       get interpreted right too. And if we could panic then we can test that the mutex
    //       getting poisoned works right
    #[must_use]
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
        .take(self.opts.limit as usize)
        .collect())
    }

    #[must_use]
    pub fn drain(self) -> Vec<Proxy> {
        self.proxies
    }
}

#[derive(Debug)]
pub struct Session {
    last_fetched: Arc<Mutex<Instant>>,
}

impl Session {
    #[must_use]
    pub fn new() -> Self {
        Self {
            // Start far enough back to avoid delay
            last_fetched: Arc::new(Mutex::new(Instant::now() - constants::DELAY)),
        }
    }

    #[must_use]
    pub fn fetcher(&self) -> Fetcher {
        self.fetcher_with(Opts::default())
    }

    #[must_use]
    pub fn fetcher_with(&self, opts: Opts) -> Fetcher {
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
        use crate::types::{Countries, Level};

        #[test]
        fn api_key() {
            let mut fetcher =
                Fetcher::oneshot_with(Opts::builder().api_key("<key>").try_build().unwrap());

            let single = fetcher.try_get(1).unwrap();
            let triple = fetcher.try_get(3).unwrap();
            let the_rest = fetcher.drain();

            assert_eq!(single.len(), 1);
            assert_eq!(triple.len(), 3);
            assert_eq!(PREMIUM_LIMIT, single.len() + triple.len() + the_rest.len());
        }

        #[test]
        fn keyless() {
            let mut fetcher = Fetcher::oneshot();

            let single = fetcher.try_get(1).unwrap();
            let triple = fetcher.try_get(3).unwrap();
            let the_rest = fetcher.drain();

            assert_eq!(single.len(), 1);
            assert_eq!(triple.len(), 3);
            assert_eq!(FREE_LIMIT, single.len() + triple.len() + the_rest.len());
        }

        #[test]
        fn multiple_requests() {
            // Multiple requests can be done with a single method call
            let mut fetcher = Fetcher::oneshot();
            let proxies = fetcher.try_get(3 * FREE_LIMIT).unwrap();
            assert_eq!(proxies.len(), 3 * FREE_LIMIT);
        }

        #[test]
        fn multiple_fetchers() {
            // Each fetcher should be independent
            let session = Session::new();
            let mut default = session.fetcher();
            let mut premium =
                session.fetcher_with(Opts::builder().api_key("<key>").try_build().unwrap());
            let mut custom = session.fetcher_with(
                Opts::builder()
                    .level(Level::Elite)
                    .cookies(true)
                    .countries(Countries::allow().country("CA"))
                    .try_build()
                    .unwrap(),
            );

            let single = default.try_get(1).unwrap();
            let double = premium.try_get(2).unwrap();
            let triple = custom.try_get(3).unwrap();
            assert_eq!(single.len(), 1);
            assert_eq!(double.len(), 2);
            assert_eq!(triple.len(), 3);
            assert_eq!(default.drain().len(), FREE_LIMIT - single.len());
            assert_eq!(premium.drain().len(), PREMIUM_LIMIT - double.len());
            assert_eq!(custom.drain().len(), FREE_LIMIT - triple.len());
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
            eprintln!("{}", elapsed.as_millis());
            assert!(elapsed >= (expected - delta) && elapsed <= (expected + delta));

            result
        }

        #[test]
        fn single_fetcher() {
            // Requesting the first `FREE_LIMIT` is done in one call
            let mut fetcher = time_it(
                || {
                    let mut fetcher = Fetcher::oneshot();
                    let _ = fetcher.try_get(FREE_LIMIT);
                    fetcher
                },
                // 10ms +/- 10ms
                (TEN_MILLISEC, TEN_MILLISEC),
            );

            // any more will take another call
            let mut fetcher = time_it(
                || {
                    let _ = fetcher.try_get(1);
                    fetcher
                },
                // delay +/- 10ms
                (constants::DELAY, TEN_MILLISEC),
            );

            // and since there are proxies in the internal list we can just use those
            time_it(
                || {
                    let _ = fetcher.try_get(1);
                    assert!(!fetcher.drain().is_empty());
                },
                // 10ms +/- 10ms
                (TEN_MILLISEC, TEN_MILLISEC),
            );
        }

        #[test]
        fn multiple_delays() {
            // Fulfilling 4 full requests should delay thrice
            time_it(
                || {
                    let session = Session::new();
                    let mut keyless = session.fetcher();
                    // TODO: this option is used several times. Reuse somehow?
                    let mut premium =
                        session.fetcher_with(Opts::builder().api_key("<key>").try_build().unwrap());

                    let _ = keyless.try_get(2 * FREE_LIMIT);
                    let _ = premium.try_get(2 * PREMIUM_LIMIT);
                },
                // 3 * delay +/- 10ms
                (3 * constants::DELAY, TEN_MILLISEC),
            );
        }

        #[test]
        fn multiple_fetchers() {
            // Multiple fetchers should still have the delays coordinated
            let (mut fetcher1, mut fetcher2) = time_it(
                || {
                    let session = Session::new();
                    let mut fetcher1 = session.fetcher();
                    let mut fetcher2 = session.fetcher();

                    let _ = fetcher1.try_get(1);
                    let _ = fetcher2.try_get(1);

                    (fetcher1, fetcher2)
                },
                // delay +/- 10ms
                (constants::DELAY, TEN_MILLISEC),
            );

            // And each fetcher should now have an internal list to pull from with no delay
            time_it(
                || {
                    let _ = fetcher1.try_get(1);
                    let _ = fetcher2.try_get(1);
                    assert!(!fetcher1.drain().is_empty());
                    assert!(!fetcher2.drain().is_empty());
                },
                // 10ms +/- 10ms
                (TEN_MILLISEC, TEN_MILLISEC),
            );
        }

        #[test]
        fn mutliple_threads() {
            // Multiple fetchers should still have the delays coordinated across threads
            time_it(
                || {
                    let session = Session::new();
                    let mut fetcher1 = session.fetcher();
                    let mut fetcher2 = session.fetcher();

                    let handle1 = thread::spawn(move || {
                        let _ = fetcher1.try_get(1);
                        assert!(!fetcher1.drain().is_empty());
                    });
                    let handle2 = thread::spawn(move || {
                        let _ = fetcher2.try_get(1);
                        assert!(!fetcher2.drain().is_empty());
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
