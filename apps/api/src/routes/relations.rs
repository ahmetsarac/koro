use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::{auth_user::AuthUser, state::AppState};

#[derive(Deserialize)]
pub struct CreateRelationRequest {
    pub relation_type: String,
    pub target_issue_key: String,
}

#[derive(Serialize)]
pub struct CreateRelationResponse {
    pub relation_id: uuid::Uuid,
}

fn parse_issue_key(key: &str) -> Option<(&str, i32)> {
    let (project_key, seq_str) = key.split_once('-')?;
    let seq = seq_str.parse::<i32>().ok()?;
    Some((project_key, seq))
}

pub async fn create_relation(
    Path((org_slug, source_issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateRelationRequest>,
) -> impl IntoResponse {
    // validate relation_type
    let allowed = ["blocks", "relates_to", "duplicate"];
    if !allowed.contains(&req.relation_type.as_str()) {
        return (StatusCode::BAD_REQUEST, "invalid relation_type").into_response();
    }

    // parse keys
    let (source_project_key, source_seq) = match parse_issue_key(&source_issue_key) {
        Some(v) => v,
        None => return (StatusCode::BAD_REQUEST, "invalid source issue key").into_response(),
    };

    let (target_project_key, target_seq) = match parse_issue_key(&req.target_issue_key) {
        Some(v) => v,
        None => return (StatusCode::BAD_REQUEST, "invalid target issue key").into_response(),
    };

    // org resolve
    let org_id = match sqlx::query_scalar::<_, uuid::Uuid>(
        r#"SELECT id FROM organizations WHERE slug = $1"#,
    )
    .bind(&org_slug)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("create_relation org resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // authz: org member mı?
    let is_member = match sqlx::query_scalar::<_, i32>(
        r#"SELECT 1 FROM org_members WHERE org_id = $1 AND user_id = $2"#,
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("create_relation member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    // source issue resolve (org içinde)
    let source_issue_id = match sqlx::query_scalar::<_, uuid::Uuid>(
        r#"
        SELECT i.id
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        WHERE p.org_id = $1 AND p.project_key = $2 AND i.key_seq = $3
        "#,
    )
    .bind(org_id)
    .bind(source_project_key)
    .bind(source_seq)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("create_relation source resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // target issue resolve (org içinde, cross-project ok)
    let target_issue_id = match sqlx::query_scalar::<_, uuid::Uuid>(
        r#"
        SELECT i.id
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        WHERE p.org_id = $1 AND p.project_key = $2 AND i.key_seq = $3
        "#,
    )
    .bind(org_id)
    .bind(target_project_key)
    .bind(target_seq)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("create_relation target resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // insert
    let rec = match sqlx::query!(
        r#"
        INSERT INTO issue_relations (org_id, source_issue_id, target_issue_id, relation_type)
        VALUES ($1,$2,$3,$4)
        RETURNING id
        "#,
        org_id,
        source_issue_id,
        target_issue_id,
        req.relation_type
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("create_relation insert error: {e:?}");
            return StatusCode::BAD_REQUEST.into_response(); // unique/self-check vb.
        }
    };

    let payload = serde_json::json!({
        "relation_type": req.relation_type,
        "target_issue_key": req.target_issue_key,
    });

    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO issue_events (org_id, issue_id, actor_id, event_type, payload)
        VALUES ($1,$2,$3,$4,$5)
        "#,
        org_id,
        source_issue_id,
        user_id,
        "relation_added",
        payload
    )
    .execute(&state.db)
    .await
    {
        eprintln!("relation_added event insert error: {e:?}");
    }

    (
        StatusCode::CREATED,
        Json(CreateRelationResponse {
            relation_id: rec.id,
        }),
    )
        .into_response()
}

#[derive(Serialize)]
pub struct RelationItem {
    pub issue_key: String, // e.g. APP-2
    pub title: String,
}

#[derive(Serialize)]
pub struct GetRelationsResponse {
    pub blocking: Vec<RelationItem>,
    pub blocked_by: Vec<RelationItem>,
    pub related: Vec<RelationItem>,
    pub duplicates: Vec<RelationItem>,
    pub duplicated_by: Vec<RelationItem>,
}

pub async fn get_relations(
    Path((org_slug, source_issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl IntoResponse {
    let (source_project_key, source_seq) = match parse_issue_key(&source_issue_key) {
        Some(v) => v,
        None => return (StatusCode::BAD_REQUEST, "invalid issue key").into_response(),
    };

    // org resolve
    let org_id = match sqlx::query_scalar::<_, uuid::Uuid>(
        r#"SELECT id FROM organizations WHERE slug = $1"#,
    )
    .bind(&org_slug)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("get_relations org resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // authz: org member
    let is_member = match sqlx::query_scalar::<_, i32>(
        r#"SELECT 1 FROM org_members WHERE org_id = $1 AND user_id = $2"#,
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("get_relations member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    // source issue id resolve
    let source_issue_id = match sqlx::query_scalar::<_, uuid::Uuid>(
        r#"
        SELECT i.id
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        WHERE p.org_id = $1 AND p.project_key = $2 AND i.key_seq = $3
        "#,
    )
    .bind(org_id)
    .bind(source_project_key)
    .bind(source_seq)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("get_relations source resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // outgoing: source_issue_id -> others
    let outgoing = match sqlx::query!(
        r#"
        SELECT
          r.relation_type,
          p.project_key as other_project_key,
          i.key_seq as other_key_seq,
          i.title as other_title
        FROM issue_relations r
        JOIN issues i ON i.id = r.target_issue_id
        JOIN projects p ON p.id = i.project_id
        WHERE r.org_id = $1 AND r.source_issue_id = $2
        "#,
        org_id,
        source_issue_id
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("get_relations outgoing error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // incoming: others -> source_issue_id
    let incoming = match sqlx::query!(
        r#"
        SELECT
          r.relation_type,
          p.project_key as other_project_key,
          i.key_seq as other_key_seq,
          i.title as other_title
        FROM issue_relations r
        JOIN issues i ON i.id = r.source_issue_id
        JOIN projects p ON p.id = i.project_id
        WHERE r.org_id = $1 AND r.target_issue_id = $2
        "#,
        org_id,
        source_issue_id
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("get_relations incoming error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut blocking = Vec::new();
    let mut related = Vec::new();
    let mut duplicates = Vec::new();

    for r in outgoing {
        let item = RelationItem {
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
            issue_key: format!("{}-{}", r.other_project_key, r.other_key_seq),
            title: r.other_title,
        };
        match r.relation_type.as_str() {
            "blocks" => blocked_by.push(item),
            "duplicate" => duplicated_by.push(item),
            "relates_to" => {
                // relates_to is symmetric-ish; show it in "related" too
                related.push(item);
            }
            _ => {}
        }
    }

    (
        StatusCode::OK,
        Json(GetRelationsResponse {
            blocking,
            blocked_by,
            related,
            duplicates,
            duplicated_by,
        }),
    )
        .into_response()
}

pub async fn delete_relation(
    Path((org_slug, _issue_key, relation_id)): Path<(String, String, uuid::Uuid)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl IntoResponse {
    // org resolve
    let org_id = match sqlx::query_scalar::<_, uuid::Uuid>(
        r#"SELECT id FROM organizations WHERE slug = $1"#,
    )
    .bind(&org_slug)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("delete_relation org resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // authz: org member
    let is_member = match sqlx::query_scalar::<_, i32>(
        r#"SELECT 1 FROM org_members WHERE org_id = $1 AND user_id = $2"#,
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("delete_relation member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    let rel = match sqlx::query!(
        r#"
        SELECT source_issue_id, target_issue_id, relation_type
        FROM issue_relations
        WHERE id = $1 AND org_id = $2
        "#,
        relation_id,
        org_id
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("delete_relation select error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // relation org check + delete
    let res = match sqlx::query!(
        r#"DELETE FROM issue_relations WHERE id = $1 AND org_id = $2"#,
        relation_id,
        org_id
    )
    .execute(&state.db)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("delete_relation delete error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if res.rows_affected() == 0 {
        return StatusCode::NOT_FOUND.into_response();
    }

    let payload = serde_json::json!({
        "relation_id": relation_id,
        "relation_type": rel.relation_type,
    });

    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO issue_events (org_id, issue_id, actor_id, event_type, payload)
        VALUES ($1,$2,$3,$4,$5)
        "#,
        org_id,
        rel.source_issue_id,
        user_id,
        "relation_removed",
        payload
    )
    .execute(&state.db)
    .await
    {
        eprintln!("relation_removed event insert error: {e:?}");
    }

    StatusCode::NO_CONTENT.into_response()
}
