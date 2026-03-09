use crate::state::AppState;
use axum::{
    Router,
    routing::{delete, get, patch, post},
};
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnResponse};
use tracing::Level;

mod auth;
mod comments;
mod demo;
mod events;
mod health;
mod invites;
mod issues;
mod orgs;
mod projects;
mod relations;
mod setup;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/demo/tasks", get(demo::list_demo_tasks))
        .route("/setup", post(setup::setup))
        .route("/orgs", post(orgs::create_org))
        .route("/orgs/{orgId}/invites", post(invites::create_invite))
        .route("/invites/{token}", get(invites::get_invite))
        .route("/invites/{token}/accept", post(invites::accept_invite))
        .route("/login", post(auth::login))
        .route("/refresh", post(auth::refresh))
        .route("/orgs/{orgId}/projects", post(projects::create_project))
        .route("/projects/{projectId}/issues", post(issues::create_issue))
        .route("/projects/{projectId}/issues", get(issues::list_issues))
        .route(
            "/orgs/{orgSlug}/projects/{projectKey}/members",
            get(projects::list_project_members),
        )
        .route(
            "/issues/{issueId}/status",
            patch(issues::update_issue_status),
        )
        .route("/projects/{projectId}/board", get(issues::get_board))
        .route("/issues/{issueId}", get(issues::get_issue))
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}",
            get(issues::get_issue_by_key),
        )
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}/relations",
            post(relations::create_relation),
        )
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}/relations",
            get(relations::get_relations),
        )
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}/relations/{relationId}",
            delete(relations::delete_relation),
        )
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}/comments",
            post(comments::create_comment),
        )
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}/comments",
            get(comments::list_comments),
        )
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}/assignee",
            patch(issues::assign_issue),
        )
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}/assignee",
            delete(issues::unassign_issue),
        )
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}/assign-me",
            post(issues::assign_me),
        )
        .route(
            "/orgs/{orgSlug}/issues/{issueKey}/events",
            get(events::list_issue_events),
        )
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO))
        )}
