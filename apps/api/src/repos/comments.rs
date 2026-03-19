use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn insert_comment(
    pool: &PgPool,
    org_id: Uuid,
    project_id: Uuid,
    issue_id: Uuid,
    author_id: Uuid,
    body: &str,
) -> Result<(Uuid, DateTime<Utc>), sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO comments (org_id, project_id, issue_id, author_id, body)
        VALUES ($1,$2,$3,$4,$5)
        RETURNING id, created_at
        "#,
        org_id,
        project_id,
        issue_id,
        author_id,
        body
    )
    .fetch_one(pool)
    .await?;
    Ok((row.id, row.created_at))
}

pub async fn insert_comment_added_event(
    pool: &PgPool,
    org_id: Uuid,
    issue_id: Uuid,
    actor_id: Uuid,
    payload: Value,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO issue_events (org_id, issue_id, actor_id, event_type, payload)
        VALUES ($1,$2,$3,$4,$5)
        "#,
        org_id,
        issue_id,
        actor_id,
        "comment_added",
        payload
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
pub struct CommentRow {
    pub id: Uuid,
    pub author_id: Option<Uuid>,
    pub author_name: Option<String>,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

pub async fn list_comments_for_issue(
    pool: &PgPool,
    issue_id: Uuid,
) -> Result<Vec<CommentRow>, sqlx::Error> {
    sqlx::query_as::<_, CommentRow>(
        r#"
        SELECT c.id, c.author_id, u.name as author_name, c.body, c.created_at
        FROM comments c
        LEFT JOIN users u ON u.id = c.author_id
        WHERE c.issue_id = $1
        ORDER BY c.created_at ASC
        "#,
    )
    .bind(issue_id)
    .fetch_all(pool)
    .await
}
