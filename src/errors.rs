use std::time::Duration;

use crate::{constants, types::NaiveResponse};

use thiserror::Error;

// TODO: look into verifying the country codes that are passed in
#[derive(Error, Debug)]
pub enum ParamError {
    #[error("'{value:?}' is outside param '{param:?}' bounds: {bounds:?}")]
    OutOfBounds {
        param: String,
        bounds: (Duration, Duration),
        value: Duration,
    },
}

#[derive(Error, Debug)]
pub enum ApiError {
    // TODO: do all of these really need to have error in their name too? Check other libs
    #[error("Client Error ({status}): {text}\n This should be prevented, please raise an issue")]
    ClientError { status: u16, text: String },

    #[error("Internal Server Error ({status}): {text}")]
    ServerError { status: u16, text: String },

    #[error("Invalid API key, make sure your key is valid")]
    ApiKeyError,

    // TODO: mention fetchers from multiple sessions
    #[error(
        "You have exceeded the rate limit. This could be due to multiple programs using the API. \
 If this is not the case then sorry but the API hates you, consider raising an issue."
    )]
    RateLimitError,

    #[error("You have exhausted the daily limit of proxies.")]
    DailyLimitError,

    #[error("No matching proxies, consider broadening the parameters used")]
    NoProxyError,

    #[error("The API returned an unexpected message. Consider raising an issue with the library")]
    UnknownError,
}

impl From<NaiveResponse> for ApiError {
    fn from(naive_resp: NaiveResponse) -> Self {
        let NaiveResponse { status, text } = naive_resp;

        // Some known errors get returned with varied `status` codes so match on response text first
        // then add context to unknown status codes
        match Self::from(text.clone()) {
            Self::UnknownError => {
                if status >= 400 && status < 500 {
                    Self::ClientError { status, text }
                } else if status >= 500 && status < 600 {
                    Self::ServerError { status, text }
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
            INVALID_API_KEY => Self::ApiKeyError,
            RATE_LIMIT => Self::RateLimitError,
            DAILY_LIMIT => Self::DailyLimitError,
            NO_PROXY => Self::NoProxyError,
            _ => Self::UnknownError,
        }
    }
}
