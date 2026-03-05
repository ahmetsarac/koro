use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::{db, state::AppState};

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    match db::ping(&state.db).await {
        Ok(_) => (StatusCode::OK, Json(HealthResponse { status: "ok"})).into_response(),
        Err(err) => {
            eprintln!("healthcheck failed: {err:?}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse { status: "db_down" }),
            ).into_response()
        }
    }
}


