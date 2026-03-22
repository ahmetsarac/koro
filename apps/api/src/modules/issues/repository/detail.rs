use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct IssueFullRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub key_seq: i32,
    pub title: String,
    pub description: Option<String>,
    pub workflow_status_id: Uuid,
    pub status_slug: String,
    pub status_name: String,
    pub status_category: String,
    pub is_blocked: bool,
    pub priority: String,
    pub assignee_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub project_key: String,
    pub assignee_name: Option<String>,
}

pub async fn find_issue_by_id(pool: &PgPool, issue_id: Uuid) -> Result<Option<IssueFullRow>, sqlx::Error> {
    let r = sqlx::query_as::<_, IssueFullRow>(
        r#"
        SELECT
            i.id,
            i.project_id,
            i.key_seq,
            i.title,
            i.description,
            i.workflow_status_id,
            pws.slug AS status_slug,
            pws.name AS status_name,
            pws.category AS status_category,
            (EXISTS (
                SELECT 1 FROM issue_relations ir
                WHERE ir.target_issue_id = i.id AND ir.relation_type = 'blocks'
            )) AS is_blocked,
            i.priority,
            i.assignee_id,
            i.created_at,
            i.updated_at,
            p.project_key,
            u.name as assignee_name
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        JOIN project_workflow_statuses pws ON pws.id = i.workflow_status_id
        LEFT JOIN users u ON u.id = i.assignee_id
        WHERE i.id = $1
        "#,
    )
    .bind(issue_id)
    .fetch_optional(pool)
    .await?;
    Ok(r)
}

pub async fn find_issue_by_org_slug_key(
    pool: &PgPool,
    org_slug: &str,
    project_key: &str,
    key_seq: i32,
) -> Result<Option<IssueFullRow>, sqlx::Error> {
    let r = sqlx::query_as::<_, IssueFullRow>(
        r#"
        SELECT
            i.id,
            i.project_id,
            i.key_seq,
            i.title,
            i.description,
            i.workflow_status_id,
            pws.slug AS status_slug,
            pws.name AS status_name,
            pws.category AS status_category,
            (EXISTS (
                SELECT 1 FROM issue_relations ir
                WHERE ir.target_issue_id = i.id AND ir.relation_type = 'blocks'
            )) AS is_blocked,
            i.priority,
            i.assignee_id,
            i.created_at,
            i.updated_at,
            p.project_key,
            u.name as assignee_name
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        JOIN organizations o ON o.id = p.org_id
        JOIN project_workflow_statuses pws ON pws.id = i.workflow_status_id
        LEFT JOIN users u ON u.id = i.assignee_id
        WHERE o.slug = $1
          AND p.project_key = $2
          AND i.key_seq = $3
        "#,
    )
    .bind(org_slug)
    .bind(project_key)
    .bind(key_seq)
    .fetch_optional(pool)
    .await?;
    Ok(r)
}

#[derive(sqlx::FromRow)]
pub struct IssueUpdateRow {
    pub issue_id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub priority: String,
}

pub async fn find_issue_for_patch_by_org(
    pool: &PgPool,
    org_id: Uuid,
    project_key: &str,
    key_seq: i32,
) -> Result<Option<IssueUpdateRow>, sqlx::Error> {
    let r = sqlx::query_as::<_, IssueUpdateRow>(
        r#"
        SELECT i.id as issue_id, i.project_id, i.title, i.description, i.priority
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
