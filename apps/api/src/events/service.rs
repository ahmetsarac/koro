use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::AppError,
    events::models::*,
    events::repository as events_repo,
    issues::models::parse_issue_key,
    orgs::repository as orgs_repo,
};

pub async fn list_issue_events(
    pool: &PgPool,
    org_slug: &str,
    issue_key: &str,
    user_id: Uuid,
) -> Result<ListEventsResponse, AppError> {
    let (project_key, key_seq) =
        parse_issue_key(issue_key).ok_or(AppError::BadRequest(Some("invalid issue key")))?;

    let org_id = orgs_repo::find_org_id_by_slug(pool, org_slug)
        .await
        .map_err(|e| {
            tracing::error!(?e, "list_issue_events org resolve");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let issue_row = events_repo::resolve_issue_in_org(pool, org_id, project_key, key_seq)
        .await
        .map_err(|e| {
            tracing::error!(?e, "list_issue_events issue resolve");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let is_member = events_repo::is_project_member(pool, issue_row.project_id, user_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "list_issue_events membership check");
            AppError::Internal
        })?;

    if !is_member {
        return Err(AppError::Forbidden);
    }

    let rows = events_repo::list_events_for_issue(pool, issue_row.issue_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "list_issue_events query");
            AppError::Internal
        })?;

    let items = rows
        .into_iter()
        .map(|r| EventItem {
            event_id: r.id,
            event_type: r.event_type,
            created_at: r.created_at,
            actor: match (r.actor_id, r.actor_name, r.actor_email) {
                (Some(aid), Some(name), Some(email)) => Some(EventActor {
                    user_id: aid,
                    name,
                    email,
                }),
                (Some(aid), Some(name), None) => Some(EventActor {
                    user_id: aid,
                    name,
                    email: "".to_string(),
                }),
                (Some(aid), None, Some(email)) => Some(EventActor {
                    user_id: aid,
                    name: "Unknown".to_string(),
                    email,
                }),
                (Some(aid), None, None) => Some(EventActor {
                    user_id: aid,
                    name: "Unknown".to_string(),
                    email: "".to_string(),
                }),
                (None, _, _) => None,
            },
            payload: r.payload,
        })
        .collect();

    Ok(ListEventsResponse { items })
}
