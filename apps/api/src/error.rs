use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Application errors mapped to HTTP responses at the handler boundary.
#[allow(dead_code)] // Variants reserved for routes not yet using `AppError`
#[derive(Debug)]
pub enum AppError {
    NotFound,
    Conflict,
    /// Plain-text body when `Some`, empty body when `None`
    BadRequest(Option<&'static str>),
    Forbidden,
    Internal,
    ServiceUnavailable,
    Unauthorized,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BadRequest(Some(msg)) => (StatusCode::BAD_REQUEST, msg).into_response(),
            AppError::BadRequest(None) => StatusCode::BAD_REQUEST.into_response(),
            AppError::NotFound => StatusCode::NOT_FOUND.into_response(),
            AppError::Conflict => StatusCode::CONFLICT.into_response(),
            AppError::Forbidden => StatusCode::FORBIDDEN.into_response(),
            AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            AppError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE.into_response(),
            AppError::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
        }
    }
}
