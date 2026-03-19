use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::modules::{
    auth::user::AuthUser,
    core::{state::AppState, AppError},
    relations::{
        models::{CreateRelationRequest, CreateRelationResponse, GetRelationsResponse},
        service as relations_service,
    },
};

#[utoipa::path(
    post,
    path = "/orgs/{orgSlug}/issues/{issueKey}/relations",
    tag = "relations",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path, description = "Organization slug"),
        ("issueKey" = String, Path, description = "Source issue key"),
    ),
    request_body = CreateRelationRequest,
    responses(
        (status = 201, description = "Relation created", body = CreateRelationResponse),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn create_relation(
    Path((org_slug, source_issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateRelationRequest>,
) -> Result<(StatusCode, Json<CreateRelationResponse>), AppError> {
    let res = relations_service::create_relation(
        &state.db,
        &org_slug,
        &source_issue_key,
        user_id,
        req,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(res)))
}

#[utoipa::path(
    get,
    path = "/orgs/{orgSlug}/issues/{issueKey}/relations",
    tag = "relations",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path),
        ("issueKey" = String, Path),
    ),
    responses(
        (status = 200, description = "Relations for source issue", body = GetRelationsResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn get_relations(
    Path((org_slug, source_issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<(StatusCode, Json<GetRelationsResponse>), AppError> {
    let res =
        relations_service::get_relations(&state.db, &org_slug, &source_issue_key, user_id).await?;
    Ok((StatusCode::OK, Json(res)))
}

#[utoipa::path(
    delete,
    path = "/orgs/{orgSlug}/issues/{issueKey}/relations/{relationId}",
    tag = "relations",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path),
        ("issueKey" = String, Path),
        ("relationId" = uuid::Uuid, Path),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn delete_relation(
    Path((org_slug, _issue_key, relation_id)): Path<(String, String, uuid::Uuid)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<StatusCode, AppError> {
    relations_service::delete_relation(&state.db, &org_slug, user_id, relation_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
