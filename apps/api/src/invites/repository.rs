use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

pub async fn is_org_admin(
    pool: &PgPool,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let found = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT 1
        FROM org_members
        WHERE org_id = $1 AND user_id = $2 AND org_role = 'org_admin'
        "#,
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(found.is_some())
}

pub async fn insert_invite(
    pool: &PgPool,
    org_id: Uuid,
    email: &str,
    invited_role: &str,
    token_hash: &str,
    expires_at: DateTime<Utc>,
    invited_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO user_invites (org_id, email, invited_role, token_hash, expires_at, invited_by)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        org_id,
        email,
        invited_role,
        token_hash,
        expires_at,
        invited_by
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
pub struct InvitePreviewRow {
    pub email: String,
    pub invited_role: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub org_name: String,
}

pub async fn find_invite_preview_by_hash(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<InvitePreviewRow>, sqlx::Error> {
    sqlx::query_as::<_, InvitePreviewRow>(
        r#"
        SELECT ui.email, ui.invited_role, ui.expires_at, ui.used_at, o.name as org_name
        FROM user_invites ui
        JOIN organizations o ON o.id = ui.org_id
        WHERE ui.token_hash = $1
        "#,
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
}

#[derive(sqlx::FromRow)]
pub struct LockedInviteRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub email: String,
    pub invited_role: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

pub async fn fetch_invite_for_update(
    tx: &mut Transaction<'_, Postgres>,
    token_hash: &str,
) -> Result<Option<LockedInviteRow>, sqlx::Error> {
    sqlx::query_as::<_, LockedInviteRow>(
        r#"
        SELECT id, org_id, email, invited_role, expires_at, used_at
        FROM user_invites
        WHERE token_hash = $1
        FOR UPDATE
        "#,
    )
    .bind(token_hash)
    .fetch_optional(&mut **tx)
    .await
}

#[derive(sqlx::FromRow)]
pub struct ExistingUserAuthRow {
    pub id: Uuid,
    pub password_hash: Option<String>,
}

pub async fn find_active_user_by_email_tx(
    tx: &mut Transaction<'_, Postgres>,
    email: &str,
) -> Result<Option<ExistingUserAuthRow>, sqlx::Error> {
    sqlx::query_as::<_, ExistingUserAuthRow>(
        r#"SELECT id, password_hash FROM users WHERE email = $1 AND is_active = true"#,
    )
    .bind(email)
    .fetch_optional(&mut **tx)
    .await
}

pub async fn insert_minimal_user_tx(
    tx: &mut Transaction<'_, Postgres>,
    email: &str,
    name: &str,
) -> Result<Uuid, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO users (email, name, platform_role)
        VALUES ($1, $2, 'user')
        RETURNING id
        "#,
        email,
        name
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(row.id)
}

pub async fn set_user_password_and_name_tx(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    password_hash: &str,
    name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE users SET password_hash = $1, name = $2 WHERE id = $3"#,
        password_hash,
        name,
        user_id
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn insert_org_member_on_conflict_nothing_tx(
    tx: &mut Transaction<'_, Postgres>,
    org_id: Uuid,
    user_id: Uuid,
    org_role: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO org_members (org_id, user_id, org_role)
        VALUES ($1, $2, $3)
        ON CONFLICT (org_id, user_id) DO NOTHING
        "#,
        org_id,
        user_id,
        org_role
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn mark_invite_used_tx(
    tx: &mut Transaction<'_, Postgres>,
    invite_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE user_invites SET used_at = now() WHERE id = $1"#,
        invite_id
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
