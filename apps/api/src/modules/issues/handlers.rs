use axum::{
    Json,
    extract::{Path, Query, State},
};
use std::collections::HashMap;

use crate::modules::{auth::user::AuthUser, core::state::AppState};

use super::{
    models::{
        AssignIssueRequest, CreateIssueRequest, ListMyIssuesQuery, UpdateIssueBoardPositionRequest,
        UpdateIssueRequest, UpdateIssueStatusRequest,
    },
    service,
};

pub async fn create_issue(
    Path(project_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateIssueRequest>,
) -> impl axum::response::IntoResponse {
    service::create_issue(&state.db, project_id, user_id, req).await
}

pub async fn list_my_issues(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(query): Query<ListMyIssuesQuery>,
) -> impl axum::response::IntoResponse {
    service::list_my_issues(&state.db, user_id, query).await
}

pub async fn list_issues(
    Path(project_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(q): Query<HashMap<String, String>>,
) -> impl axum::response::IntoResponse {
    service::list_issues(&state.db, project_id, user_id, q).await
}

pub async fn list_project_issues_by_key(
    Path((org_slug, project_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(q): Query<HashMap<String, String>>,
) -> impl axum::response::IntoResponse {
    service::list_project_issues_by_key(&state.db, org_slug, project_key, user_id, q).await
}

pub async fn update_issue_status(
    Path(issue_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<UpdateIssueStatusRequest>,
) -> impl axum::response::IntoResponse {
    service::update_issue_status(&state.db, issue_id, user_id, req).await
}

pub async fn update_issue(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<UpdateIssueRequest>,
) -> impl axum::response::IntoResponse {
    service::update_issue(&state.db, org_slug, issue_key, user_id, req).await
}

pub async fn get_board(
    Path(project_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl axum::response::IntoResponse {
    service::get_board(&state.db, project_id, user_id).await
}

pub async fn get_board_by_key(
    Path((org_slug, project_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl axum::response::IntoResponse {
    service::get_board_by_key(&state.db, org_slug, project_key, user_id).await
}

pub async fn update_issue_board_position(
    Path(issue_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<UpdateIssueBoardPositionRequest>,
) -> impl axum::response::IntoResponse {
    service::update_issue_board_position(&state.db, issue_id, user_id, req).await
}

pub async fn get_issue(
    Path(issue_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl axum::response::IntoResponse {
    service::get_issue(&state.db, issue_id, user_id).await
}

pub async fn get_issue_by_key(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl axum::response::IntoResponse {
    service::get_issue_by_key(&state.db, org_slug, issue_key, user_id).await
}

pub async fn assign_issue(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(_actor_id): AuthUser,
    Json(req): Json<AssignIssueRequest>,
) -> impl axum::response::IntoResponse {
    service::assign_issue(&state.db, org_slug, issue_key, req).await
}

pub async fn assign_me(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(actor_id): AuthUser,
) -> impl axum::response::IntoResponse {
    service::assign_me(&state.db, org_slug, issue_key, actor_id).await
}

pub async fn unassign_issue(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(actor_id): AuthUser,
) -> impl axum::response::IntoResponse {
    service::unassign_issue(&state.db, org_slug, issue_key, actor_id).await
}
