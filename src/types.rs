//! [`types`][self] contains auxillary types used by [`Opts`][crate::opts::Opts].
//!
//! This includes NewType wrappers around parameters like [`LastChecked`][LastChecked] and
//! [`TimeToConnect`][TimeToConnect] along with `enum`s for parameters with a limited number of
//! options like [`Countries`][Countries], [`Level`][Level], and [`Protocol`][Protocol].

use crate::errors::ParamError;

use std::{convert::TryFrom, fmt, time::Duration};

use iso_country::Country;
use serde::{Deserialize, Serialize};
use ureq::Response;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
struct BoundedVal<T>
where
    T: fmt::Debug + PartialEq + PartialOrd,
{
    #[serde(flatten)]
    pub val: T,
}

impl<T> BoundedVal<T>
where
    T: fmt::Debug + PartialEq + PartialOrd,
{
    pub fn new(val: T, bounds: (T, T)) -> Result<Self, ParamError<T>> {
        debug_assert!(bounds.0 <= bounds.1);

        if val >= bounds.0 && val <= bounds.1 {
            Ok(Self { val })
        } else {
            Err(ParamError::out_of_bounds(val, bounds))
        }
    }
}

macro_rules! bounded_val {
    ($name:ident, $type:ty, $bounds:ident) => {
        #[derive(Clone, Debug, PartialEq, Serialize)]
        pub struct $name {
            #[serde(flatten)]
            inner: BoundedVal<$type>,
        }

        impl $name {
            pub const BOUNDS: ($type, $type) = $bounds;

            pub fn new(val: $type) -> Result<Self, ParamError<$type>> {
                let inner = BoundedVal::new(val, Self::BOUNDS)?;
                Ok(Self { inner })
            }

            pub fn value(&self) -> $type {
                self.inner.val
            }
        }

        impl TryFrom<$type> for $name {
            type Error = ParamError<$type>;

            fn try_from(val: $type) -> Result<Self, Self::Error> {
                Self::new(val)
            }
        }
    };
}

// One minute to an hour
const LAST_CHECKED_BOUNDS: (Duration, Duration) =
    (Duration::from_secs(60), Duration::from_secs(60 * 60));
// One second to a minute
const TIME_TO_CONNECT_BOUNDS: (Duration, Duration) =
    (Duration::from_secs(1), Duration::from_secs(60));
bounded_val! {LastChecked, Duration, LAST_CHECKED_BOUNDS}
bounded_val! {TimeToConnect, Duration, TIME_TO_CONNECT_BOUNDS}

pub(crate) struct NaiveResponse {
    pub(crate) status: u16,
    pub(crate) text: String,
}

impl NaiveResponse {
    pub fn new(status: u16, text: String) -> Self {
        Self { status, text }
    }

    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

impl From<Response> for NaiveResponse {
    fn from(resp: Response) -> Self {
        let status = resp.status();
        let text = resp.into_string().unwrap_or_default();

        Self::new(status, text)
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub enum Countries {
    #[serde(rename = "country")]
    AllowList(String),
    #[serde(rename = "not_country")]
    BlockList(String),
}

impl Countries {
    pub fn allow() -> Self {
        Self::AllowList(String::new())
    }

    pub fn block() -> Self {
        Self::BlockList(String::new())
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::AllowList(countries) => countries.is_empty(),
            Self::BlockList(countries) => countries.is_empty(),
        }
    }

    pub fn countries(mut self, countries: &[Country]) -> Self {
        for country in countries {
            self = self.country(*country);
        }

        self
    }

    pub fn country(self, country: Country) -> Self {
        // TODO: make sure this is documented. Mention that unknowns are automatically filtered out
        // if any country is used in the allow or blocklist
        if let Country::Unspecified = country {
            // TODO: this could be returned as a `ParamError` instead of panicking
            panic!("This library doesn't allow `Unspecified` country in the allow or blocklist");
        }

        let push_country = |list: String, new_tag: Country| {
            let new_tag = new_tag.to_string();
            if list.is_empty() {
                new_tag
            } else {
                [list, new_tag].join(",")
            }
        };

        match self {
            Self::AllowList(list) => Self::AllowList(push_country(list, country)),
            Self::BlockList(list) => Self::BlockList(push_country(list, country)),
        }
    }
}

impl Default for Countries {
    fn default() -> Self {
        // Default is to block none
        Countries::block()
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Anonymous,
    Elite,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Http,
    Socks4,
    Socks5,
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    mod bounded_vals {
        use super::*;

        #[test]
        fn bounds_checking() {
            let zero_seconds = Duration::from_secs(0);
            let just_over_minute = Duration::from_secs(61);
            let just_over_hour = Duration::from_secs(60 * 60 + 1);

            let bounds_err = TimeToConnect::try_from(zero_seconds).unwrap_err();
            assert_eq!(
                bounds_err,
                ParamError::out_of_bounds(zero_seconds, TIME_TO_CONNECT_BOUNDS)
            );

            let bounds_err = TimeToConnect::try_from(just_over_minute).unwrap_err();
            assert_eq!(
                bounds_err,
                ParamError::out_of_bounds(just_over_minute, TIME_TO_CONNECT_BOUNDS)
            );

            let bounds_err = LastChecked::try_from(zero_seconds).unwrap_err();
            assert_eq!(
                bounds_err,
                ParamError::out_of_bounds(zero_seconds, LAST_CHECKED_BOUNDS)
            );

            let bounds_err = LastChecked::try_from(just_over_hour).unwrap_err();
            assert_eq!(
                bounds_err,
                ParamError::out_of_bounds(just_over_hour, LAST_CHECKED_BOUNDS)
            );
        }

        #[test]
        fn it_works() {
            let half_minute = Duration::from_secs(30);
            let half_hour = Duration::from_secs(30 * 60);

            let valid_time_to_connect = TimeToConnect::try_from(half_minute).unwrap();
            assert_eq!(valid_time_to_connect.value(), half_minute);

            let valid_last_checked = LastChecked::try_from(half_hour).unwrap();
            assert_eq!(valid_last_checked.value(), half_hour);
        }
    }
}
