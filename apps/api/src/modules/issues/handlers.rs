use axum::{
    Json,
    extract::{Path, Query, State},
};
use crate::modules::{auth::user::AuthUser, core::state::AppState};

use super::{
    models::{
        AssignIssueRequest, BulkMyIssuesRequest, CreateIssueRequest, CreateWorkflowStatusRequest,
        DeleteWorkflowStatusQuery, ListIssuesQuery, ListMyIssuesQuery, PatchWorkflowStatusRequest,
        UpdateIssueBoardPositionRequest, UpdateIssueRequest, UpdateIssueStatusRequest,
    },
    service,
};

#[utoipa::path(
    post,
    path = "/projects/{projectId}/issues",
    tag = "issues",
    security(("bearer_auth" = [])),
    params(
        ("projectId" = uuid::Uuid, Path, description = "Project id"),
    ),
    request_body = CreateIssueRequest,
    responses(
        (status = 201, description = "Created", body = super::models::CreateIssueResponse),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn create_issue(
    Path(project_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateIssueRequest>,
) -> impl axum::response::IntoResponse {
    service::create_issue(&state.db, project_id, user_id, req).await
}

#[utoipa::path(
    get,
    path = "/my-issues",
    tag = "issues",
    security(("bearer_auth" = [])),
    params(ListMyIssuesQuery),
    responses(
        (status = 200, description = "Cursor page + facets", body = super::models::ListMyIssuesResponse),
        (status = 500, description = "Server error"),
    )
)]
pub async fn list_my_issues(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(query): Query<ListMyIssuesQuery>,
) -> impl axum::response::IntoResponse {
    service::list_my_issues(&state.db, user_id, query).await
}

