use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json, 
};
use serde::{Deserialize, Serialize};

use crate::{auth, state::AppState};

#[derive(Deserialize)]
pub struct SetupRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct SetupResponse {
    pub user_id: uuid::Uuid,
    pub email: String,
    pub platform_role: &'static str,
}


pub async fn setup(
    State(state): State<AppState>,
    Json(req): Json<SetupRequest>,
) -> impl IntoResponse {
    // basit input guard
    if req.password.len() < 8 {
        return (StatusCode::BAD_REQUEST, "password must be at least 8 chars").into_response();
    }

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // users boş mu?
    let user_count: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&mut *tx)
        .await
    {
        Ok(v) => v,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if user_count > 0 {
        return StatusCode::CONFLICT.into_response();
    }

    let password_hash = match auth::hash_password(&req.password) {
        Ok(h) => h,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let rec = match sqlx::query!(
        r#"
        INSERT INTO users (email, name, password_hash, platform_role)
        VALUES ($1, $2, $3, 'platform_admin')
        RETURNING id, email
        "#,
        req.email,
        req.name,
        password_hash
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("setup insert user failed: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if let Err(e) = tx.commit().await {
        eprintln!("setup tx commit failed: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (
        StatusCode::CREATED,
        Json(SetupResponse {
            user_id: rec.id,
            email: rec.email,
            platform_role: "platform_admin",
        }),
    )
        .into_response()
}
