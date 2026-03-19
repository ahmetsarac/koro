use axum::{Json, extract::State, http::StatusCode};

use crate::modules::{
    auth::user::AuthUser,
    core::{state::AppState, AppError},
    orgs::{
        models::{CreateOrgInput, CreateOrgRequest, CreateOrgResult},
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
