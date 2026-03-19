use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub struct ProjectForCreate {
    pub org_id: Uuid,
    pub project_key: String,
    pub next_issue_seq: i32,
}

pub async fn lock_project_for_create(
    tx: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
) -> Result<Option<ProjectForCreate>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT org_id, project_key, next_issue_seq
        FROM projects
        WHERE id = $1
        FOR UPDATE
        "#,
        project_id
    )
    .fetch_optional(&mut **tx)
    .await?;
    Ok(row.map(|r| ProjectForCreate {
        org_id: r.org_id,
        project_key: r.project_key,
        next_issue_seq: r.next_issue_seq,
    }))
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
    status: &str,
    priority: &str,
) -> Result<(Uuid, String), sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO issues (org_id, project_id, key_seq, title, description, reporter_id, assignee_id, status, priority)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, title
        "#,
        org_id,
        project_id,
        key_seq,
        title,
        description,
        reporter_id,
        assignee_id,
        status,
        priority
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok((row.id, row.title))
}

pub async fn increment_project_issue_seq(
    tx: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE projects SET next_issue_seq = next_issue_seq + 1 WHERE id = $1"#,
        project_id
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
