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

#[utoipa::path(
    post,
    path = "/orgs/{orgSlug}/issues/{issueKey}/comments",
    tag = "comments",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path),
        ("issueKey" = String, Path),
    ),
    request_body = CreateCommentRequest,
    responses(
        (status = 201, description = "Comment created", body = CreateCommentResponse),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
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

#[utoipa::path(
    get,
    path = "/orgs/{orgSlug}/issues/{issueKey}/comments",
    tag = "comments",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path),
        ("issueKey" = String, Path),
    ),
    responses(
        (status = 200, description = "Comments", body = ListCommentsResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn list_comments(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<(StatusCode, Json<ListCommentsResponse>), AppError> {
    let res =
        comments_service::list_comments(&state.db, &org_slug, &issue_key, user_id).await?;
    Ok((StatusCode::OK, Json(res)))
}
