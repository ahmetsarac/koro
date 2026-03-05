use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{auth, jwt, state::AppState};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

pub async fn login(State(state): State<AppState>, Json(req): Json<LoginRequest>) -> impl IntoResponse {

    let secret = match std::env::var("JWT_SECRET") {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let row = match sqlx::query!(
        r#"SELECT id, password_hash FROM users WHERE email = $1 AND is_active = true"#,
        req.email
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(e) => {
            eprintln!("login query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let hash = match row.password_hash {
        Some(h) => h,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let ok = match auth::verify_password(&req.password, &hash) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("verify password error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !ok {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let token = match jwt::sign(row.id, &secret) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("jwt sign error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    (StatusCode::OK, Json(LoginResponse { token })).into_response()
}
