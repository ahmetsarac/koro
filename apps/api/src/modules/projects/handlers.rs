use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::modules::{
    auth::user::AuthUser,
    core::{state::AppState, AppError},
    orgs::repository as orgs_repo,
    projects::{
        models::{
            CreateProjectRequest, CreateProjectResponse, GetProjectResponse,
            ListMyProjectsQuery, ListMyProjectsResponse, ListProjectMembersResponse,
            PatchProjectRequest, PatchProjectResponse,
        },
        service as projects_service,
    },
};

#[utoipa::path(
    get,
    path = "/projects",
    tag = "projects",
    security(("bearer_auth" = [])),
    params(ListMyProjectsQuery),
    responses(
        (status = 200, description = "My projects", body = ListMyProjectsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn list_projects(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(query): Query<ListMyProjectsQuery>,
) -> Result<(StatusCode, Json<ListMyProjectsResponse>), AppError> {
    let res = projects_service::list_projects(&state.db, user_id, query).await?;
    Ok((StatusCode::OK, Json(res)))
}

#[utoipa::path(
    get,
    path = "/orgs/{orgSlug}/projects/{projectKey}",
    tag = "projects",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path),
        ("projectKey" = String, Path),
    ),
    responses(
        (status = 200, description = "Project detail", body = GetProjectResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn get_project(
    Path((org_slug, project_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<(StatusCode, Json<GetProjectResponse>), AppError> {
    let res = projects_service::get_project(&state.db, user_id, &org_slug, &project_key).await?;
    Ok((StatusCode::OK, Json(res)))
}

#[utoipa::path(
    patch,
    path = "/orgs/{orgSlug}/projects/{projectKey}",
    tag = "projects",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path),
        ("projectKey" = String, Path),
    ),
    request_body = PatchProjectRequest,
    responses(
        (status = 200, description = "Project updated", body = PatchProjectResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn patch_project(
    Path((org_slug, project_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<PatchProjectRequest>,
) -> Result<(StatusCode, Json<PatchProjectResponse>), AppError> {
    let res = projects_service::patch_project(&state.db, user_id, &org_slug, &project_key, req).await?;
    Ok((StatusCode::OK, Json(res)))
}

#[utoipa::path(
    post,
    path = "/orgs/{orgSlug}/projects",
    tag = "projects",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path),
    ),
    request_body = CreateProjectRequest,
    responses(
        (status = 201, description = "Project created", body = CreateProjectResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn create_project(
    Path(org_slug): Path<String>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<CreateProjectResponse>), AppError> {
    let org_id = orgs_repo::find_org_id_by_slug(&state.db, &org_slug)
        .await
        .map_err(|e| {
            tracing::error!(?e, "create_project org resolve");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let res = projects_service::create_project(&state.db, org_id, user_id, req).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

#[utoipa::path(
    get,
    path = "/orgs/{orgSlug}/projects/{projectKey}/members",
    tag = "projects",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path),
        ("projectKey" = String, Path),
    ),
    responses(
        (status = 200, description = "Members", body = ListProjectMembersResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn list_project_members(
    Path((org_slug, project_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<(StatusCode, Json<ListProjectMembersResponse>), AppError> {
    let res =
        projects_service::list_project_members(&state.db, user_id, &org_slug, &project_key)
            .await?;
    Ok((StatusCode::OK, Json(res)))
}
