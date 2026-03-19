use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::{
    auth_user::AuthUser, error::AppError, services::projects as projects_service, state::AppState,
};

pub async fn list_projects(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(query): Query<projects_service::ListMyProjectsQuery>,
) -> Result<(StatusCode, Json<projects_service::ListMyProjectsResponse>), AppError> {
    let res = projects_service::list_projects(&state.db, user_id, query).await?;
    Ok((StatusCode::OK, Json(res)))
}

pub async fn get_project(
    Path((org_slug, project_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<(StatusCode, Json<projects_service::GetProjectResponse>), AppError> {
    let res = projects_service::get_project(&state.db, user_id, &org_slug, &project_key).await?;
    Ok((StatusCode::OK, Json(res)))
}

pub async fn create_project(
    Path(org_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<projects_service::CreateProjectRequest>,
) -> Result<(StatusCode, Json<projects_service::CreateProjectResponse>), AppError> {
    let res = projects_service::create_project(&state.db, org_id, user_id, req).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

pub async fn list_project_members(
    Path((org_slug, project_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<(StatusCode, Json<projects_service::ListProjectMembersResponse>), AppError> {
    let res =
        projects_service::list_project_members(&state.db, user_id, &org_slug, &project_key)
            .await?;
    Ok((StatusCode::OK, Json(res)))
}
