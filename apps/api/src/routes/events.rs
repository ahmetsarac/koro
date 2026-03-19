use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    auth_user::AuthUser, error::AppError, services::events as events_service, state::AppState,
};

pub async fn list_issue_events(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<(StatusCode, Json<events_service::ListEventsResponse>), AppError> {
    let res = events_service::list_issue_events(&state.db, &org_slug, &issue_key, user_id).await?;
    Ok((StatusCode::OK, Json(res)))
}
