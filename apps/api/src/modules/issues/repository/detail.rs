use sqlx::PgPool;
use uuid::Uuid;

pub struct IssueFullRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub key_seq: i32,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub assignee_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub project_key: String,
    pub assignee_name: Option<String>,
}

pub async fn find_issue_by_id(pool: &PgPool, issue_id: Uuid) -> Result<Option<IssueFullRow>, sqlx::Error> {
    let r = sqlx::query!(
        r#"
        SELECT
            i.id,
            i.project_id,
            i.key_seq,
            i.title,
            i.description,
            i.status,
            i.priority,
            i.assignee_id,
            i.created_at,
            i.updated_at,
            p.project_key,
            u.name as "assignee_name?"
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        LEFT JOIN users u ON u.id = i.assignee_id
        WHERE i.id = $1
        "#,
        issue_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(r.map(|row| IssueFullRow {
        id: row.id,
        project_id: row.project_id,
        key_seq: row.key_seq,
        title: row.title,
        description: row.description,
        status: row.status,
        priority: row.priority,
        assignee_id: row.assignee_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        project_key: row.project_key,
        assignee_name: row.assignee_name,
    }))
}

pub async fn find_issue_by_org_slug_key(
    pool: &PgPool,
    org_slug: &str,
    project_key: &str,
    key_seq: i32,
) -> Result<Option<IssueFullRow>, sqlx::Error> {
    let r = sqlx::query!(
        r#"
        SELECT
            i.id,
            i.project_id,
            i.key_seq,
            i.title,
            i.description,
            i.status,
            i.priority,
            i.assignee_id,
            i.created_at,
            i.updated_at,
            p.project_key,
            u.name as "assignee_name?"
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        JOIN organizations o ON o.id = p.org_id
        LEFT JOIN users u ON u.id = i.assignee_id
        WHERE o.slug = $1
          AND p.project_key = $2
          AND i.key_seq = $3
        "#,
        org_slug,
        project_key,
        key_seq
    )
    .fetch_optional(pool)
    .await?;
    Ok(r.map(|row| IssueFullRow {
        id: row.id,
        project_id: row.project_id,
        key_seq: row.key_seq,
        title: row.title,
        description: row.description,
        status: row.status,
        priority: row.priority,
        assignee_id: row.assignee_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        project_key: row.project_key,
        assignee_name: row.assignee_name,
    }))
}

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
    let r = sqlx::query!(
        r#"
        SELECT i.id as issue_id, i.project_id, i.title, i.description, i.priority
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        WHERE p.org_id = $1 AND p.project_key = $2 AND i.key_seq = $3
        "#,
        org_id,
        project_key,
        key_seq
    )
    .fetch_optional(pool)
    .await?;
    Ok(r.map(|row| IssueUpdateRow {
        issue_id: row.issue_id,
        project_id: row.project_id,
        title: row.title,
        description: row.description,
        priority: row.priority,
    }))
}
