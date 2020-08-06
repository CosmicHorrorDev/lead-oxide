use std::time::Duration;

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
