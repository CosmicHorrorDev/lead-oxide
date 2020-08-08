use std::time::Duration;

use crate::constants;

use thiserror::Error;
use ureq::Response;

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
    #[error("Client Error ({code}): {resp}\n This should be prevented, please raise an issue")]
    ClientError { code: u16, resp: String },

    #[error("Internal Server Error ({code}): {resp}")]
    ServerError { code: u16, resp: String },

    #[error("Invalid API key, make sure your key is valid")]
    ApiKeyError,

    #[error(
        r"You have exceeded the rate limit. This could be due to multiple programs using the API.
 If this is not the case then sorry but the API hates you, consider raising an issue."
    )]
    RateLimitError,

    #[error("You have exhausted the daily limit of proxies.")]
    DailyLimitError,

    #[error("No matching proxies, consider broadening the parameters used")]
    NoProxyError,
}

impl From<Response> for ApiError {
    fn from(resp: Response) -> Self {
        let status = resp.status();
        let resp_str = resp
            .into_string()
            .expect("Failed converting response to string");

        if status >= 400 && status < 500 {
            Self::ClientError {
                code: status,
                resp: resp_str,
            }
        } else if status >= 500 && status < 600 {
            Self::ServerError {
                code: status,
                resp: resp_str,
            }
        } else {
            panic!(format!(
                "Tried creating ApiError from valid response ({}). Please raise an issue at {}.",
                status,
                constants::REPO_URI
            ));
        }
    }
}

const INVALID_API_KEY: &str =
    "Invalid API. Get your API to make unlimited requests at http://pubproxy.com/#premium";
const RATE_LIMIT: &str = r"We have to temporarily stop you. You're requesting proxies a little too
 fast (2+ requests per second). Get your API to remove this limit at http://pubproxy.com/#premium";
const DAILY_LIMIT: &str = r"You have reached the maximum 50 requests for today. Get your API to make
 unlimited requests at http://pubproxy.com/#premium";
const NO_PROXY: &str = "No proxy";

impl From<String> for ApiError {
    fn from(s: String) -> Self {
        match s.as_str() {
            INVALID_API_KEY => Self::ApiKeyError,
            RATE_LIMIT => Self::RateLimitError,
            DAILY_LIMIT => Self::DailyLimitError,
            NO_PROXY => Self::NoProxyError,
            _ => {
                panic!(format!(
                    "The API returned an unexpected message '{}'. Please raise an issue at {}",
                    s,
                    constants::REPO_URI
                ));
            }
        }
    }
}
