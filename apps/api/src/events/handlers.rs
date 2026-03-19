use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    auth::user::AuthUser,
    core::state::AppState,
    error::AppError,
    events::service as events_service,
};

pub async fn list_issue_events(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<(StatusCode, Json<events_service::ListEventsResponse>), AppError> {
    let res = events_service::list_issue_events(&state.db, &org_slug, &issue_key, user_id).await?;
    Ok((StatusCode::OK, Json(res)))
}
