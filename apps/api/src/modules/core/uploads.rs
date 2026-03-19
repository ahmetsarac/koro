use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Multipart, Path as AxumPath, State},
    http::{header, StatusCode},
    response::Response,
};
use bytes::Bytes;
use object_store::aws::AmazonS3;
use object_store::path::Path as ObjectStorePath;
use object_store::ObjectStoreExt;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::modules::{
    auth::user::AuthUser,
    core::{state::AppState, AppError},
};

const UPLOAD_PREFIX: &str = "attachments";
const MAX_IMAGE_SIZE: usize = 10 * 1024 * 1024; // 10 MiB
const ALLOWED_IMAGE_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
];

#[derive(Serialize, ToSchema)]
pub struct UploadResponse {
    pub url: String,
}

pub async fn upload_multipart(
    store: Arc<AmazonS3>,
    mut multipart: Multipart,
) -> Result<UploadResponse, AppError> {
    let mut data: Option<Bytes> = None;
    let mut content_type: Option<String> = None;
    let mut filename: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!(?e, "upload multipart next_field");
        AppError::BadRequest(Some("Invalid multipart"))
    })? {
        if field.name().as_deref() != Some("file") {
            continue;
        }
        if let Some(ct) = field.content_type() {
            content_type = Some(ct.to_string());
        }
        if let Some(name) = field.file_name() {
            filename = Some(name.to_string());
        }
        let bytes = field.bytes().await.map_err(|e| {
            tracing::error!(?e, "upload field bytes");
            AppError::BadRequest(Some("Invalid file data"))
        })?;
        if bytes.len() > MAX_IMAGE_SIZE {
            return Err(AppError::PayloadTooLarge(Some("File too large")));
        }
        data = Some(bytes);
        break;
    }

    let data = data.ok_or(AppError::BadRequest(Some("Missing file field")))?;

    let ext = extension_from_content_type_or_filename(content_type.as_deref(), filename.as_deref())
        .ok_or(AppError::BadRequest(Some(
            "Unsupported file type (allowed: jpeg, png, gif, webp)",
        )))?;

    let object_name = format!("{}.{}", Uuid::new_v4(), ext);
    let path = ObjectStorePath::from_iter([UPLOAD_PREFIX, &object_name]);

    store.put(&path, data.into()).await.map_err(|e| {
        tracing::error!(?e, "upload store put");
        AppError::Internal
    })?;

    let url = format!("/api/uploads/attachments/{}", object_name);
    Ok(UploadResponse { url })
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

pub async fn fetch_attachment_bytes(
    store: Arc<AmazonS3>,
    filename: &str,
) -> Result<(Bytes, &'static str), AppError> {
    if filename.is_empty()
        || filename.contains("..")
        || filename.contains('/')
        || filename.contains('\\')
    {
        return Err(AppError::BadRequest(None));
    }

    let path = ObjectStorePath::from_iter([UPLOAD_PREFIX, filename]);
    let get_result = store.get(&path).await.map_err(|_| AppError::NotFound)?;

    let bytes = get_result.bytes().await.map_err(|e| {
        tracing::error!(?e, "get_attachment bytes");
        AppError::Internal
    })?;

    let content_type = content_type_for_filename(filename);
    Ok((bytes, content_type))
}

/// POST /uploads — multipart form with field `file` (image).
#[utoipa::path(
    post,
    path = "/uploads",
    tag = "uploads",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Relative URL for the stored object", body = UploadResponse),
        (status = 400, description = "Invalid multipart or file"),
        (status = 401, description = "Unauthorized"),
        (status = 413, description = "Payload too large"),
        (status = 503, description = "Upload storage not configured"),
    )
)]
pub async fn upload(
    State(state): State<AppState>,
    AuthUser(_user_id): AuthUser,
    multipart: Multipart,
) -> Result<(StatusCode, axum::Json<UploadResponse>), AppError> {
    let store = state.upload_store.clone().ok_or(AppError::ServiceUnavailable(Some(
        "Upload storage is not configured",
    )))?;

    let res = upload_multipart(store, multipart).await?;
    Ok((StatusCode::OK, axum::Json(res)))
}

/// GET /uploads/attachments/:filename — returns image bytes.
#[utoipa::path(
    get,
    path = "/uploads/attachments/{filename}",
    tag = "uploads",
    params(
        ("filename" = String, Path, description = "Object basename (uuid.ext)"),
    ),
    responses(
        (status = 200, description = "Binary body with correct Content-Type"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_attachment(
    State(state): State<AppState>,
    AxumPath(filename): AxumPath<String>,
) -> Result<Response, AppError> {
    let store = state
        .upload_store
        .clone()
        .ok_or(AppError::NotFound)?;

    let (bytes, content_type) = fetch_attachment_bytes(store, &filename).await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "public, max-age=31536000")
        .body(Body::from(bytes))
        .unwrap())
}

pub fn build_upload_store() -> Option<Arc<AmazonS3>> {
    use object_store::aws::AmazonS3Builder;

    let endpoint = std::env::var("MINIO_ENDPOINT").ok()?;
    let access_key = std::env::var("MINIO_ACCESS_KEY").ok()?;
    let secret_key = std::env::var("MINIO_SECRET_KEY").ok()?;
    let bucket = std::env::var("MINIO_BUCKET").ok()?;

    let store = AmazonS3Builder::new()
        .with_endpoint(endpoint)
        .with_allow_http(true)
        .with_region("us-east-1")
        .with_bucket_name(&bucket)
        .with_access_key_id(access_key)
        .with_secret_access_key(secret_key)
        .build()
        .ok()?;

    Some(Arc::new(store))
}
