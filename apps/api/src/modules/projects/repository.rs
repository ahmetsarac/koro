use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

pub async fn count_member_projects(
    pool: &PgPool,
    user_id: Uuid,
    search_pattern: &str,
    filter_org_id: Option<Uuid>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(DISTINCT p.id)
        FROM projects p
        JOIN project_members pm ON pm.project_id = p.id
        WHERE pm.user_id = $1
          AND (LOWER(p.name) LIKE $2 OR LOWER(p.project_key) LIKE $2)
          AND ($3::uuid IS NULL OR p.org_id = $3)
        "#,
    )
    .bind(user_id)
    .bind(search_pattern)
    .bind(filter_org_id)
    .fetch_one(pool)
    .await
}

#[derive(sqlx::FromRow)]
pub struct ProjectWithOrgRow {
    pub id: Uuid,
    pub project_key: String,
    pub name: String,
    pub description: Option<String>,
    pub org_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub org_name: String,
    pub org_slug: String,
    pub my_role: String,
    pub issue_count: i64,
    pub member_count: i64,
}

pub async fn list_member_projects(
    pool: &PgPool,
    user_id: Uuid,
    search_pattern: &str,
    filter_org_id: Option<Uuid>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ProjectWithOrgRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectWithOrgRow>(
        r#"
        SELECT 
            p.id,
            p.project_key,
            p.name,
            p.description,
            p.org_id,
            p.created_at,
            o.name AS org_name,
            o.slug AS org_slug,
            pm.project_role AS my_role,
            (SELECT COUNT(*) FROM issues WHERE project_id = p.id) AS issue_count,
            (SELECT COUNT(*) FROM project_members WHERE project_id = p.id) AS member_count
        FROM projects p
        JOIN project_members pm ON pm.project_id = p.id AND pm.user_id = $1
        JOIN organizations o ON o.id = p.org_id
        WHERE (LOWER(p.name) LIKE $2 OR LOWER(p.project_key) LIKE $2)
          AND ($5::uuid IS NULL OR p.org_id = $5)
        ORDER BY p.name ASC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(user_id)
    .bind(search_pattern)
    .bind(limit)
    .bind(offset)
    .bind(filter_org_id)
    .fetch_all(pool)
    .await
}

pub async fn get_project_for_member(
    pool: &PgPool,
    user_id: Uuid,
    org_id: Uuid,
    project_key: &str,
) -> Result<Option<ProjectWithOrgRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectWithOrgRow>(
        r#"
        SELECT 
            p.id,
            p.project_key,
            p.name,
            p.description,
            p.org_id,
            p.created_at,
            o.name AS org_name,
            o.slug AS org_slug,
            pm.project_role AS my_role,
            (SELECT COUNT(*) FROM issues WHERE project_id = p.id) AS issue_count,
            (SELECT COUNT(*) FROM project_members WHERE project_id = p.id) AS member_count
        FROM projects p
        JOIN project_members pm ON pm.project_id = p.id AND pm.user_id = $1
        JOIN organizations o ON o.id = p.org_id
        WHERE p.org_id = $2 AND UPPER(p.project_key) = UPPER($3)
        "#,
    )
    .bind(user_id)
    .bind(org_id)
    .bind(project_key)
    .fetch_optional(pool)
    .await
}

pub async fn insert_project_tx(
    tx: &mut Transaction<'_, Postgres>,
    org_id: Uuid,
    project_key: &str,
    name: &str,
    description: Option<&str>,
) -> Result<(Uuid, String, String), sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO projects (org_id, project_key, name, description)
        VALUES ($1,$2,$3,$4)
        RETURNING id, project_key, name
        "#,
        org_id,
        project_key,
        name,
        description
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok((row.id, row.project_key, row.name))
}

pub async fn insert_project_manager_tx(
    tx: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO project_members (project_id, user_id, project_role)
        VALUES ($1,$2,'project_manager')
        "#,
        project_id,
        user_id
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn update_project_name(
    pool: &PgPool,
    project_id: Uuid,
    name: &str,
) -> Result<String, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        r#"UPDATE projects SET name = $1, updated_at = now() WHERE id = $2 RETURNING project_key"#,
    )
    .bind(name)
    .bind(project_id)
    .fetch_one(pool)
    .await
}

pub async fn find_project_id_in_org(
    pool: &PgPool,
    org_id: Uuid,
    project_key_upper: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM projects WHERE org_id = $1 AND UPPER(project_key) = $2",
    )
    .bind(org_id)
    .bind(project_key_upper)
    .fetch_optional(pool)
    .await
}

#[derive(sqlx::FromRow)]
pub struct ProjectMemberRow {
    pub user_id: Uuid,
    pub project_role: String,
    pub email: String,
    pub name: String,
}

pub async fn list_project_members_for_project(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<ProjectMemberRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectMemberRow>(
        r#"
        SELECT pm.user_id, pm.project_role, u.email, u.name
        FROM project_members pm
        JOIN users u ON u.id = pm.user_id
        WHERE pm.project_id = $1
        ORDER BY u.name ASC
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}
