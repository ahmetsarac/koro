use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn insert_relation(
    pool: &PgPool,
    org_id: Uuid,
    source_issue_id: Uuid,
    target_issue_id: Uuid,
    relation_type: &str,
) -> Result<Uuid, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO issue_relations (org_id, source_issue_id, target_issue_id, relation_type)
        VALUES ($1,$2,$3,$4)
        RETURNING id
        "#,
        org_id,
        source_issue_id,
        target_issue_id,
        relation_type
    )
    .fetch_one(pool)
    .await?;
    Ok(row.id)
}

pub async fn insert_relation_added_event(
    pool: &PgPool,
    org_id: Uuid,
    source_issue_id: Uuid,
    actor_id: Uuid,
    payload: Value,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO issue_events (org_id, issue_id, actor_id, event_type, payload)
        VALUES ($1,$2,$3,$4,$5)
        "#,
        org_id,
        source_issue_id,
        actor_id,
        "relation_added",
        payload
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
pub struct RelationEdgeRow {
    pub relation_id: Uuid,
    pub relation_type: String,
    pub other_project_key: String,
    pub other_key_seq: i32,
    pub other_title: String,
}

pub async fn list_outgoing_relations(
    pool: &PgPool,
    org_id: Uuid,
    source_issue_id: Uuid,
) -> Result<Vec<RelationEdgeRow>, sqlx::Error> {
    sqlx::query_as::<_, RelationEdgeRow>(
        r#"
        SELECT
          r.id as relation_id,
          r.relation_type,
          p.project_key as other_project_key,
          i.key_seq as other_key_seq,
          i.title as other_title
        FROM issue_relations r
        JOIN issues i ON i.id = r.target_issue_id
        JOIN projects p ON p.id = i.project_id
        WHERE r.org_id = $1 AND r.source_issue_id = $2
        "#,
    )
    .bind(org_id)
    .bind(source_issue_id)
    .fetch_all(pool)
    .await
}

pub async fn list_incoming_relations(
    pool: &PgPool,
    org_id: Uuid,
    target_issue_id: Uuid,
) -> Result<Vec<RelationEdgeRow>, sqlx::Error> {
    sqlx::query_as::<_, RelationEdgeRow>(
        r#"
        SELECT
          r.id as relation_id,
          r.relation_type,
          p.project_key as other_project_key,
          i.key_seq as other_key_seq,
          i.title as other_title
        FROM issue_relations r
        JOIN issues i ON i.id = r.source_issue_id
        JOIN projects p ON p.id = i.project_id
        WHERE r.org_id = $1 AND r.target_issue_id = $2
        "#,
    )
    .bind(org_id)
    .bind(target_issue_id)
    .fetch_all(pool)
    .await
}

#[derive(sqlx::FromRow)]
pub struct RelationRecord {
    pub source_issue_id: Uuid,
    pub relation_type: String,
}

pub async fn find_relation_in_org(
    pool: &PgPool,
    org_id: Uuid,
    relation_id: Uuid,
) -> Result<Option<RelationRecord>, sqlx::Error> {
    sqlx::query_as::<_, RelationRecord>(
        r#"
        SELECT source_issue_id, relation_type
        FROM issue_relations
        WHERE id = $1 AND org_id = $2
        "#,
    )
    .bind(relation_id)
    .bind(org_id)
    .fetch_optional(pool)
    .await
}

pub async fn delete_relation_in_org(
    pool: &PgPool,
    org_id: Uuid,
    relation_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let r = sqlx::query!(
        r#"DELETE FROM issue_relations WHERE id = $1 AND org_id = $2"#,
        relation_id,
        org_id
    )
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}

pub async fn insert_relation_removed_event(
    pool: &PgPool,
    org_id: Uuid,
    source_issue_id: Uuid,
    actor_id: Uuid,
    payload: Value,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO issue_events (org_id, issue_id, actor_id, event_type, payload)
        VALUES ($1,$2,$3,$4,$5)
        "#,
        org_id,
        source_issue_id,
        actor_id,
        "relation_removed",
        payload
    )
    .execute(pool)
    .await?;
    Ok(())
}
