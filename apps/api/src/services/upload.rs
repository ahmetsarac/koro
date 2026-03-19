use std::sync::Arc;

use axum::extract::Multipart;
use bytes::Bytes;
use object_store::aws::AmazonS3;
use object_store::path::Path as ObjectStorePath;
use object_store::ObjectStoreExt;
use serde::Serialize;
use uuid::Uuid;

use crate::error::AppError;

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

pub fn content_type_for_filename(filename: &str) -> &'static str {
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
