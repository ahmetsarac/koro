use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct UserProfileRow {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}

#[derive(sqlx::FromRow)]
pub struct OrgMembershipRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub org_role: String,
}

pub async fn find_user_by_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<UserProfileRow>, sqlx::Error> {
    sqlx::query_as::<_, UserProfileRow>(
        r#"SELECT id, email, name FROM users WHERE id = $1"#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_org_memberships(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<OrgMembershipRow>, sqlx::Error> {
    sqlx::query_as::<_, OrgMembershipRow>(
        r#"
        SELECT o.id, o.name, o.slug, om.org_role
        FROM organizations o
        JOIN org_members om ON om.org_id = o.id
        WHERE om.user_id = $1
        ORDER BY o.name ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn count_users_tx(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(&mut **tx)
        .await
}

#[derive(sqlx::FromRow)]
pub struct LoginUserRow {
    pub id: Uuid,
    pub password_hash: Option<String>,
}

pub async fn find_active_user_for_login(
    pool: &PgPool,
    email: &str,
) -> Result<Option<LoginUserRow>, sqlx::Error> {
    sqlx::query_as::<_, LoginUserRow>(
        r#"SELECT id, password_hash FROM users WHERE email = $1 AND is_active = true"#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}

pub async fn insert_signup_user(
    pool: &PgPool,
    email: &str,
    name: &str,
    password_hash: &str,
) -> Result<Uuid, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO users (email, name, password_hash, platform_role)
        VALUES ($1, $2, $3, 'user')
        RETURNING id
        "#,
        email,
        name,
        password_hash
    )
    .fetch_one(pool)
    .await?;
    Ok(row.id)
}

pub async fn find_active_user_by_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>(
        r#"SELECT id FROM users WHERE id = $1 AND is_active = true"#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn insert_platform_admin_user(
    tx: &mut Transaction<'_, Postgres>,
    email: &str,
    name: &str,
    password_hash: &str,
) -> Result<(Uuid, String), sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO users (email, name, password_hash, platform_role)
        VALUES ($1, $2, $3, 'platform_admin')
        RETURNING id, email
        "#,
        email,
        name,
        password_hash
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok((row.id, row.email))
}
