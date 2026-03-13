use axum::{
    body::Body,
    extract::{Multipart, Path as AxumPath, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use object_store::path::Path as ObjectStorePath;
use object_store::ObjectStoreExt;
use serde::Serialize;
use uuid::Uuid;

use crate::{auth_user::AuthUser, state::AppState};

const UPLOAD_PREFIX: &str = "attachments";
const MAX_IMAGE_SIZE: usize = 10 * 1024 * 1024; // 10 MiB
const ALLOWED_IMAGE_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
];

#[derive(Serialize)]
pub struct UploadResponse {
    pub url: String,
}

/// POST /uploads — multipart form with file field "file"
pub async fn upload(
    State(state): State<AppState>,
    AuthUser(_user_id): AuthUser,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let store = match &state.upload_store {
        Some(s) => s.clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Upload storage is not configured",
            )
                .into_response();
        }
    };

    let mut data: Option<Bytes> = None;
    let mut content_type: Option<String> = None;
    let mut filename: Option<String> = None;

    while let Some(field) = match multipart.next_field().await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("upload multipart next_field error: {e:?}");
            return (StatusCode::BAD_REQUEST, "Invalid multipart").into_response();
        }
    } {
        if field.name().as_deref() != Some("file") {
            continue;
        }
        if let Some(ct) = field.content_type() {
            content_type = Some(ct.to_string());
        }
        if let Some(name) = field.file_name() {
            filename = Some(name.to_string());
        }
        let bytes = match field.bytes().await {
            Ok(b) => b,
            Err(e) => {
                eprintln!("upload field bytes error: {e:?}");
                return (StatusCode::BAD_REQUEST, "Invalid file data").into_response();
            }
        };
        if bytes.len() > MAX_IMAGE_SIZE {
            return (StatusCode::PAYLOAD_TOO_LARGE, "File too large").into_response();
        }
        data = Some(bytes);
        break;
    }

    let data = match data {
        Some(d) => d,
        None => return (StatusCode::BAD_REQUEST, "Missing file field").into_response(),
    };

    let ext = match extension_from_content_type_or_filename(
        content_type.as_deref(),
        filename.as_deref(),
    ) {
        Some(e) => e,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                "Unsupported file type (allowed: jpeg, png, gif, webp)",
            )
                .into_response();
        }
    };

    let object_name = format!("{}.{}", Uuid::new_v4(), ext);
    let path = ObjectStorePath::from_iter([UPLOAD_PREFIX, &object_name]);

    if let Err(e) = store.put(&path, data.into()).await {
        eprintln!("upload store put error: {e:?}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Upload failed").into_response();
    }

    let url = format!("/api/uploads/attachments/{}", object_name);
    (StatusCode::OK, axum::Json(UploadResponse { url })).into_response()
}

fn extension_from_content_type_or_filename(
    content_type: Option<&str>,
    filename: Option<&str>,
) -> Option<&'static str> {
    if let Some(ct) = content_type {
        if ALLOWED_IMAGE_TYPES.contains(&ct) {
            return match ct {
                "image/jpeg" => Some("jpg"),
                "image/png" => Some("png"),
                "image/gif" => Some("gif"),
                "image/webp" => Some("webp"),
                _ => None,
            };
        }
    }
    if let Some(name) = filename {
        let lower = name.to_lowercase();
        if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
            return Some("jpg");
        }
        if lower.ends_with(".png") {
            return Some("png");
        }
        if lower.ends_with(".gif") {
            return Some("gif");
        }
        if lower.ends_with(".webp") {
            return Some("webp");
        }
    }
    None
}

/// GET /uploads/attachments/:filename — serve file from store
pub async fn get_attachment(
    State(state): State<AppState>,
    AxumPath(filename): AxumPath<String>,
) -> Response {
    let store = match &state.upload_store {
        Some(s) => s.clone(),
        None => {
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    if filename.is_empty()
        || filename.contains("..")
        || filename.contains('/')
        || filename.contains('\\')
    {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let path = ObjectStorePath::from_iter([UPLOAD_PREFIX, &filename]);

    let get_result = match store.get(&path).await {
        Ok(r) => r,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let bytes = match get_result.bytes().await {
        Ok(b) => b,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let content_type = content_type_for_filename(&filename);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "public, max-age=31536000")
        .body(Body::from(bytes))
        .unwrap()
}

fn content_type_for_filename(filename: &str) -> &'static str {
    let lower = filename.to_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else {
        "application/octet-stream"
    }
}
