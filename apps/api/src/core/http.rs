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

use crate::{
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

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

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

pub fn router(state: AppState) -> Router {
    Router::new()
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
        .route("/projects", get(projects_handlers::list_projects))
        .route("/setup", post(users_handlers::setup))
        .route("/orgs", post(orgs_handlers::create_org))
        .route("/orgs/{orgId}/invites", post(invites_handlers::create_invite))
        .route("/invites/{token}", get(invites_handlers::get_invite))
        .route("/invites/{token}/accept", post(invites_handlers::accept_invite))
        .route("/login", post(auth_handlers::login))
        .route("/signup", post(auth_handlers::signup))
        .route("/refresh", post(auth_handlers::refresh))
        .route("/orgs/{orgId}/projects", post(projects_handlers::create_project))
        .route("/projects/{projectId}/issues", post(issues_handlers::create_issue))
        .route("/projects/{projectId}/issues", get(issues_handlers::list_issues))
        .route(
            "/orgs/{orgSlug}/projects/{projectKey}",
            get(projects_handlers::get_project),
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
            get(issues_handlers::get_issue_by_key).patch(issues_handlers::update_issue),
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
