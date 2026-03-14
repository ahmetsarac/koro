use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth, jwt, state::AppState};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}


#[derive(Serialize)]
pub struct AuthTokensResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

impl From<jwt::TokenPair> for AuthTokensResponse {
    fn from(tokens: jwt::TokenPair) -> Self {
        Self {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        }
    }
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    println!("JWT = {}", std::env::var("JWT_SECRET").is_ok());
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

    let tokens = match jwt::issue_token_pair(row.id, &secret) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("jwt sign error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    (StatusCode::OK, Json(AuthTokensResponse::from(tokens))).into_response()
}

/// POST /signup — public registration. Creates user with platform_role 'user'.
pub async fn signup(
    State(state): State<AppState>,
    Json(req): Json<SignupRequest>,
) -> impl IntoResponse {
    if req.password.len() < 8 {
        return (StatusCode::BAD_REQUEST, "password must be at least 8 chars").into_response();
    }
    let email = req.email.trim().to_lowercase();
    let name = req.name.trim();
    if name.is_empty() {
        return (StatusCode::BAD_REQUEST, "name is required").into_response();
    }

    let password_hash = match auth::hash_password(&req.password) {
        Ok(h) => h,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let rec = match sqlx::query!(
        r#"
        INSERT INTO users (email, name, password_hash, platform_role)
        VALUES ($1, $2, $3, 'user')
        RETURNING id
        "#,
        email,
        name,
        password_hash
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return (StatusCode::CONFLICT, "email already registered").into_response();
                }
            }
            eprintln!("signup insert user failed: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let secret = match std::env::var("JWT_SECRET") {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let tokens = match jwt::issue_token_pair(rec.id, &secret) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("signup jwt error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    (StatusCode::CREATED, Json(AuthTokensResponse::from(tokens))).into_response()
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> impl IntoResponse {
    let secret = match std::env::var("JWT_SECRET") {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let claims = match jwt::verify(&req.refresh_token, &secret) {
        Ok(claims) => claims,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    if claims.token_type != jwt::TokenType::Refresh {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(user_id) => user_id,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let user = match sqlx::query!(
        r#"SELECT id FROM users WHERE id = $1 AND is_active = true"#,
        user_id
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(e) => {
            eprintln!("refresh query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let tokens = match jwt::issue_token_pair(user.id, &secret) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("jwt refresh sign error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    (StatusCode::OK, Json(AuthTokensResponse::from(tokens))).into_response()
}
