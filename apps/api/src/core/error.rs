use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Application errors mapped to HTTP responses at the handler boundary.
#[derive(Debug)]
pub enum AppError {
    NotFound,
    /// Optional plain-text body
    Conflict(Option<&'static str>),
    /// Plain-text body when `Some`, empty body when `None`
    BadRequest(Option<&'static str>),
    Forbidden,
    Internal,
    /// Optional plain-text body (e.g. upload store message)
    ServiceUnavailable(Option<&'static str>),
    Unauthorized,
    PayloadTooLarge(Option<&'static str>),
    /// HTTP 410 — e.g. invite used or expired
    Gone,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BadRequest(Some(msg)) => (StatusCode::BAD_REQUEST, msg).into_response(),
            AppError::BadRequest(None) => StatusCode::BAD_REQUEST.into_response(),
            AppError::NotFound => StatusCode::NOT_FOUND.into_response(),
            AppError::Conflict(Some(msg)) => (StatusCode::CONFLICT, msg).into_response(),
            AppError::Conflict(None) => StatusCode::CONFLICT.into_response(),
            AppError::Forbidden => StatusCode::FORBIDDEN.into_response(),
            AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            AppError::ServiceUnavailable(Some(msg)) => {
                (StatusCode::SERVICE_UNAVAILABLE, msg).into_response()
            }
            AppError::ServiceUnavailable(None) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
            AppError::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            AppError::PayloadTooLarge(Some(msg)) => {
                (StatusCode::PAYLOAD_TOO_LARGE, msg).into_response()
            }
            AppError::PayloadTooLarge(None) => StatusCode::PAYLOAD_TOO_LARGE.into_response(),
            AppError::Gone => StatusCode::GONE.into_response(),
        }
    }
}
