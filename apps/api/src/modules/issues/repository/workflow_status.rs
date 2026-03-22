//! Project-scoped workflow statuses (category + custom name/slug).

use chrono::{DateTime, Utc};
use sqlx::{Executor, Postgres, Transaction};
use uuid::Uuid;

pub const DEFAULT_SEED: &[(&str, &str, &str, i32, bool)] = &[
    ("backlog", "Backlog", "backlog", 0, true),
    ("unstarted", "Todo", "todo", 1, false),
    ("started", "In Progress", "in_progress", 2, false),
    ("completed", "Done", "done", 3, false),
    ("canceled", "Canceled", "canceled", 4, false),
];

pub async fn insert_default_workflow_statuses_tx(
    tx: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
) -> Result<(), sqlx::Error> {
    for (category, name, slug, position, is_default) in DEFAULT_SEED {
        sqlx::query(
            r#"
            INSERT INTO project_workflow_statuses (project_id, category, name, slug, position, is_default)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(project_id)
        .bind(*category)
        .bind(*name)
        .bind(*slug)
        .bind(*position)
        .bind(*is_default)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

pub async fn default_workflow_status_id_for_project<'e, E>(e: E, project_id: Uuid) -> Result<Option<Uuid>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id FROM project_workflow_statuses
        WHERE project_id = $1 AND is_default = true
        LIMIT 1
        "#,
    )
    .bind(project_id)
    .fetch_optional(e)
    .await
}

pub async fn resolve_status_id_by_slug<'e, E>(
    e: E,
    project_id: Uuid,
    slug: &str,
) -> Result<Option<Uuid>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_scalar::<_, Uuid>(
        r#"SELECT id FROM project_workflow_statuses WHERE project_id = $1 AND slug = $2"#,
    )
    .bind(project_id)
    .bind(slug)
    .fetch_optional(e)
    .await
}

/// First workflow row in this category for the project (by position).
pub async fn first_status_id_in_category_for_project<'e, E>(
    e: E,
    project_id: Uuid,
    category: &str,
) -> Result<Option<Uuid>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id FROM project_workflow_statuses
        WHERE project_id = $1 AND category = $2
        ORDER BY position ASC
        LIMIT 1
        "#,
    )
    .bind(project_id)
    .bind(category)
    .fetch_optional(e)
    .await
}

pub async fn status_belongs_to_project<'e, E>(
    e: E,
    project_id: Uuid,
    workflow_status_id: Uuid,
) -> Result<bool, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let ok: Option<i32> = sqlx::query_scalar(
        r#"SELECT 1 FROM project_workflow_statuses WHERE id = $1 AND project_id = $2"#,
    )
    .bind(workflow_status_id)
    .bind(project_id)
    .fetch_optional(e)
    .await?;
    Ok(ok.is_some())
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WorkflowStatusRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub category: String,
    pub name: String,
    pub slug: String,
    pub position: i32,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

pub async fn list_workflow_statuses_for_project_ordered<'e, E>(
    e: E,
    project_id: Uuid,
) -> Result<Vec<WorkflowStatusRow>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as::<_, WorkflowStatusRow>(
        r#"
        SELECT id, project_id, category, name, slug, position, is_default, created_at
        FROM project_workflow_statuses
        WHERE project_id = $1
        ORDER BY
          CASE category
            WHEN 'backlog' THEN 0
            WHEN 'unstarted' THEN 1
            WHEN 'started' THEN 2
            WHEN 'completed' THEN 3
            WHEN 'canceled' THEN 4
            ELSE 5
          END,
          position ASC,
          slug ASC
        "#,
    )
    .bind(project_id)
    .fetch_all(e)
    .await
}

pub async fn get_workflow_status<'e, E>(e: E, id: Uuid) -> Result<Option<WorkflowStatusRow>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as::<_, WorkflowStatusRow>(
        r#"
        SELECT id, project_id, category, name, slug, position, is_default, created_at
        FROM project_workflow_statuses
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(e)
    .await
}

pub async fn insert_workflow_status<'e, E>(
    e: E,
    project_id: Uuid,
    category: &str,
    name: &str,
    slug: &str,
    position: i32,
    is_default: bool,
) -> Result<WorkflowStatusRow, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as::<_, WorkflowStatusRow>(
        r#"
        INSERT INTO project_workflow_statuses (project_id, category, name, slug, position, is_default)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, project_id, category, name, slug, position, is_default, created_at
        "#,
    )
    .bind(project_id)
    .bind(category)
    .bind(name)
    .bind(slug)
    .bind(position)
    .bind(is_default)
    .fetch_one(e)
    .await
}

