use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateCommentRequest {
    pub body: String,
}

#[derive(Serialize)]
pub struct CreateCommentResponse {
    pub comment_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct CommentItem {
    pub comment_id: Uuid,
    pub author_id: Option<Uuid>,
    pub author_name: Option<String>,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ListCommentsResponse {
    pub items: Vec<CommentItem>,
}
