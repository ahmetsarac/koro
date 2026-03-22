use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::modules::{
    auth::user::AuthUser,
    core::{state::AppState, AppError},
    orgs::{
        models::{
            CreateOrgInput, CreateOrgRequest, CreateOrgResult, PatchOrgRequest, PatchOrgResponse,
        },
        service as orgs_service,
    },
};

#[utoipa::path(
    post,
    path = "/orgs",
    tag = "orgs",
    security(("bearer_auth" = [])),
    request_body = CreateOrgRequest,
    responses(
        (status = 201, description = "Organization created", body = CreateOrgResult),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn create_org(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateOrgRequest>,
) -> Result<(StatusCode, Json<CreateOrgResult>), AppError> {
    let created = orgs_service::create_org(
        &state.db,
        user_id,
        CreateOrgInput {
            name: &req.name,
            slug: &req.slug,
        },
    )
    .await?;
    Ok((StatusCode::CREATED, Json(created)))
}

#[utoipa::path(
    patch,
    path = "/orgs/{orgSlug}",
    tag = "orgs",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path, description = "Organization slug"),
    ),
    request_body = PatchOrgRequest,
    responses(
        (status = 200, description = "Organization updated", body = PatchOrgResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (not org admin)"),
        (status = 404, description = "Organization not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn patch_org(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(org_slug): Path<String>,
    Json(req): Json<PatchOrgRequest>,
) -> Result<(StatusCode, Json<PatchOrgResponse>), AppError> {
    let updated = orgs_service::patch_org(&state.db, &org_slug, user_id, &req.name).await?;
    Ok((StatusCode::OK, Json(updated)))
}
