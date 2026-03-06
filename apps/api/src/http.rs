use axum::http::{HeaderMap, StatusCode};

pub fn user_id_from_headers(headers: &HeaderMap) -> Result<uuid::Uuid, StatusCode> {
    let v = headers
        .get("x-user-id")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    uuid::Uuid::parse_str(v).map_err(|_| StatusCode::UNAUTHORIZED)
}
