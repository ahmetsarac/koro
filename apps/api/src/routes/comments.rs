use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{auth_user::AuthUser, state::AppState};

fn parse_issue_key(key: &str) -> Option<(&str, i32)> {
    let (project_key, seq_str) = key.split_once('-')?;
    let seq = seq_str.parse::<i32>().ok()?;
    Some((project_key, seq))
}

#[derive(Deserialize)]
pub struct CreateCommentRequest {
    pub body: String,
}

#[derive(Serialize)]
pub struct CreateCommentResponse {
    pub comment_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct CommentItem {
    pub comment_id: uuid::Uuid,
    pub author_id: Option<uuid::Uuid>,
    pub body: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct ListCommentsResponse {
    pub items: Vec<CommentItem>,
}

pub async fn create_comment(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateCommentRequest>,
) -> impl IntoResponse {
    if req.body.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "body is required").into_response();
    }

    let (project_key, key_seq) = match parse_issue_key(&issue_key) {
        Some(v) => v,
        None => return (StatusCode::BAD_REQUEST, "invalid issue key").into_response(),
    };

    // org resolve
    let org_id = match sqlx::query_scalar::<_, uuid::Uuid>(
        r#"SELECT id FROM organizations WHERE slug = $1"#,
    )
    .bind(&org_slug)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("create_comment org resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // issue resolve (org + project_key + key_seq)
    let issue_row = match sqlx::query!(
        r#"
        SELECT i.id as issue_id, i.project_id
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        WHERE p.org_id = $1 AND p.project_key = $2 AND i.key_seq = $3
        "#,
        org_id,
        project_key,
        key_seq
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("create_comment issue resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // authz: project member
    let is_member = match sqlx::query_scalar::<_, i32>(
        r#"SELECT 1 FROM project_members WHERE project_id = $1 AND user_id = $2"#,
    )
    .bind(issue_row.project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("create_comment member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    let rec = match sqlx::query!(
        r#"
        INSERT INTO comments (org_id, project_id, issue_id, author_id, body)
        VALUES ($1,$2,$3,$4,$5)
        RETURNING id, created_at
        "#,
        org_id,
        issue_row.project_id,
        issue_row.issue_id,
        user_id,
        req.body
    )
    .fetch_one(&state.db)
    .await    
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("create_comment insert error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let payload = serde_json::json!({ "comment_id": rec.id });

    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO issue_events (org_id, issue_id, actor_id, event_type, payload)
        VALUES ($1,$2,$3,$4,$5)
        "#,
        org_id,
        issue_row.issue_id,
        user_id,
        "comment_added",
        payload
    )
    .execute(&state.db)
    .await
    {
        eprintln!("comment_added event insert error: {e:?}");
    }

    (
        StatusCode::CREATED,
        Json(CreateCommentResponse {
            comment_id: rec.id,
            created_at: rec.created_at,
        }),
    )
        .into_response()
}

pub async fn list_comments(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl IntoResponse {
    let (project_key, key_seq) = match parse_issue_key(&issue_key) {
        Some(v) => v,
        None => return (StatusCode::BAD_REQUEST, "invalid issue key").into_response(),
    };

    let org_id = match sqlx::query_scalar::<_, uuid::Uuid>(
        r#"SELECT id FROM organizations WHERE slug = $1"#,
    )
    .bind(&org_slug)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("list_comments org resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let issue_row = match sqlx::query!(
        r#"
        SELECT i.id as issue_id, i.project_id
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        WHERE p.org_id = $1 AND p.project_key = $2 AND i.key_seq = $3
        "#,
        org_id,
        project_key,
        key_seq
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("list_comments issue resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let is_member = match sqlx::query_scalar::<_, i32>(
        r#"SELECT 1 FROM project_members WHERE project_id = $1 AND user_id = $2"#,
    )
    .bind(issue_row.project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("list_comments member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    let rows = match sqlx::query!(
        r#"
        SELECT id, author_id, body, created_at
        FROM comments
        WHERE issue_id = $1
        ORDER BY created_at ASC
        "#,
        issue_row.issue_id
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("list_comments query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let items = rows
        .into_iter()
        .map(|r| CommentItem {
            comment_id: r.id,
            author_id: r.author_id,
            body: r.body,
            created_at: r.created_at,
        })
        .collect();

    (StatusCode::OK, Json(ListCommentsResponse { items })).into_response()
}
