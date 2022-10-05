//! Represents all the errors epressed by the API and library.
//!
//! These are divided into an `APIError` which represents all errors returned by pubproxy.com and
//! `ParamError` which expresses any parameters that were invalid and can't be caught at compile time.

use std::fmt;

use crate::{constants, types::NaiveResponse};

use thiserror::Error;

/// Represents an error with a parameter type.
///
/// Currently the only types that can error are [`LastChecked`][crate::types::LastChecked] and
/// [`TimeToConnect`][crate::types::TimeToConnect] since they are both bounded values which will
/// error if the provided value is out of bounds.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParamError<T: PartialEq + fmt::Debug> {
    #[error("'{value:?}' is outside bounds: {bounds:?}")]
    OutOfBounds { bounds: (T, T), value: T },
}

impl<T: PartialEq + fmt::Debug> ParamError<T> {
    pub fn out_of_bounds(value: T, bounds: (T, T)) -> Self {
        Self::OutOfBounds { value, bounds }
    }
}

/// Represents all possible errors returned by the API.
///
/// Some variants should be entirely prevented by this library like `Client`, while others are
/// expected from heavy use like `RateLimit` or from being too strict on parameters like `NoProxy`.
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Client Error ({status}): {text}\n This should be prevented, please raise an issue")]
    Client { status: u16, text: String },

    #[error("Internal Server Error ({status}): {text}")]
    Server { status: u16, text: String },

    #[error("Invalid API key, make sure your key is valid")]
    ApiKey,

    // TODO: mention fetchers from multiple sessions
    #[error(
        "You have exceeded the rate limit. This could be due to multiple programs using the API. \
 If this is not the case then sorry but the API hates you, consider raising an issue."
    )]
    RateLimit,

    #[error("You have exhausted the daily limit of proxies.")]
    DailyLimit,

    #[error("No matching proxies, consider broadening the parameters used")]
    NoProxy,

    #[error("The API returned an unexpected message. Consider raising an issue with the library")]
    Unknown,
}

impl From<NaiveResponse> for ApiError {
    fn from(naive_resp: NaiveResponse) -> Self {
        let NaiveResponse { status, text } = naive_resp;

        // Some known errors get returned with varied `status` codes so match on response text first
        // then add context to unknown status codes
        match Self::from(text.clone()) {
            Self::Unknown => {
                if (400..500).contains(&status) {
                    Self::Client { status, text }
                } else if (500..600).contains(&status) {
                    Self::Server { status, text }
                } else {
                    unreachable!(
                        "Tried creating ApiError from valid response ({}). Please raise an issue \
                         at {}.",
                        status,
                        constants::REPO_URI
                    );
                }
            }
            err => err,
        }
    }
}

const INVALID_API_KEY: &str =
    "Invalid API. Get your API to make unlimited requests at http://pubproxy.com/#premium";
const RATE_LIMIT: &str = "We have to temporarily stop you. You're requesting proxies a little too \
                          fast (2+ requests per second). Get your API to remove this limit at
                          http://pubproxy.com/#premium";
const DAILY_LIMIT: &str = "You reached the maximum 50 requests for today. Get your API to make \
                           unlimited requests at http://pubproxy.com/#premium";
const NO_PROXY: &str = "No proxy";

impl From<String> for ApiError {
    fn from(s: String) -> Self {
        match s.as_str() {
            INVALID_API_KEY => Self::ApiKey,
            RATE_LIMIT => Self::RateLimit,
            DAILY_LIMIT => Self::DailyLimit,
            NO_PROXY => Self::NoProxy,
            _ => Self::Unknown,
        }
    }
}