#[utoipa::path(
    post,
    path = "/my-issues/bulk",
    tag = "issues",
    security(("bearer_auth" = [])),
    request_body = BulkMyIssuesRequest,
    responses(
        (status = 200, description = "Updated", body = super::models::BulkMyIssuesResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn bulk_my_issues(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<BulkMyIssuesRequest>,
) -> impl axum::response::IntoResponse {
    service::bulk_my_issues(&state.db, user_id, req).await
}

#[utoipa::path(
    get,
    path = "/projects/{projectId}/issues",
    tag = "issues",
    security(("bearer_auth" = [])),
    params(
        ("projectId" = uuid::Uuid, Path, description = "Project id"),
        ListIssuesQuery,
    ),
    responses(
        (status = 200, description = "Issue list", body = super::models::ListIssuesResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn list_issues(
    Path(project_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(q): Query<ListIssuesQuery>,
) -> impl axum::response::IntoResponse {
    service::list_issues(&state.db, project_id, user_id, q).await
}

#[utoipa::path(
    get,
    path = "/orgs/{orgSlug}/projects/{projectKey}/issues",
    tag = "issues",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path),
        ("projectKey" = String, Path),
        ListIssuesQuery,
    ),
    responses(
        (status = 200, description = "Paged issue list", body = super::models::ListProjectIssuesResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn list_project_issues_by_key(
    Path((org_slug, project_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(q): Query<ListIssuesQuery>,
) -> impl axum::response::IntoResponse {
    service::list_project_issues_by_key(&state.db, org_slug, project_key, user_id, q).await
}

#[utoipa::path(
    patch,
    path = "/issues/{issueId}/status",
    tag = "issues",
    security(("bearer_auth" = [])),
    params(
        ("issueId" = uuid::Uuid, Path),
    ),
    request_body = UpdateIssueStatusRequest,
    responses(
        (status = 200, description = "Updated", body = super::models::UpdateIssueStatusResponse),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn update_issue_status(
    Path(issue_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<UpdateIssueStatusRequest>,
) -> impl axum::response::IntoResponse {
    service::update_issue_status(&state.db, issue_id, user_id, req).await
}

#[utoipa::path(
    patch,
    path = "/orgs/{orgSlug}/issues/{issueKey}",
    tag = "issues",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path),
        ("issueKey" = String, Path),
    ),
    request_body = UpdateIssueRequest,
    responses(
        (status = 200, description = "Updated", body = super::models::UpdateIssueResponse),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn update_issue(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<UpdateIssueRequest>,
) -> impl axum::response::IntoResponse {
    service::update_issue(&state.db, org_slug, issue_key, user_id, req).await
}

#[utoipa::path(
    delete,
    path = "/orgs/{orgSlug}/issues/{issueKey}",
    tag = "issues",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path),
        ("issueKey" = String, Path),
    ),
    responses(
        (status = 204, description = "Deleted (archived issue only)"),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn delete_issue_by_key(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl axum::response::IntoResponse {
    service::delete_issue_by_key(&state.db, org_slug, issue_key, user_id).await
}

#[utoipa::path(
    get,
    path = "/projects/{projectId}/board",
    tag = "issues",
    security(("bearer_auth" = [])),
    params(
        ("projectId" = uuid::Uuid, Path),
    ),
    responses(
        (status = 200, description = "Kanban columns", body = super::models::BoardResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn get_board(
    Path(project_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl axum::response::IntoResponse {
    service::get_board(&state.db, project_id, user_id).await
}

#[utoipa::path(
    get,
    path = "/orgs/{orgSlug}/projects/{projectKey}/board",
    tag = "issues",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path),
        ("projectKey" = String, Path),
    ),
    responses(
        (status = 200, description = "Kanban columns", body = super::models::BoardResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn get_board_by_key(
    Path((org_slug, project_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl axum::response::IntoResponse {
    service::get_board_by_key(&state.db, org_slug, project_key, user_id).await
}

#[utoipa::path(
    patch,
    path = "/issues/{issueId}/board-position",
    tag = "issues",
    security(("bearer_auth" = [])),
    params(
        ("issueId" = uuid::Uuid, Path),
    ),
    request_body = UpdateIssueBoardPositionRequest,
    responses(
        (status = 200, description = "Updated position", body = super::models::UpdateIssueBoardPositionResponse),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn update_issue_board_position(
    Path(issue_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<UpdateIssueBoardPositionRequest>,
) -> impl axum::response::IntoResponse {
    service::update_issue_board_position(&state.db, issue_id, user_id, req).await
}

#[utoipa::path(
    get,
    path = "/issues/{issueId}",
    tag = "issues",
    security(("bearer_auth" = [])),
    params(
        ("issueId" = uuid::Uuid, Path),
    ),
    responses(
        (status = 200, description = "Issue detail", body = super::models::IssueDetailResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn get_issue(
    Path(issue_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl axum::response::IntoResponse {
    service::get_issue(&state.db, issue_id, user_id).await
}

#[utoipa::path(
    get,
    path = "/orgs/{orgSlug}/issues/{issueKey}",
    tag = "issues",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path),
        ("issueKey" = String, Path),
    ),
    responses(
        (status = 200, description = "Issue detail", body = super::models::IssueDetailResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn get_issue_by_key(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl axum::response::IntoResponse {
    service::get_issue_by_key(&state.db, org_slug, issue_key, user_id).await
}

#[utoipa::path(
    patch,
    path = "/orgs/{orgSlug}/issues/{issueKey}/assignee",
    tag = "issues",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path),
        ("issueKey" = String, Path),
    ),
    request_body = AssignIssueRequest,
    responses(
        (status = 204, description = "Assigned"),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn assign_issue(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(_actor_id): AuthUser,
    Json(req): Json<AssignIssueRequest>,
) -> impl axum::response::IntoResponse {
    service::assign_issue(&state.db, org_slug, issue_key, req).await
}

#[utoipa::path(
    post,
    path = "/orgs/{orgSlug}/issues/{issueKey}/assign-me",
    tag = "issues",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path),
        ("issueKey" = String, Path),
    ),
    responses(
        (status = 204, description = "Assigned to caller"),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn assign_me(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(actor_id): AuthUser,
) -> impl axum::response::IntoResponse {
    service::assign_me(&state.db, org_slug, issue_key, actor_id).await
}

#[utoipa::path(
    delete,
    path = "/orgs/{orgSlug}/issues/{issueKey}/assignee",
    tag = "issues",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path),
        ("issueKey" = String, Path),
    ),
    responses(
        (status = 204, description = "Unassigned"),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn unassign_issue(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(actor_id): AuthUser,
) -> impl axum::response::IntoResponse {
    service::unassign_issue(&state.db, org_slug, issue_key, actor_id).await
}

#[utoipa::path(
    get,
    path = "/orgs/{orgSlug}/projects/{projectKey}/workflow-statuses",
    tag = "issues",
    security(("bearer_auth" = [])),
    params(
        ("orgSlug" = String, Path),
        ("projectKey" = String, Path),
    ),
    responses(
        (status = 200, description = "Grouped workflow statuses", body = super::models::ListWorkflowStatusesResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn list_workflow_statuses(
    Path((org_slug, project_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl axum::response::IntoResponse {
    service::list_workflow_statuses(&state.db, org_slug, project_key, user_id).await
}

#[utoipa::path(
    post,
    path = "/orgs/{orgSlug}/projects/{projectKey}/workflow-statuses",
    tag = "issues",
    security(("bearer_auth" = [])),
    request_body = CreateWorkflowStatusRequest,
    responses(
        (status = 201, description = "Created", body = super::models::WorkflowStatusItem),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn create_workflow_status(
    Path((org_slug, project_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateWorkflowStatusRequest>,
) -> impl axum::response::IntoResponse {
    service::create_workflow_status(&state.db, org_slug, project_key, user_id, req).await
}

#[utoipa::path(
    patch,
    path = "/orgs/{orgSlug}/projects/{projectKey}/workflow-statuses/{statusId}",
    tag = "issues",
    security(("bearer_auth" = [])),
    request_body = PatchWorkflowStatusRequest,
    responses(
        (status = 200, description = "Updated", body = super::models::WorkflowStatusItem),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn patch_workflow_status(
    Path((org_slug, project_key, status_id)): Path<(String, String, uuid::Uuid)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<PatchWorkflowStatusRequest>,
) -> impl axum::response::IntoResponse {
    service::patch_workflow_status(&state.db, org_slug, project_key, status_id, user_id, req).await
}

#[utoipa::path(
    delete,
    path = "/orgs/{orgSlug}/projects/{projectKey}/workflow-statuses/{statusId}",
    tag = "issues",
    security(("bearer_auth" = [])),
    params(DeleteWorkflowStatusQuery),
    responses(
        (status = 204, description = "Deleted"),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_workflow_status(
    Path((org_slug, project_key, status_id)): Path<(String, String, uuid::Uuid)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(q): Query<DeleteWorkflowStatusQuery>,
) -> impl axum::response::IntoResponse {
    service::delete_workflow_status(&state.db, org_slug, project_key, status_id, user_id, q).await
}
