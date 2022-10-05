//! [`Opts`][Opts] provide the ability to filter the returned proxies.

use std::num::NonZeroU16;

use crate::types::{Countries, LastChecked, Level, Protocol, TimeToConnect};

use serde::Serialize;
use serde_repr::Serialize_repr;

// TODO: allow for multiple things being specified on the different things that accept it?
/// A builder for setting up [`Opts`][Opts].
///
/// Constructed with `Opts::builder()`. By default any field that isn't specified will just return
/// any possible value so these options just constrain the returned results.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OptsBuilder {
    api_key: Option<String>,
    level: Option<Level>,
    protocol: Option<Protocol>,
    countries: Option<Countries>,
    last_checked: Option<LastChecked>,
    port: Option<NonZeroU16>,
    time_to_connect: Option<TimeToConnect>,
    cookies: Option<bool>,
    connects_to_google: Option<bool>,
    https: Option<bool>,
    post: Option<bool>,
    referer: Option<bool>,
    forwards_user_agent: Option<bool>,
}

impl OptsBuilder {
    /// Passes an API key to the API. This removes both the rate limit and daily limit on the API.
    pub fn api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// The anonymity level of proxies returned by the API where the proxies are either Anonymous or
    /// Elite (Transparent isn't provided).
    pub fn level(mut self, level: Level) -> Self {
        self.level = Some(level);
        self
    }

    /// The protocol supported by the proxies. This can either be HTTP, SOCKS4, or SOCKS5.
    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = Some(protocol);
        self
    }

    /// Either a block or allowlist of countries for where the proxies can be located.
    pub fn countries(mut self, countries: Countries) -> Self {
        self.countries = Some(countries);
        self
    }

    /// Time when the proxies were last checked. Resolution down to minutes with a valid range of
    /// 1 to 1,000 minutes.
    pub fn last_checked(mut self, last_checked: LastChecked) -> Self {
        self.last_checked = Some(last_checked);
        self
    }

    /// Specifies the port that the proxy exposes.
    pub fn port(mut self, port: NonZeroU16) -> Self {
        self.port = Some(port);
        self
    }

    /// Filters based on how long it took to connect to the proxy when testing. Will return values
    /// at or below the specified time with a resolution down to seconds with a valid range of 1 to
    /// 60 seconds.
    pub fn time_to_connect(mut self, time_to_connect: TimeToConnect) -> Self {
        self.time_to_connect = Some(time_to_connect);
        self
    }

    /// If the proxy supports cookies or not.
    pub fn cookies(mut self, cookies: bool) -> Self {
        self.cookies = Some(cookies);
        self
    }

    /// If the proxy was able to connect to google or not.
    pub fn connects_to_google(mut self, connects_to_google: bool) -> Self {
        self.connects_to_google = Some(connects_to_google);
        self
    }

    /// If the proxy supports HTTPS requests or not.
    pub fn https(mut self, https: bool) -> Self {
        self.https = Some(https);
        self
    }

    /// If the proxy supports POST requests or not.
    pub fn post(mut self, post: bool) -> Self {
        self.post = Some(post);
        self
    }

    /// If the proxy supports referer requests or not.
    pub fn referer(mut self, referer: bool) -> Self {
        self.referer = Some(referer);
        self
    }

    /// If the proxy supports forwarding your user agent.
    pub fn forwards_user_agent(mut self, forwards_user_agent: bool) -> Self {
        self.forwards_user_agent = Some(forwards_user_agent);
        self
    }

    /// Constructs the `OptsBuilder` into the corresponding [`Opts`][Opts] value.
    pub fn build(self) -> Opts {
        Opts::from(self)
    }
}

/// Internal
#[derive(Serialize_repr, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Limit {
    Free = 5,
    Premium = 20,
}

impl Default for Limit {
    fn default() -> Self {
        Self::Free
    }
}

/// Internal
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Format {
    // Techically txt is also allowed, but this library only uses json
    Json,
}

impl Default for Format {
    fn default() -> Self {
        Self::Json
    }
}

