use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Serialize;

use crate::{services::health as health_service, state::AppState};

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    match health_service::check(&state.db).await {
        health_service::HealthStatus::Ok => {
            (StatusCode::OK, Json(HealthResponse { status: "ok" })).into_response()
        }
        health_service::HealthStatus::DbDown => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthResponse {
                status: "db_down",
            }),
        )
            .into_response(),
    }
}
