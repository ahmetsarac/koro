use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::modules::{
    auth::user::AuthUser,
    core::{state::AppState, AppError},
    events::{
        models::ListEventsResponse,
        service as events_service,
    },
};

#[utoipa::path(
    get,
    path = "/orgs/{orgSlug}/issues/{issueKey}/events",
    tag = "events",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path),
        ("issueKey" = String, Path),
    ),
    responses(
        (status = 200, description = "Activity log", body = ListEventsResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn list_issue_events(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<(StatusCode, Json<ListEventsResponse>), AppError> {
    let res = events_service::list_issue_events(&state.db, &org_slug, &issue_key, user_id).await?;
    Ok((StatusCode::OK, Json(res)))
}
