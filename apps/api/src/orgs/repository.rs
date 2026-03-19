use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

pub async fn is_org_member(
    pool: &PgPool,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let found = sqlx::query_scalar::<_, i32>(
        r#"SELECT 1 FROM org_members WHERE org_id = $1 AND user_id = $2"#,
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(found.is_some())
}

pub async fn find_org_id_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM organizations WHERE slug = $1")
        .bind(slug)
        .fetch_optional(pool)
        .await
}

pub async fn insert_organization(
    tx: &mut Transaction<'_, Postgres>,
    name: &str,
    slug: &str,
    created_by: Uuid,
) -> Result<(Uuid, String, String), sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO organizations (name, slug, created_by)
        VALUES ($1, $2, $3)
        RETURNING id, name, slug
        "#,
        name,
        slug,
        created_by
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok((row.id, row.name, row.slug))
}

pub async fn insert_org_admin_member(
    tx: &mut Transaction<'_, Postgres>,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO org_members (org_id, user_id, org_role)
        VALUES ($1, $2, 'org_admin')
        "#,
        org_id,
        user_id
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
