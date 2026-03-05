use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::{auth_user::AuthUser, state::AppState};

fn parse_issue_key(key: &str) -> Option<(&str, i32)> {
    let (project_key, seq_str) = key.split_once('-')?;
    let seq = seq_str.parse::<i32>().ok()?;
    Some((project_key, seq))
}

#[derive(Serialize)]
pub struct EventItem {
    pub event_id: uuid::Uuid,
    pub event_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub actor: Option<EventActor>,
    pub payload: serde_json::Value,
}

#[derive(Serialize)]
pub struct EventActor {
    pub user_id: uuid::Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct ListEventsResponse {
    pub items: Vec<EventItem>,
}

pub async fn list_issue_events(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl IntoResponse {
    let (project_key, key_seq) = match parse_issue_key(&issue_key) {
        Some(v) => v,
        None => return (StatusCode::BAD_REQUEST, "invalid issue key").into_response(),
    };

    // org resolve
    let org_id = match sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT id FROM organizations WHERE slug = $1",
    )
    .bind(&org_slug)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("list_issue_events org resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // issue resolve + project_id (auth için)
    let issue_row = match sqlx::query!(
        r#"
        SELECT i.id as issue_id, i.project_id
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        WHERE p.org_id = $1 AND p.project_key = $2 AND i.key_seq = $3
        "#,
        org_id,
        project_key,
        key_seq
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("list_issue_events issue resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // authz: project member
    let is_member = match sqlx::query_scalar::<_, i32>(
        "SELECT 1 FROM project_members WHERE project_id = $1 AND user_id = $2",
    )
    .bind(issue_row.project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("list_issue_events membership check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    // list events (+ actor join)
    let rows = match sqlx::query!(
        r#"
        SELECT
          e.id,
          e.event_type,
          e.payload,
          e.created_at,
          e.actor_id as "actor_id?",
          u.name as "actor_name?",
          u.email as "actor_email?"
        FROM issue_events e
        LEFT JOIN users u ON u.id = e.actor_id
        WHERE e.issue_id = $1
        ORDER BY e.created_at DESC
        LIMIT 200
        "#,
        issue_row.issue_id
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("list_issue_events query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let items = rows
        .into_iter()
        .map(|r| EventItem {
            event_id: r.id,
            event_type: r.event_type,
            created_at: r.created_at,
            actor: match (r.actor_id, r.actor_name, r.actor_email) {
                (Some(aid), Some(name), Some(email)) => Some(EventActor { user_id: aid, name, email }),
                (Some(aid), Some(name), None) => Some(EventActor { user_id: aid, name, email: "".to_string() }),
                (Some(aid), None, Some(email)) => Some(EventActor { user_id: aid, name: "Unknown".to_string(), email }),
                (Some(aid), None, None) => Some(EventActor { user_id: aid, name: "Unknown".to_string(), email: "".to_string() }),
                (None, _, _) => None,
            },
            payload: r.payload,        
        })
        .collect();

    (StatusCode::OK, Json(ListEventsResponse { items })).into_response()
}
