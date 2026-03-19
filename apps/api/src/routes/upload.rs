use axum::{
    body::Body,
    extract::{Multipart, Path as AxumPath, State},
    http::{header, StatusCode},
    response::Response,
};

use crate::{
    auth_user::AuthUser, error::AppError, services::upload as upload_service, state::AppState,
};

/// POST /uploads — multipart form with field "file"
pub async fn upload(
    State(state): State<AppState>,
    AuthUser(_user_id): AuthUser,
    multipart: Multipart,
) -> Result<(StatusCode, axum::Json<upload_service::UploadResponse>), AppError> {
    let store = state.upload_store.clone().ok_or(AppError::ServiceUnavailable(Some(
        "Upload storage is not configured",
    )))?;

    let res = upload_service::upload_multipart(store, multipart).await?;
    Ok((StatusCode::OK, axum::Json(res)))
}

/// GET /uploads/attachments/:filename
pub async fn get_attachment(
    State(state): State<AppState>,
    AxumPath(filename): AxumPath<String>,
) -> Result<Response, AppError> {
    let store = state
        .upload_store
        .clone()
        .ok_or(AppError::NotFound)?;

    let (bytes, content_type) =
        upload_service::fetch_attachment_bytes(store, &filename).await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "public, max-age=31536000")
        .body(Body::from(bytes))
        .unwrap())
}
