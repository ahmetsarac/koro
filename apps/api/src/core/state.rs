use std::sync::Arc;

use object_store::aws::AmazonS3;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    /// S3-compatible store (MinIO) for uploads. None if MINIO_* env not set.
    pub upload_store: Option<Arc<AmazonS3>>,
}
