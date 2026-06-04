use crate::error::ApiError;

#[cfg(feature = "client")]
pub mod client;
pub mod error;
pub mod model;
#[cfg(feature = "server")]
pub mod server;

pub type ApiResult<T> = Result<T, ApiError>;
