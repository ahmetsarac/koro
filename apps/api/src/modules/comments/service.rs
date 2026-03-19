use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::{
    comments::models::*,
    comments::repository as comments_repo,
    core::AppError,
    events::repository as events_repo,
    issues::models::parse_issue_key,
    orgs::repository as orgs_repo,
};

pub async fn create_comment(
    pool: &PgPool,
    org_slug: &str,
    issue_key: &str,
    user_id: Uuid,
    body: &str,
) -> Result<CreateCommentResponse, AppError> {
    if body.trim().is_empty() {
        return Err(AppError::BadRequest(Some("body is required")));
    }

    let (project_key, key_seq) =
        parse_issue_key(issue_key).ok_or(AppError::BadRequest(Some("invalid issue key")))?;

    let org_id = orgs_repo::find_org_id_by_slug(pool, org_slug)
        .await
        .map_err(|e| {
            tracing::error!(?e, "create_comment org resolve");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let issue_row = events_repo::resolve_issue_in_org(pool, org_id, project_key, key_seq)
        .await
        .map_err(|e| {
            tracing::error!(?e, "create_comment issue resolve");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let is_member = events_repo::is_project_member(pool, issue_row.project_id, user_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "create_comment member check");
            AppError::Internal
        })?;

    if !is_member {
        return Err(AppError::Forbidden);
    }

    let (comment_id, created_at) = comments_repo::insert_comment(
        pool,
        org_id,
        issue_row.project_id,
        issue_row.issue_id,
        user_id,
        body,
    )
    .await
    .map_err(|e| {
        tracing::error!(?e, "create_comment insert");
        AppError::Internal
    })?;

    let payload = serde_json::json!({ "comment_id": comment_id });
    if let Err(e) = comments_repo::insert_comment_added_event(
        pool,
        org_id,
        issue_row.issue_id,
        user_id,
        payload,
    )
    .await
    {
        tracing::error!(?e, "comment_added event insert");
    }

    Ok(CreateCommentResponse {
        comment_id,
        created_at,
    })
}

pub async fn list_comments(
    pool: &PgPool,
    org_slug: &str,
    issue_key: &str,
    user_id: Uuid,
) -> Result<ListCommentsResponse, AppError> {
    let (project_key, key_seq) =
        parse_issue_key(issue_key).ok_or(AppError::BadRequest(Some("invalid issue key")))?;

    let org_id = orgs_repo::find_org_id_by_slug(pool, org_slug)
        .await
        .map_err(|e| {
            tracing::error!(?e, "list_comments org resolve");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let issue_row = events_repo::resolve_issue_in_org(pool, org_id, project_key, key_seq)
        .await
        .map_err(|e| {
            tracing::error!(?e, "list_comments issue resolve");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let is_member = events_repo::is_project_member(pool, issue_row.project_id, user_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "list_comments member check");
            AppError::Internal
        })?;

    if !is_member {
        return Err(AppError::Forbidden);
    }

    let rows = comments_repo::list_comments_for_issue(pool, issue_row.issue_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "list_comments query");
            AppError::Internal
        })?;

    let items = rows
        .into_iter()
        .map(|r| CommentItem {
            comment_id: r.id,
            author_id: r.author_id,
            author_name: r.author_name,
            body: r.body,
            created_at: r.created_at,
        })
        .collect();

    Ok(ListCommentsResponse { items })
}
