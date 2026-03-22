use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct ProjectForCreate {
    pub org_id: Uuid,
    pub project_key: String,
    pub next_issue_seq: i32,
}

pub async fn lock_project_for_create(
    tx: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
) -> Result<Option<ProjectForCreate>, sqlx::Error> {
    let row = sqlx::query_as::<_, ProjectForCreate>(
        r#"
        SELECT org_id, project_key, next_issue_seq
        FROM projects
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(project_id)
    .fetch_optional(&mut **tx)
    .await?;
    Ok(row)
}

pub async fn assignee_is_project_member_tx(
    tx: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    assignee_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let ok = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT 1
        FROM project_members
        WHERE project_id = $1 AND user_id = $2
        "#,
    )
    .bind(project_id)
    .bind(assignee_id)
    .fetch_optional(&mut **tx)
    .await?;
    Ok(ok.is_some())
}

pub async fn insert_issue_returning_id_title(
    tx: &mut Transaction<'_, Postgres>,
    org_id: Uuid,
    project_id: Uuid,
    key_seq: i32,
    title: String,
    description: Option<String>,
    reporter_id: Uuid,
    assignee_id: Option<Uuid>,
    workflow_status_id: Uuid,
    is_blocked: bool,
    priority: &str,
) -> Result<(Uuid, String), sqlx::Error> {
    let row: (Uuid, String) = sqlx::query_as(
        r#"
        INSERT INTO issues (org_id, project_id, key_seq, title, description, reporter_id, assignee_id, workflow_status_id, is_blocked, priority)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, title
        "#,
    )
    .bind(org_id)
    .bind(project_id)
    .bind(key_seq)
    .bind(&title)
    .bind(&description)
    .bind(reporter_id)
    .bind(assignee_id)
    .bind(workflow_status_id)
    .bind(is_blocked)
    .bind(priority)
    .fetch_one(&mut **tx)
    .await?;
    Ok(row)
}

pub async fn increment_project_issue_seq(
    tx: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(r#"UPDATE projects SET next_issue_seq = next_issue_seq + 1 WHERE id = $1"#)
        .bind(project_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
