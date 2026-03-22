use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::{
    core::AppError,
    events::repository as events_repo,
    issues::models::parse_issue_key,
    orgs::repository as orgs_repo,
    relations::models::*,
    relations::repository as relations_repo,
};

const ALLOWED_RELATION_TYPES: [&str; 3] = ["blocks", "relates_to", "duplicate"];

pub async fn create_relation(
    pool: &PgPool,
    org_slug: &str,
    source_issue_key: &str,
    user_id: Uuid,
    req: CreateRelationRequest,
) -> Result<CreateRelationResponse, AppError> {
    if !ALLOWED_RELATION_TYPES.contains(&req.relation_type.as_str()) {
        return Err(AppError::BadRequest(Some("invalid relation_type")));
    }

    let (source_project_key, source_seq) = parse_issue_key(source_issue_key)
        .ok_or(AppError::BadRequest(Some("invalid source issue key")))?;

    let (target_project_key, target_seq) = parse_issue_key(&req.target_issue_key)
        .ok_or(AppError::BadRequest(Some("invalid target issue key")))?;

    let org_id = orgs_repo::find_org_id_by_slug(pool, org_slug)
        .await
        .map_err(|e| {
            tracing::error!(?e, "create_relation org resolve");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let is_member = orgs_repo::is_org_member(pool, org_id, user_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "create_relation org membership check");
            AppError::Internal
        })?;

    if !is_member {
        return Err(AppError::Forbidden);
    }

    let source_issue = events_repo::resolve_issue_in_org(pool, org_id, source_project_key, source_seq)
        .await
        .map_err(|e| {
            tracing::error!(?e, "create_relation source resolve");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let target_issue = events_repo::resolve_issue_in_org(pool, org_id, target_project_key, target_seq)
        .await
        .map_err(|e| {
            tracing::error!(?e, "create_relation target resolve");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let source_issue_id = source_issue.issue_id;
    let target_issue_id = target_issue.issue_id;

    let relation_id = relations_repo::insert_relation(
        pool,
        org_id,
        source_issue_id,
        target_issue_id,
        &req.relation_type,
    )
    .await
    .map_err(|e| {
        tracing::error!(?e, "create_relation insert");
        AppError::BadRequest(None)
    })?;

    let payload = serde_json::json!({
        "relation_type": req.relation_type,
        "target_issue_key": req.target_issue_key,
    });

    if let Err(e) = relations_repo::insert_relation_added_event(
        pool,
        org_id,
        source_issue_id,
        user_id,
        payload,
    )
    .await
    {
        tracing::error!(?e, "relation_added event insert");
    }

    Ok(CreateRelationResponse { relation_id })
}

pub async fn get_relations(
    pool: &PgPool,
    org_slug: &str,
    source_issue_key: &str,
    user_id: Uuid,
) -> Result<GetRelationsResponse, AppError> {
    let (source_project_key, source_seq) =
        parse_issue_key(source_issue_key).ok_or(AppError::BadRequest(Some("invalid issue key")))?;

    let org_id = orgs_repo::find_org_id_by_slug(pool, org_slug)
        .await
        .map_err(|e| {
            tracing::error!(?e, "get_relations org resolve");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let is_member = orgs_repo::is_org_member(pool, org_id, user_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "get_relations org membership check");
            AppError::Internal
        })?;

    if !is_member {
        return Err(AppError::Forbidden);
    }

    let source_issue = events_repo::resolve_issue_in_org(pool, org_id, source_project_key, source_seq)
        .await
        .map_err(|e| {
            tracing::error!(?e, "get_relations source resolve");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let source_issue_id = source_issue.issue_id;

    let outgoing = relations_repo::list_outgoing_relations(pool, org_id, source_issue_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "get_relations outgoing");
            AppError::Internal
        })?;

    let incoming = relations_repo::list_incoming_relations(pool, org_id, source_issue_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "get_relations incoming");
            AppError::Internal
        })?;

    let mut blocking = Vec::new();
    let mut related = Vec::new();
    let mut duplicates = Vec::new();

    for r in outgoing {
        let item = RelationItem {
            relation_id: r.relation_id,
            issue_key: format!("{}-{}", r.other_project_key, r.other_key_seq),
            title: r.other_title,
        };
        match r.relation_type.as_str() {
            "blocks" => blocking.push(item),
            "relates_to" => related.push(item),
            "duplicate" => duplicates.push(item),
            _ => {}
        }
    }

    let mut blocked_by = Vec::new();
    let mut duplicated_by = Vec::new();

    for r in incoming {
        let item = RelationItem {
            relation_id: r.relation_id,
            issue_key: format!("{}-{}", r.other_project_key, r.other_key_seq),
            title: r.other_title,
        };
        match r.relation_type.as_str() {
            "blocks" => blocked_by.push(item),
            "duplicate" => duplicated_by.push(item),
            "relates_to" => related.push(item),
            _ => {}
        }
    }

    Ok(GetRelationsResponse {
        blocking,
        blocked_by,
        related,
        duplicates,
        duplicated_by,
    })
}

pub async fn delete_relation(
    pool: &PgPool,
    org_slug: &str,
    user_id: Uuid,
    relation_id: Uuid,
) -> Result<(), AppError> {
    let org_id = orgs_repo::find_org_id_by_slug(pool, org_slug)
        .await
        .map_err(|e| {
            tracing::error!(?e, "delete_relation org resolve");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let is_member = orgs_repo::is_org_member(pool, org_id, user_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "delete_relation org membership check");
            AppError::Internal
        })?;

    if !is_member {
        return Err(AppError::Forbidden);
    }

    let rel = relations_repo::find_relation_in_org(pool, org_id, relation_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "delete_relation select");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let affected = relations_repo::delete_relation_in_org(pool, org_id, relation_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "delete_relation delete");
            AppError::Internal
        })?;

    if affected == 0 {
        return Err(AppError::NotFound);
    }

    let payload = serde_json::json!({
        "relation_id": relation_id,
        "relation_type": rel.relation_type,
    });

    if let Err(e) = relations_repo::insert_relation_removed_event(
        pool,
        org_id,
        rel.source_issue_id,
        user_id,
        payload,
    )
    .await
    {
        tracing::error!(?e, "relation_removed event insert");
    }

    Ok(())
}
