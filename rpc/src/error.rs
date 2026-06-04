use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use common::error::AppError;
use thiserror::Error;

use crate::model::ErrorResponse;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("internal server error")]
    Internal,
}

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        let err = value.to_string();
        tracing::error!("{}", err);
        match value {
            AppError::BlockNotFound => Self::NotFound(err),
            AppError::TxNotFound => Self::NotFound(err),
            _ => Self::Internal,
        }
    }
}

#[cfg(feature = "server")]
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let error: ErrorResponse = self.into();
        let body = Json::from(error);
        (status, body).into_response()
    }
}
