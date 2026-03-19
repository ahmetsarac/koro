use sqlx::PgPool;
use uuid::Uuid;

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
