# PbO

`lead-oxide` is a wrapper around pubproxy.com's public proxy API.

`lead-oxide` strives to make it impossible to use the API incorrectly while still making regular usage as ergonomic as possible.

## Keyless API Limitations

### Daily Limit

At the time of writing this without an API key the pubproxy API limits users to 5 proxies per request and 50 requests per day. The maximum proxies per request is always used to minimize rate limiting along with getting the most proxies possible within the request limit meaning you should get 250 proxies per day without needing an API key.

### Rate Limiting

Without an API key pubproxy limits users to one request per second so a `Fetcher` will try to ensure that at most only one request per second is done without an API key. This is synchronized between fetchers including across different threads: however, there can still be issues from running multiple programs from the same IP causing rate limiting to occur. The rate-limiting is quite severe (will deny requests for potentially several hours), so it's best to avoid by all means possible.

## Quickstart

```rust
use iso_country::Country;
use lead_oxide::{
    errors::ApiError,
    fetcher::Fetcher,
    opts::Opts,
    types::{Countries, Level, Protocol, TimeToConnect},
};

use std::{convert::TryFrom, time::Duration};

fn main() -> Result<(), ApiError> {
    // Fetcher for SOCKS5 proxies located in the US and Canada that support POST requests
    let mut socks_fetcher = Fetcher::new(
        Opts::builder()
            .protocol(Protocol::Socks5)
            .countries(Countries::allow().countries(&[Country::US, Country::CA]))
            .post(true)
            .build(),
    );

    // Fetcher for Elite HTTPS proxies that connected in 15 seconds or less
    let mut https_fetcher = Fetcher::new(
        Opts::builder()
            .protocol(Protocol::Http)
            .https(true)
            .level(Level::Elite)
            .time_to_connect(TimeToConnect::try_from(Duration::from_secs(15)).unwrap())
            .build(),
    );

    // Get one SOCKS proxy and 10 HTTPS proxies
    let socks_proxy = &socks_fetcher.try_get(1)?[0];
    let https_proxies = https_fetcher.try_get(10)?;

    println!("SOCKS proxy: {:#?}", socks_proxy);
    println!("HTTPS proxies {:#?}", https_proxies);

    Ok(())
}
```
