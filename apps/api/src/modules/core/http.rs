use axum::{
    extract::DefaultBodyLimit,
    response::IntoResponse,
    Json,
    Router,
    extract::State,
    http::StatusCode,
    routing::{delete, get, patch, post},
};
use serde::Serialize;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

use crate::modules::{
    auth::handlers as auth_handlers,
    comments::handlers as comments_handlers,
    core::{db, state::AppState, uploads},
    events::handlers as events_handlers,
    invites::handlers as invites_handlers,
    issues::handlers as issues_handlers,
    orgs::handlers as orgs_handlers,
    projects::handlers as projects_handlers,
    relations::handlers as relations_handlers,
    users::handlers as users_handlers,
};

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
}

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_default();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "system",
    responses(
        (status = 200, description = "Database OK", body = HealthResponse),
        (status = 503, description = "Database unreachable", body = HealthResponse),
    )
)]
pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    match db::ping(&state.db).await {
        Ok(()) => (StatusCode::OK, Json(HealthResponse { status: "ok" })).into_response(),
        Err(err) => {
            tracing::error!(?err, "healthcheck failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse {
                    status: "db_down",
                }),
            )
                .into_response()
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Koro API",
        version = "0.1.0",
        description = "REST API for Koro (issues, orgs, projects, comments, uploads, invites)."
    ),
    modifiers(&SecurityAddon),
    paths(
        health,
        uploads::upload,
        uploads::get_attachment,
        users_handlers::get_me,
        users_handlers::setup,
        orgs_handlers::create_org,
        orgs_handlers::patch_org,
        projects_handlers::list_projects,
        projects_handlers::get_project,
        projects_handlers::patch_project,
        projects_handlers::delete_project,
        projects_handlers::create_project,
        projects_handlers::list_project_members,
        projects_handlers::record_project_view,
        invites_handlers::create_invite,
        invites_handlers::get_invite,
        invites_handlers::accept_invite,
        auth_handlers::login,
        auth_handlers::signup,
        auth_handlers::refresh,
        issues_handlers::create_issue,
        issues_handlers::list_my_issues,
        issues_handlers::bulk_my_issues,
        issues_handlers::list_issues,
        issues_handlers::list_project_issues_by_key,
        issues_handlers::update_issue_status,
        issues_handlers::update_issue,
        issues_handlers::delete_issue_by_key,
        issues_handlers::get_board,
        issues_handlers::get_board_by_key,
        issues_handlers::list_workflow_statuses,
        issues_handlers::create_workflow_status,
        issues_handlers::patch_workflow_status,
        issues_handlers::delete_workflow_status,
        issues_handlers::update_issue_board_position,
        issues_handlers::get_issue,
        issues_handlers::get_issue_by_key,
        issues_handlers::assign_issue,
        issues_handlers::assign_me,
        issues_handlers::unassign_issue,
        comments_handlers::create_comment,
        comments_handlers::list_comments,
        relations_handlers::create_relation,
        relations_handlers::get_relations,
        relations_handlers::delete_relation,
        events_handlers::list_issue_events,
    ),
    components(schemas(
        HealthResponse,
        crate::modules::auth::models::LoginRequest,
        crate::modules::auth::models::SignupRequest,
        crate::modules::auth::models::RefreshRequest,
        crate::modules::auth::models::AuthTokensResponse,
        crate::modules::users::models::UserOrganization,
        crate::modules::users::models::MeResponse,
        crate::modules::users::models::SetupRequest,
        crate::modules::users::models::SetupResponse,
        crate::modules::orgs::models::CreateOrgRequest,
        crate::modules::orgs::models::CreateOrgResult,
        crate::modules::orgs::models::PatchOrgRequest,
        crate::modules::orgs::models::PatchOrgResponse,
        crate::modules::projects::models::ListMyProjectsQuery,
        crate::modules::projects::models::ProjectItem,
        crate::modules::projects::models::ListMyProjectsResponse,
        crate::modules::projects::models::GetProjectResponse,
        crate::modules::projects::models::CreateProjectRequest,
        crate::modules::projects::models::CreateProjectResponse,
        crate::modules::projects::models::PatchProjectRequest,
        crate::modules::projects::models::PatchProjectResponse,
        crate::modules::projects::models::DeleteProjectRequest,
        crate::modules::projects::models::ProjectMemberItem,
        crate::modules::projects::models::ListProjectMembersResponse,
        crate::modules::invites::models::CreateInviteRequest,
        crate::modules::invites::models::CreateInviteResponse,
        crate::modules::invites::models::GetInviteResponse,
        crate::modules::invites::models::AcceptInviteRequest,
        crate::modules::invites::models::AcceptInviteResponse,
        crate::modules::issues::models::MyIssueSortBy,
        crate::modules::issues::models::SortDirection,
        crate::modules::issues::models::MyIssuesCursor,
        crate::modules::issues::models::ListMyIssuesQuery,
        crate::modules::issues::models::ListIssuesQuery,
        crate::modules::issues::models::CreateIssueRequest,
        crate::modules::issues::models::CreateIssueResponse,
        crate::modules::issues::models::IssueListItem,
        crate::modules::issues::models::ListIssuesResponse,
        crate::modules::issues::models::ListProjectIssuesResponse,
        crate::modules::issues::models::MyIssueItem,
        crate::modules::issues::models::MyIssueFacets,
        crate::modules::issues::models::ListMyIssuesResponse,
        crate::modules::issues::models::UpdateIssueStatusRequest,
        crate::modules::issues::models::UpdateIssueStatusResponse,
        crate::modules::issues::models::UpdateIssueRequest,
        crate::modules::issues::models::UpdateIssueResponse,
        crate::modules::issues::models::BulkArchiveIssues,
        crate::modules::issues::models::BulkSetIssueStatus,
        crate::modules::issues::models::BulkSetIssuePriority,
        crate::modules::issues::models::BulkMyIssuesRequest,
        crate::modules::issues::models::BulkMyIssuesResponse,
        crate::modules::issues::models::BoardColumnDef,
        crate::modules::issues::models::BoardResponse,
        crate::modules::issues::models::CreateWorkflowStatusRequest,
        crate::modules::issues::models::DeleteWorkflowStatusQuery,
        crate::modules::issues::models::ListWorkflowStatusesResponse,
        crate::modules::issues::models::PatchWorkflowStatusRequest,
        crate::modules::issues::models::UpdateIssueBoardPositionRequest,
        crate::modules::issues::models::UpdateIssueBoardPositionResponse,
        crate::modules::issues::models::WorkflowStatusItem,
        crate::modules::issues::models::WorkflowStatusesByCategory,
        crate::modules::issues::models::IssueDetailResponse,
        crate::modules::issues::models::AssignIssueRequest,
        crate::modules::comments::models::CreateCommentRequest,
        crate::modules::comments::models::CreateCommentResponse,
        crate::modules::comments::models::CommentItem,
        crate::modules::comments::models::ListCommentsResponse,
        crate::modules::events::models::EventActor,
        crate::modules::events::models::EventItem,
        crate::modules::events::models::ListEventsResponse,
        crate::modules::relations::models::CreateRelationRequest,
        crate::modules::relations::models::CreateRelationResponse,
        crate::modules::relations::models::RelationItem,
        crate::modules::relations::models::GetRelationsResponse,
        uploads::UploadResponse,
    )),
    tags(
        (name = "system", description = "Health"),
        (name = "auth", description = "Authentication (JWT)"),
        (name = "users", description = "Current user and bootstrap"),
        (name = "uploads", description = "Attachments (multipart)"),
        (name = "orgs", description = "Organizations"),
        (name = "projects", description = "Projects"),
        (name = "invites", description = "Org invites"),
        (name = "issues", description = "Issues, board, assignment"),
        (name = "comments", description = "Issue comments"),
        (name = "relations", description = "Issue relations"),
        (name = "events", description = "Issue activity / audit log"),
    ),
)]
pub struct ApiDoc;

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(
            SwaggerUi::new("/swagger")
                .url("/openapi.json", ApiDoc::openapi()),
        )
        .route("/health", get(health))
        .route(
            "/uploads",
            post(uploads::upload).layer(DefaultBodyLimit::max(11 * 1024 * 1024)), // 10 MiB uploads
        )
        .route(
            "/uploads/attachments/{filename}",
            get(uploads::get_attachment),
        )
        .route("/me", get(users_handlers::get_me))
        .route("/my-issues", get(issues_handlers::list_my_issues))
        .route(
            "/my-issues/bulk",
            post(issues_handlers::bulk_my_issues),
        )
        .route("/projects", get(projects_handlers::list_projects))
        .route("/setup", post(users_handlers::setup))
        .route("/orgs", post(orgs_handlers::create_org))
        .route(
            "/orgs/{orgSlug}",
            patch(orgs_handlers::patch_org),
        )
        .route("/orgs/{orgId}/invites", post(invites_handlers::create_invite))
        .route("/invites/{token}", get(invites_handlers::get_invite))
        .route("/invites/{token}/accept", post(invites_handlers::accept_invite))
        .route("/login", post(auth_handlers::login))
        .route("/signup", post(auth_handlers::signup))
        .route("/refresh", post(auth_handlers::refresh))
        .route(
            "/orgs/{orgSlug}/projects",
            post(projects_handlers::create_project),
        )
        .route("/projects/{projectId}/issues", post(issues_handlers::create_issue))
        .route("/projects/{projectId}/issues", get(issues_handlers::list_issues))
        .route(
            "/orgs/{orgSlug}/projects/{projectKey}",
            get(projects_handlers::get_project)
                .patch(projects_handlers::patch_project)
                .delete(projects_handlers::delete_project),
        )
        .route(
            "/orgs/{orgSlug}/projects/{projectKey}/view",
            post(projects_handlers::record_project_view),
        )
        .route(
            "/orgs/{orgSlug}/projects/{projectKey}/members",
            get(projects_handlers::list_project_members),
        )
        .route(
            "/orgs/{orgSlug}/projects/{projectKey}/issues",
            get(issues_handlers::list_project_issues_by_key),
        )
        .route(
            "/orgs/{orgSlug}/projects/{projectKey}/board",
            get(issues_handlers::get_board_by_key),
        )
        .route(
            "/orgs/{orgSlug}/projects/{projectKey}/workflow-statuses",
            get(issues_handlers::list_workflow_statuses).post(issues_handlers::create_workflow_status),
        )
        .route(
            "/orgs/{orgSlug}/projects/{projectKey}/workflow-statuses/{statusId}",
            patch(issues_handlers::patch_workflow_status).delete(issues_handlers::delete_workflow_status),
        )
        .route(
            "/issues/{issueId}/status",
            patch(issues_handlers::update_issue_status),
        )
        .route(
            "/issues/{issueId}/board-position",
            patch(issues_handlers::update_issue_board_position),
        )
        .route("/projects/{projectId}/board", get(issues_handlers::get_board))
        .route("/issues/{issueId}", get(issues_handlers::get_issue))
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}",
            get(issues_handlers::get_issue_by_key)
                .patch(issues_handlers::update_issue)
                .delete(issues_handlers::delete_issue_by_key),
        )
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}/relations",
            post(relations_handlers::create_relation),
        )
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}/relations",
            get(relations_handlers::get_relations),
        )
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}/relations/{relationId}",
            delete(relations_handlers::delete_relation),
        )
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}/comments",
            post(comments_handlers::create_comment),
        )
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}/comments",
            get(comments_handlers::list_comments),
        )
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}/assignee",
            patch(issues_handlers::assign_issue),
        )
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}/assignee",
            delete(issues_handlers::unassign_issue),
        )
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}/assign-me",
            post(issues_handlers::assign_me),
        )
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}/events",
            get(events_handlers::list_issue_events),
        )
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
}