pub async fn update_workflow_status_fields<'e, E>(
    e: E,
    id: Uuid,
    name: Option<&str>,
    slug: Option<&str>,
    position: Option<i32>,
) -> Result<Option<WorkflowStatusRow>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as::<_, WorkflowStatusRow>(
        r#"
        UPDATE project_workflow_statuses
        SET
          name = COALESCE($2, name),
          slug = COALESCE($3, slug),
          position = COALESCE($4, position)
        WHERE id = $1
        RETURNING id, project_id, category, name, slug, position, is_default, created_at
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(slug)
    .bind(position)
    .fetch_optional(e)
    .await
}

pub async fn clear_default_flags_for_project_tx(
    tx: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(r#"UPDATE project_workflow_statuses SET is_default = false WHERE project_id = $1"#)
        .bind(project_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub async fn set_workflow_status_default_tx(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    project_id: Uuid,
) -> Result<(), sqlx::Error> {
    clear_default_flags_for_project_tx(tx, project_id).await?;
    sqlx::query(
        r#"UPDATE project_workflow_statuses SET is_default = true WHERE id = $1 AND project_id = $2"#,
    )
    .bind(id)
    .bind(project_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn count_issues_on_status<'e, E>(e: E, workflow_status_id: Uuid) -> Result<i64, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let c: i64 = sqlx::query_scalar(r#"SELECT COUNT(*)::bigint FROM issues WHERE workflow_status_id = $1"#)
        .bind(workflow_status_id)
        .fetch_one(e)
        .await?;
    Ok(c)
}

pub async fn reassign_issues_status<'e, E>(
    e: E,
    from_id: Uuid,
    to_id: Uuid,
) -> Result<u64, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let r = sqlx::query(r#"UPDATE issues SET workflow_status_id = $2, updated_at = NOW() WHERE workflow_status_id = $1"#)
        .bind(from_id)
        .bind(to_id)
        .execute(e)
        .await?;
    Ok(r.rows_affected())
}

pub async fn delete_workflow_status<'e, E>(e: E, id: Uuid) -> Result<u64, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let r = sqlx::query(r#"DELETE FROM project_workflow_statuses WHERE id = $1"#)
        .bind(id)
        .execute(e)
        .await?;
    Ok(r.rows_affected())
}

pub async fn max_position_for_project_and_category<'e, E>(
    e: E,
    project_id: Uuid,
    category: &str,
) -> Result<Option<i32>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_scalar::<_, Option<i32>>(
        r#"SELECT MAX(position) FROM project_workflow_statuses WHERE project_id = $1 AND category = $2"#,
    )
    .bind(project_id)
    .bind(category)
    .fetch_one(e)
    .await
}

/// True if slug already used in project (`except_id` skips that row for updates).
pub async fn user_can_manage_workflow_statuses<'e, E>(
    e: E,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let ok: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
          SELECT 1 FROM project_members pm
          WHERE pm.project_id = $1 AND pm.user_id = $2 AND pm.project_role = 'project_manager'
        )
        OR EXISTS (
          SELECT 1 FROM projects p
          JOIN org_members om ON om.org_id = p.org_id AND om.user_id = $2 AND om.org_role = 'org_admin'
          WHERE p.id = $1
        )
        "#,
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_one(e)
    .await?;
    Ok(ok)
}

pub async fn slug_in_use<'e, E>(
    e: E,
    project_id: Uuid,
    slug: &str,
    except_id: Option<Uuid>,
) -> Result<bool, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let found = match except_id {
        Some(ex) => {
            sqlx::query_scalar::<_, i32>(
                r#"SELECT 1 FROM project_workflow_statuses WHERE project_id = $1 AND slug = $2 AND id <> $3 LIMIT 1"#,
            )
            .bind(project_id)
            .bind(slug)
            .bind(ex)
            .fetch_optional(e)
            .await?
        }
        None => {
            sqlx::query_scalar::<_, i32>(
                r#"SELECT 1 FROM project_workflow_statuses WHERE project_id = $1 AND slug = $2 LIMIT 1"#,
            )
            .bind(project_id)
            .bind(slug)
            .fetch_optional(e)
            .await?
        }
    };
    Ok(found.is_some())
}
