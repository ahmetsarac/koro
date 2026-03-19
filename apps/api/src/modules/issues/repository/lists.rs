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

pub struct ProjectIdAndKey {
    pub id: Uuid,
    pub project_key: String,
}

pub async fn find_project_by_org_slug_and_key(
    pool: &PgPool,
    org_slug: &str,
    project_key: &str,
) -> Result<Option<ProjectIdAndKey>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT p.id, p.project_key
        FROM projects p
        JOIN organizations o ON o.id = p.org_id
        WHERE o.slug = $1 AND p.project_key = $2
        "#,
        org_slug,
        project_key
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| ProjectIdAndKey {
        id: r.id,
        project_key: r.project_key,
    }))
}

pub async fn list_issue_summaries_for_project(
    pool: &PgPool,
    project_id: Uuid,
    status_filter: Option<String>,
    assignee_filter: Option<Uuid>,
    q_filter: Option<String>,
    limit: i64,
    offset: i64,
) -> Result<Vec<IssueSummaryRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT id, key_seq, title, status
        FROM issues
        WHERE project_id = $1
            AND ($2::text IS NULL OR status = $2)
            AND ($3::uuid IS NULL OR assignee_id = $3)
            AND (
                $4::text IS NULL
                OR title ILIKE '%' || $4 || '%'
                OR COALESCE(description, '') ILIKE '%' || $4 || '%'
            )
        ORDER BY key_seq DESC
        LIMIT $5
        OFFSET $6
        "#,
        project_id,
        status_filter,
        assignee_filter,
        q_filter,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| IssueSummaryRow {
            id: r.id,
            key_seq: r.key_seq,
            title: r.title,
            status: r.status,
        })
        .collect())
}

pub struct IssueSummaryRow {
    pub id: Uuid,
    pub key_seq: i32,
    pub title: String,
    pub status: String,
}

pub async fn count_issues_for_project_filtered(
    pool: &PgPool,
    project_id: Uuid,
    status_filter: Option<String>,
    assignee_filter: Option<Uuid>,
    q_filter: Option<String>,
) -> Result<i64, sqlx::Error> {
    let c = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM issues
        WHERE project_id = $1
            AND ($2::text IS NULL OR status = $2)
            AND ($3::uuid IS NULL OR assignee_id = $3)
            AND (
                $4::text IS NULL
                OR title ILIKE '%' || $4 || '%'
                OR COALESCE(description, '') ILIKE '%' || $4 || '%'
            )
        "#,
        project_id,
        status_filter,
        assignee_filter,
        q_filter
    )
    .fetch_one(pool)
    .await?;
    Ok(c)
}

pub async fn list_board_issues_by_key_seq(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<IssueSummaryRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT id, key_seq, title, status
        FROM issues
        WHERE project_id = $1
        ORDER BY key_seq DESC
        "#,
        project_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| IssueSummaryRow {
            id: r.id,
            key_seq: r.key_seq,
            title: r.title,
            status: r.status,
        })
        .collect())
}

pub async fn list_board_issues_by_board_order(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<IssueSummaryRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT id, key_seq, title, status
        FROM issues
        WHERE project_id = $1
        ORDER BY board_order ASC, key_seq DESC
        "#,
        project_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| IssueSummaryRow {
            id: r.id,
            key_seq: r.key_seq,
            title: r.title,
            status: r.status,
        })
        .collect())
}
