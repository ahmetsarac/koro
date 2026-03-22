use sqlx::PgPool;
use uuid::Uuid;

pub async fn project_key_by_id(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>("SELECT project_key FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_optional(pool)
        .await
}

#[derive(sqlx::FromRow)]
pub struct ProjectIdAndKey {
    pub id: Uuid,
    pub project_key: String,
}

pub async fn find_project_by_org_slug_and_key(
    pool: &PgPool,
    org_slug: &str,
    project_key: &str,
) -> Result<Option<ProjectIdAndKey>, sqlx::Error> {
    let row = sqlx::query_as::<_, ProjectIdAndKey>(
        r#"
        SELECT p.id, p.project_key
        FROM projects p
        JOIN organizations o ON o.id = p.org_id
        WHERE o.slug = $1 AND p.project_key = $2
        "#,
    )
    .bind(org_slug)
    .bind(project_key)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

#[derive(sqlx::FromRow)]
pub struct IssueSummaryRow {
    pub id: Uuid,
    pub key_seq: i32,
    pub title: String,
    pub workflow_status_id: Uuid,
    pub status_slug: String,
    pub status_name: String,
    pub status_category: String,
    pub is_blocked: bool,
}

pub async fn list_issue_summaries_for_project(
    pool: &PgPool,
    project_id: Uuid,
    workflow_status_filter: Option<Uuid>,
    assignee_filter: Option<Uuid>,
    q_filter: Option<String>,
    limit: i64,
    offset: i64,
) -> Result<Vec<IssueSummaryRow>, sqlx::Error> {
    sqlx::query_as::<_, IssueSummaryRow>(
        r#"
        SELECT
            i.id,
            i.key_seq,
            i.title,
            i.workflow_status_id,
            pws.slug AS status_slug,
            pws.name AS status_name,
            pws.category AS status_category,
            (EXISTS (
                SELECT 1 FROM issue_relations ir
                WHERE ir.target_issue_id = i.id AND ir.relation_type = 'blocks'
            )) AS is_blocked
        FROM issues i
        JOIN project_workflow_statuses pws ON pws.id = i.workflow_status_id
        WHERE i.project_id = $1
            AND ($2::uuid IS NULL OR i.workflow_status_id = $2)
            AND ($3::uuid IS NULL OR i.assignee_id = $3)
            AND (
                $4::text IS NULL
                OR i.title ILIKE '%' || $4 || '%'
                OR COALESCE(i.description, '') ILIKE '%' || $4 || '%'
            )
        ORDER BY i.key_seq DESC
        LIMIT $5
        OFFSET $6
        "#,
    )
    .bind(project_id)
    .bind(workflow_status_filter)
    .bind(assignee_filter)
    .bind(q_filter)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn count_issues_for_project_filtered(
    pool: &PgPool,
    project_id: Uuid,
    workflow_status_filter: Option<Uuid>,
    assignee_filter: Option<Uuid>,
    q_filter: Option<String>,
) -> Result<i64, sqlx::Error> {
    let c: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM issues i
        WHERE i.project_id = $1
            AND ($2::uuid IS NULL OR i.workflow_status_id = $2)
            AND ($3::uuid IS NULL OR i.assignee_id = $3)
            AND (
                $4::text IS NULL
                OR i.title ILIKE '%' || $4 || '%'
                OR COALESCE(i.description, '') ILIKE '%' || $4 || '%'
            )
        "#,
    )
    .bind(project_id)
    .bind(workflow_status_filter)
    .bind(assignee_filter)
    .bind(q_filter)
    .fetch_one(pool)
    .await?;
    Ok(c)
}

pub async fn list_board_issues_by_key_seq(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<IssueSummaryRow>, sqlx::Error> {
    sqlx::query_as::<_, IssueSummaryRow>(
        r#"
        SELECT
            i.id,
            i.key_seq,
            i.title,
            i.workflow_status_id,
            pws.slug AS status_slug,
            pws.name AS status_name,
            pws.category AS status_category,
            (EXISTS (
                SELECT 1 FROM issue_relations ir
                WHERE ir.target_issue_id = i.id AND ir.relation_type = 'blocks'
            )) AS is_blocked
        FROM issues i
        JOIN project_workflow_statuses pws ON pws.id = i.workflow_status_id
        WHERE i.project_id = $1
        ORDER BY i.key_seq DESC
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

pub async fn list_board_issues_by_board_order(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<IssueSummaryRow>, sqlx::Error> {
    sqlx::query_as::<_, IssueSummaryRow>(
        r#"
        SELECT
            i.id,
            i.key_seq,
            i.title,
            i.workflow_status_id,
            pws.slug AS status_slug,
            pws.name AS status_name,
            pws.category AS status_category,
            (EXISTS (
                SELECT 1 FROM issue_relations ir
                WHERE ir.target_issue_id = i.id AND ir.relation_type = 'blocks'
            )) AS is_blocked
        FROM issues i
        JOIN project_workflow_statuses pws ON pws.id = i.workflow_status_id
        WHERE i.project_id = $1
        ORDER BY i.board_order ASC, i.key_seq DESC
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}
