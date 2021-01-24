use crate::errors::ParamError;

use std::{convert::TryFrom, time::Duration};

use iso_country::Country;
use serde::{Deserialize, Serialize};
use ureq::Response;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
struct BoundedVal<T: PartialEq> {
    #[serde(flatten)]
    pub val: T,
    #[serde(skip_serializing)]
    pub bounds: (T, T),
}

impl<T: PartialEq> BoundedVal<T> {
    pub fn new(val: T, bounds: (T, T)) -> Self {
        Self { val, bounds }
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
            // Bounds go from one minute to an hour
            const BOUNDS: ($type, $type) = $bounds;

            fn new(val: $type) -> Self {
                Self {
                    inner: BoundedVal::new(val, Self::BOUNDS),
                }
            }

            pub fn value(&self) -> $type {
                self.inner.val
            }
        }

        impl TryFrom<$type> for $name {
            type Error = ParamError;

            fn try_from(duration: $type) -> Result<Self, Self::Error> {
                if duration >= Self::BOUNDS.0 && duration <= Self::BOUNDS.1 {
                    Ok(Self::new(duration))
                } else {
                    Err(Self::Error::OutOfBounds {
                        bounds: Self::BOUNDS,
                        value: duration,
                    })
                }
            }
        }
    };
}

const LAST_CHECKED_BOUNDS: (Duration, Duration) =
    (Duration::from_secs(60), Duration::from_secs(60 * 60));
const TIME_TO_CONNECT_BOUNDS: (Duration, Duration) =
    (Duration::from_secs(1), Duration::from_secs(60));
bounded_val! {LastChecked, Duration, LAST_CHECKED_BOUNDS}
bounded_val! {TimeToConnect, Duration, TIME_TO_CONNECT_BOUNDS}

pub struct NaiveResponse {
    pub status: u16,
    pub text: String,
}

impl NaiveResponse {
    pub fn new(status: u16, text: String) -> Self {
        Self { status, text }
    }

    pub fn ok(&self) -> bool {
        self.status >= 200 && self.status < 300
    }
}

impl From<Response> for NaiveResponse {
    fn from(resp: Response) -> Self {
        let status = resp.status();
        let text = resp.into_string().unwrap_or_default();

        Self::new(status, text)
    }
}

// TODO: this could be valid for the whole time now, so could remove the builder for it
#[derive(Serialize, Clone, Debug, PartialEq)]
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
        // TODO: make sure this is documented. Mention that unknows are automatically filtered out
        // if any country is used in the allow or blocklist
        if let Country::Unspecified = country {
            panic!(format!(
                "This library doesn't allow `Unspecified` country in the allow or blocklist"
            ));
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

        let bounds_err = LastChecked::try_from(Duration::from_secs(0)).unwrap_err();
        assert_eq!(
            bounds_err,
            ParamError::out_of_bounds(zero_seconds, LAST_CHECKED_BOUNDS)
        );

        let bounds_err = LastChecked::try_from(Duration::from_secs(60 * 60 + 1)).unwrap_err();
        assert_eq!(
            bounds_err,
            ParamError::out_of_bounds(just_over_hour, LAST_CHECKED_BOUNDS)
        );
    }
}
