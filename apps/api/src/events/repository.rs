use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct IssueScopeRow {
    pub issue_id: Uuid,
    pub project_id: Uuid,
}

#[derive(sqlx::FromRow)]
pub struct IssueEventRow {
    pub id: Uuid,
    pub event_type: String,
    pub payload: Value,
    pub created_at: DateTime<Utc>,
    pub actor_id: Option<Uuid>,
    pub actor_name: Option<String>,
    pub actor_email: Option<String>,
}

pub async fn resolve_issue_in_org(
    pool: &PgPool,
    org_id: Uuid,
    project_key: &str,
    key_seq: i32,
) -> Result<Option<IssueScopeRow>, sqlx::Error> {
    sqlx::query_as::<_, IssueScopeRow>(
        r#"
        SELECT i.id as issue_id, i.project_id
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        WHERE p.org_id = $1 AND p.project_key = $2 AND i.key_seq = $3
        "#,
    )
    .bind(org_id)
    .bind(project_key)
    .bind(key_seq)
    .fetch_optional(pool)
    .await
}

pub async fn is_project_member(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let found = sqlx::query_scalar::<_, i32>(
        "SELECT 1 FROM project_members WHERE project_id = $1 AND user_id = $2",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(found.is_some())
}

pub async fn list_events_for_issue(
    pool: &PgPool,
    issue_id: Uuid,
) -> Result<Vec<IssueEventRow>, sqlx::Error> {
    sqlx::query_as::<_, IssueEventRow>(
        r#"
        SELECT
          e.id,
          e.event_type,
          e.payload,
          e.created_at,
          e.actor_id,
          u.name as actor_name,
          u.email as actor_email
        FROM issue_events e
        LEFT JOIN users u ON u.id = e.actor_id
        WHERE e.issue_id = $1
        ORDER BY e.created_at DESC
        LIMIT 200
        "#,
    )
    .bind(issue_id)
    .fetch_all(pool)
    .await
}
