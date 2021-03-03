use std::{
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use crate::{
    constants,
    errors::ApiError,
    opts::Opts,
    proxy::{proxies_from_json, Proxy},
    types::NaiveResponse,
};

lazy_static! {
    static ref LAST_FETCHED: Arc<Mutex<Instant>> =
        Arc::new(Mutex::new(Instant::now() - constants::DELAY));
}

#[derive(Clone, Debug)]
pub struct Fetcher {
    opts: Opts,
    proxies: Vec<Proxy>,
}

impl Fetcher {
    pub fn new(opts: Opts) -> Self {
        Self {
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

            if self.opts.is_premium() {
                // Don't need to mess with any delays if we're using an api key. (This information
                // was based off emailing the dev. I never got an api key to test)
                while self.proxies.len() < amount {
                    let mut proxies = self.fetch(&mut request)?;
                    self.proxies.append(&mut proxies);
                }
            } else {
                // If we don't have an api key then we need to coordinate delays to ensure we don't
                // do more than one request per `constants::DELAY`
                let mut last_fetched = match LAST_FETCHED.lock() {
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
            }

            Ok(self.proxies.split_off(self.proxies.len() - amount))
        }
    }

    fn request_builder(&self) -> ureq::Request {
        let params = serde_urlencoded::to_string(&self.opts).unwrap_or_else(|_| {
            panic!(
                "Failed to serialize url, please raise an issue to address this: {}",
                constants::REPO_URI
            )
        });
        ureq::get(constants::API_URI).query_str(&params).build()
    }

    fn fetch(&self, request: &mut ureq::Request) -> Result<Vec<Proxy>, ApiError> {
        if cfg!(not(test)) {
            let resp = request.call();
            let naive_resp = NaiveResponse::from(resp);

            if naive_resp.ok() {
                proxies_from_json(&naive_resp.text).map_err(|_| ApiError::from(naive_resp))
            } else {
                Err(ApiError::from(naive_resp))
            }
        } else {
            use chrono::naive::NaiveDate;
            use iso_country::Country;

            use crate::{
                proxy::Supports,
                types::{Level, Protocol},
            };

            use std::{
                iter,
                net::{Ipv4Addr, SocketAddrV4},
                time::Duration,
            };

            // TODO: is there a better way to mock the api response? It would be nice to test that
            // errors get interpreted right too. And if we could panic then we can test that the
            // mutex getting poisoned works right
            Ok(iter::repeat(Proxy {
                socket: SocketAddrV4::new(Ipv4Addr::new(1, 2, 3, 4), 4321),
                country: Country::CA,
                last_checked: NaiveDate::from_ymd(2020, 1, 1).and_hms(1, 1, 1),
                level: Level::Anonymous,
                protocol: Protocol::Http,
                time_to_connect: Duration::from_secs(21),
                supports: Supports::default(),
            })
            .take(self.opts.limit as usize)
            .collect())
        }
    }

    pub fn drain(self) -> Vec<Proxy> {
        self.proxies
    }
}

impl Default for Fetcher {
    fn default() -> Self {
        Self::new(Opts::default())
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;

    // XXX: need to handle clearing the env var key once that's added in
    const FREE_LIMIT: usize = 5;
    const PREMIUM_LIMIT: usize = 20;

    mod functionality {
        use iso_country::Country;

        use super::*;
        use crate::types::{Countries, Level};

        #[test]
        #[serial]
        fn api_key() {
            let mut fetcher = Fetcher::new(Opts::builder().api_key("<key>".to_string()).build());

            let single = fetcher.try_get(1).unwrap();
            let triple = fetcher.try_get(3).unwrap();
            let the_rest = fetcher.drain();

            assert_eq!(single.len(), 1);
            assert_eq!(triple.len(), 3);
            assert_eq!(PREMIUM_LIMIT, single.len() + triple.len() + the_rest.len());
        }

        #[test]
        #[serial]
        fn keyless() {
            let mut fetcher = Fetcher::default();

            let single = fetcher.try_get(1).unwrap();
            let triple = fetcher.try_get(3).unwrap();
            let the_rest = fetcher.drain();

            assert_eq!(single.len(), 1);
            assert_eq!(triple.len(), 3);
            assert_eq!(FREE_LIMIT, single.len() + triple.len() + the_rest.len());
        }

        #[test]
        #[serial]
        fn multiple_requests() {
            // Multiple requests can be done with a single method call
            for i in 0..=2 * FREE_LIMIT {
                let mut fetcher = Fetcher::default();
                let proxies = fetcher.try_get(i).unwrap();
                assert_eq!(proxies.len(), i);
            }
        }

        #[test]
        #[serial]
        fn multiple_fetchers() {
            // Each fetcher should be independent
            let mut default = Fetcher::default();
            let mut premium = Fetcher::new(Opts::builder().api_key("<key>".to_string()).build());
            let mut custom = Fetcher::new(
                Opts::builder()
                    .level(Level::Elite)
                    .cookies(true)
                    .countries(Countries::allow().country(Country::CA))
                    .build(),
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
        use std::time::Duration;

        use super::*;

        const TEN_MILLISEC: Duration = Duration::from_millis(10);

        // TODO: do this with a fixture
        fn reset_last_fetched() {
            let mut last_fetched = LAST_FETCHED.lock().unwrap();
            *last_fetched = Instant::now() - constants::DELAY;
        }

        // Helper function for ensuring runtime of a `FnOnce`
        fn time_it<F, T>(f: F, (expected, delta): (Duration, Duration)) -> T
        where
            F: FnOnce() -> T,
        {
            let start = Instant::now();

            let result = f();

            let end = Instant::now();
            let elapsed = end.duration_since(start);
            eprintln!("Elapsed time: {:?}", elapsed);
            eprintln!("Expected time: {:?} +/- {:?}", expected, delta);
            assert!(elapsed >= (expected - delta), "Too fast");
            assert!(elapsed <= (expected + delta), "Too slow");

            result
        }

        #[test]
        #[serial]
        fn single_fetcher() {
            // Requesting the first `FREE_LIMIT` is done in one call
            let mut fetcher = time_it(
                || {
                    reset_last_fetched();
                    let mut fetcher = Fetcher::default();
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
        #[serial]
        fn multiple_delays() {
            // Fulfilling 4 full requests should delay thrice
            time_it(
                || {
                    reset_last_fetched();

                    let mut keyless1 = Fetcher::default();
                    let mut keyless2 = Fetcher::default();
                    // TODO: this option is used several times. Reuse somehow?
                    let mut premium =
                        Fetcher::new(Opts::builder().api_key("<key>".to_string()).build());

                    let _ = keyless1.try_get(2 * FREE_LIMIT);
                    // Even while the keyless ones would be delayed, the premium is not
                    let _ = premium.try_get(2 * PREMIUM_LIMIT);
                    let _ = keyless2.try_get(2 * FREE_LIMIT);
                },
                // 3 * delay +/- 10ms
                (3 * constants::DELAY, TEN_MILLISEC),
            );
        }

        #[test]
        #[serial]
        fn multiple_fetchers() {
            // Multiple fetchers should still have the delays coordinated
            let (mut fetcher1, mut fetcher2) = time_it(
                || {
                    reset_last_fetched();

                    let mut fetcher1 = Fetcher::default();
                    let mut fetcher2 = Fetcher::default();

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
        #[serial]
        fn multiple_threads() {
            // Multiple fetchers should still have the delays coordinated across threads
            time_it(
                || {
                    reset_last_fetched();

                    let mut fetcher1 = Fetcher::default();
                    let mut fetcher2 = Fetcher::default();

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
