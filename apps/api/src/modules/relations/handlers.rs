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

pub async fn get_relations(
    Path((org_slug, source_issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<(StatusCode, Json<GetRelationsResponse>), AppError> {
    let res =
        relations_service::get_relations(&state.db, &org_slug, &source_issue_key, user_id).await?;
    Ok((StatusCode::OK, Json(res)))
}

pub async fn delete_relation(
    Path((org_slug, _issue_key, relation_id)): Path<(String, String, uuid::Uuid)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<StatusCode, AppError> {
    relations_service::delete_relation(&state.db, &org_slug, user_id, relation_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
