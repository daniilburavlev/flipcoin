use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl From<ApiError> for ErrorResponse {
    fn from(value: ApiError) -> Self {
        Self {
            error: value.to_string(),
        }
    }
}
