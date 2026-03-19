use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::modules::{
    auth::user::AuthUser,
    comments::{
        models::{CreateCommentRequest, CreateCommentResponse, ListCommentsResponse},
        service as comments_service,
    },
    core::{state::AppState, AppError},
};

pub async fn create_comment(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<CreateCommentResponse>), AppError> {
    let res = comments_service::create_comment(
        &state.db,
        &org_slug,
        &issue_key,
        user_id,
        &req.body,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(res)))
}

pub async fn list_comments(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<(StatusCode, Json<ListCommentsResponse>), AppError> {
    let res =
        comments_service::list_comments(&state.db, &org_slug, &issue_key, user_id).await?;
    Ok((StatusCode::OK, Json(res)))
}