/// A set of options to constrain the returned proxies.
///
/// `Opts` represents all the filtering options that are passed on to the API by the corresponding
/// [`Fetcher`][crate::fetcher::Fetcher]. By default no values are filtered and any kind of proxy
/// can be returned so this list of options only serves to restrict the proxies returned. The
/// typical way to construct `Opts` is with [`OptsBuilder`][OptsBuilder] with the entrypoint of
/// `Opts::builder()`.
///
/// ```
/// use iso_country::Country;
/// use lead_oxide::{
///     opts::Opts,
///     types::{Countries, LastChecked, Level, Protocol, TimeToConnect}
/// };
/// use std::{convert::TryFrom, num::NonZeroU16, time::Duration};
///
/// let basic_opts = Opts::builder()
///     .post(true)
///     .cookies(true)
///     .build();
/// let kitchen_sink = Opts::builder()
///     .api_key("<key>".to_string())
///     .level(Level::Elite)
///     .protocol(Protocol::Socks4)
///     .countries(Countries::block().countries(&[Country::CH, Country::ES]))
///     .last_checked(LastChecked::try_from(Duration::from_secs(60 * 10)).unwrap())
///     .time_to_connect(TimeToConnect::try_from(Duration::from_secs(10)).unwrap())
///     .port(NonZeroU16::new(8080).unwrap())
///     .cookies(true)
///     .connects_to_google(false)
///     .https(true)
///     .post(false)
///     .referer(true)
///     .forwards_user_agent(false)
///     .build();
/// ```
#[derive(Serialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Opts {
    #[serde(rename = "api")]
    api_key: Option<String>,
    level: Option<Level>,
    #[serde(rename = "type")]
    protocol: Option<Protocol>,
    // An empty country list is essentially `None`
    #[serde(flatten, skip_serializing_if = "Countries::is_empty")]
    countries: Countries,
    #[serde(rename = "last_check")]
    last_checked: Option<u64>,
    // Note: using a port of 0 will return any port from the api :silly:
    port: Option<NonZeroU16>,
    #[serde(rename = "speed")]
    time_to_connect: Option<u64>,
    cookies: Option<bool>,
    #[serde(rename = "google")]
    connects_to_google: Option<bool>,
    https: Option<bool>,
    post: Option<bool>,
    referer: Option<bool>,
    #[serde(rename = "user_agent")]
    forwards_user_agent: Option<bool>,
    pub(crate) limit: Limit,
    format: Format,
}

impl Opts {
    /// Constructs an [`OptsBuilder`][OptsBuilder]
    pub fn builder() -> OptsBuilder {
        OptsBuilder::default()
    }

    /// Internal
    pub(crate) fn is_premium(&self) -> bool {
        self.api_key.is_some()
    }
}

impl From<OptsBuilder> for Opts {
    fn from(builder: OptsBuilder) -> Self {
        Self {
            limit: match builder.api_key {
                Some(_) => Limit::Premium,
                None => Limit::Free,
            },
            api_key: builder.api_key,
            level: builder.level,
            protocol: builder.protocol,
            countries: builder.countries.unwrap_or_default(),
            last_checked: builder
                .last_checked
                .map(|last_checked| last_checked.value().as_secs() / 60),
            port: builder.port,
            time_to_connect: builder
                .time_to_connect
                .map(|time_to_connect| time_to_connect.value().as_secs()),
            cookies: builder.cookies,
            connects_to_google: builder.connects_to_google,
            https: builder.https,
            post: builder.post,
            referer: builder.referer,
            forwards_user_agent: builder.forwards_user_agent,
            format: Format::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{convert::TryFrom, time::Duration};

    use iso_country::Country;

    #[test]
    fn url_serialization() -> Result<(), serde_urlencoded::ser::Error> {
        let check_equivalent_params = |opts, expected: &[&str]| {
            // Convert `opts` to a url and sort the values
            let url = serde_urlencoded::to_string(&opts)?;
            let mut params: Vec<_> = url.split('&').map(String::from).collect();
            params.sort();

            // Sort the `expected` values
            let mut expected = expected.to_vec();
            expected.sort_unstable();

            assert_eq!(params, expected);

            Ok(())
        };

        // Base `Opts`
        check_equivalent_params(Opts::default(), &["format=json", "limit=5"])?;
        // Using a key will up the limit
        check_equivalent_params(
            Opts::builder().api_key("<key>".to_string()).build(),
            &["api=%3Ckey%3E", "format=json", "limit=20"],
        )?;
        // Empty countries list won't be included (api seems to work with an empty list, but I don't
        // want to rely on this behavior)
        check_equivalent_params(
            Opts::builder().countries(Countries::default()).build(),
            &["format=json", "limit=5"],
        )?;
        // Kitchen sink
        check_equivalent_params(
            Opts::builder()
                .api_key("<key>".to_string())
                .level(Level::Elite)
                .protocol(Protocol::Socks4)
                .countries(Countries::block().countries(&[Country::CH, Country::ES]))
                .last_checked(LastChecked::try_from(Duration::from_secs(60 * 10)).unwrap())
                .time_to_connect(TimeToConnect::try_from(Duration::from_secs(10)).unwrap())
                .port(NonZeroU16::new(8080).unwrap())
                .cookies(true)
                .connects_to_google(false)
                .https(true)
                .post(false)
                .referer(true)
                .forwards_user_agent(false)
                .build(),
            &[
                // Automatic
                "limit=20",
                "format=json",
                // Key
                "api=%3Ckey%3E",
                // Enums
                "level=elite",
                "type=socks4",
                "not_country=CH%2CES",
                // Durations
                "last_check=10",
                "speed=10",
                // NonZero
                "port=8080",
                // Bools
                "cookies=true",
                "google=false",
                "https=true",
                "post=false",
                "referer=true",
                "user_agent=false",
            ],
        )
    }
}
