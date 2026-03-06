use std::future::Future;

use axum::{
    extract::FromRequestParts,
    http::StatusCode,
    http::request::Parts,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::jwt;

#[derive(Clone, Copy)]
pub struct AuthUser(pub Uuid);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        async move {
            let Some(h) = auth_header else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };

            let token = h
                .strip_prefix("Bearer ")
                .ok_or_else(|| StatusCode::UNAUTHORIZED.into_response())?;

            let secret = std::env::var("JWT_SECRET")
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

            let claims = jwt::verify(token, &secret)
                .map_err(|_| StatusCode::UNAUTHORIZED.into_response())?;

            if claims.token_type != jwt::TokenType::Access {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            }

            let user_id = Uuid::parse_str(&claims.sub)
                .map_err(|_| StatusCode::UNAUTHORIZED.into_response())?;

            Ok(AuthUser(user_id))
        }
    }
}
