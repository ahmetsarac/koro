use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn set_issue_workflow_status(
    pool: &PgPool,
    issue_id: Uuid,
    workflow_status_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE issues SET workflow_status_id = $1, updated_at = NOW() WHERE id = $2"#,
    )
    .bind(workflow_status_id)
    .bind(issue_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_issue_priority(
    pool: &PgPool,
    issue_id: Uuid,
    priority: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(r#"UPDATE issues SET priority = $1, updated_at = NOW() WHERE id = $2"#)
        .bind(priority)
        .bind(issue_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_issue_title_desc_priority(
    pool: &PgPool,
    issue_id: Uuid,
    title: &str,
    description: Option<String>,
    priority: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(r#"UPDATE issues SET title = $1, description = $2, priority = $3 WHERE id = $4"#)
        .bind(title)
        .bind(description)
        .bind(priority)
        .bind(issue_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn find_issue_project_id_only(
    pool: &PgPool,
    issue_id: Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>("SELECT project_id FROM issues WHERE id = $1")
        .bind(issue_id)
        .fetch_optional(pool)
        .await
}

pub async fn update_issue_status_and_board_order(
    pool: &PgPool,
    issue_id: Uuid,
    workflow_status_id: Uuid,
    position: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE issues
        SET workflow_status_id = $1, board_order = $2, updated_at = NOW()
        WHERE id = $3
        "#,
    )
    .bind(workflow_status_id)
    .bind(position)
    .bind(issue_id)
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
pub struct IssueIdRow {
    pub issue_id: Uuid,
    pub project_id: Uuid,
}

pub async fn find_issue_id_in_org(
    pool: &PgPool,
    org_id: Uuid,
    project_key: &str,
    key_seq: i32,
) -> Result<Option<IssueIdRow>, sqlx::Error> {
    let r = sqlx::query_as::<_, IssueIdRow>(
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
    .await?;
    Ok(r)
}

pub async fn set_issue_assignee(
    pool: &PgPool,
    issue_id: Uuid,
    assignee_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(r#"UPDATE issues SET assignee_id = $1 WHERE id = $2"#)
        .bind(assignee_id)
        .bind(issue_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn clear_issue_assignee(pool: &PgPool, issue_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(r#"UPDATE issues SET assignee_id = NULL WHERE id = $1"#)
        .bind(issue_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn find_issue_id_only_in_org(
    pool: &PgPool,
    org_id: Uuid,
    project_key: &str,
    key_seq: i32,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT i.id
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

pub async fn insert_assigned_event(
    pool: &PgPool,
    org_id: Uuid,
    issue_id: Uuid,
    actor_id: Uuid,
    payload: Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO issue_events (org_id, issue_id, actor_id, event_type, payload)
        VALUES ($1,$2,$3,$4,$5)
        "#,
    )
    .bind(org_id)
    .bind(issue_id)
    .bind(actor_id)
    .bind("assigned")
    .bind(payload)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_unassigned_event(
    pool: &PgPool,
    org_id: Uuid,
    issue_id: Uuid,
    actor_id: Uuid,
    payload: Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO issue_events (org_id, issue_id, actor_id, event_type, payload)
        VALUES ($1,$2,$3,$4,$5)
        "#,
    )
    .bind(org_id)
    .bind(issue_id)
    .bind(actor_id)
    .bind("unassigned")
    .bind(payload)
    .execute(pool)
    .await?;
    Ok(())
}

/// `None` if issue missing; `Some(None)` if active; `Some(Some(t))` if archived.
pub async fn get_issue_archived_state(
    pool: &PgPool,
    issue_id: Uuid,
) -> Result<Option<Option<DateTime<Utc>>>, sqlx::Error> {
    sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
        "SELECT archived_at FROM issues WHERE id = $1",
    )
    .bind(issue_id)
    .fetch_optional(pool)
    .await
}

/// Returns how many rows matched `issue_ids` and were archived (active + project member).
pub async fn archive_issues_for_member(
    pool: &PgPool,
    user_id: Uuid,
    issue_ids: &[Uuid],
) -> Result<u64, sqlx::Error> {
    if issue_ids.is_empty() {
        return Ok(0);
    }
    let res = sqlx::query(
        r#"
        UPDATE issues i
        SET archived_at = NOW(), updated_at = NOW()
        WHERE i.id = ANY($1)
          AND i.archived_at IS NULL
          AND EXISTS (
            SELECT 1 FROM project_members pm
            WHERE pm.project_id = i.project_id AND pm.user_id = $2
          )
        "#,
    )
    .bind(issue_ids)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected())
}

/// Expected count: issues that exist, are active, and user is a project member.
pub async fn count_issues_eligible_for_member_action(
    pool: &PgPool,
    user_id: Uuid,
    issue_ids: &[Uuid],
) -> Result<i64, sqlx::Error> {
    if issue_ids.is_empty() {
        return Ok(0);
    }
    let c: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM issues i
        WHERE i.id = ANY($1)
          AND i.archived_at IS NULL
          AND EXISTS (
            SELECT 1 FROM project_members pm
            WHERE pm.project_id = i.project_id AND pm.user_id = $2
          )
        "#,
    )
    .bind(issue_ids)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(c)
}

#[derive(sqlx::FromRow)]
pub struct IssueIdProjectArchivedRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub archived_at: Option<DateTime<Utc>>,
}

pub async fn fetch_issues_id_project_archived(
    pool: &PgPool,
    issue_ids: &[Uuid],
) -> Result<Vec<IssueIdProjectArchivedRow>, sqlx::Error> {
    if issue_ids.is_empty() {
        return Ok(Vec::new());
    }
    sqlx::query_as::<_, IssueIdProjectArchivedRow>(
        r#"
        SELECT i.id, i.project_id, i.archived_at
        FROM issues i
        WHERE i.id = ANY($1)
        "#,
    )
    .bind(issue_ids)
    .fetch_all(pool)
    .await
}

pub async fn delete_archived_issue_by_org_key(
    pool: &PgPool,
    org_id: Uuid,
    project_key: &str,
    key_seq: i32,
    user_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        r#"
        DELETE FROM issues i
        USING projects p
        WHERE i.project_id = p.id
          AND p.org_id = $1
          AND p.project_key = $2
          AND i.key_seq = $3
          AND i.archived_at IS NOT NULL
          AND EXISTS (
            SELECT 1 FROM project_members pm
            WHERE pm.project_id = i.project_id AND pm.user_id = $4
          )
        "#,
    )
    .bind(org_id)
    .bind(project_key)
    .bind(key_seq)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected())
}
