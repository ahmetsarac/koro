use sqlx::{Postgres, Transaction};
use uuid::Uuid;

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
