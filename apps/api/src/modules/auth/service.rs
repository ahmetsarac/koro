use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::{
    auth::{jwt, models::*, password, repository as auth_repo},
    core::AppError,
};

pub fn require_jwt_secret() -> Result<String, AppError> {
    std::env::var("JWT_SECRET").map_err(|_| {
        tracing::error!("JWT_SECRET is not set");
        AppError::Internal
    })
}

pub async fn login(
    pool: &PgPool,
    input: LoginInput,
    secret: &str,
) -> Result<AuthTokensResponse, AppError> {
    let row = auth_repo::find_active_user_for_login(pool, &input.email)
        .await
        .map_err(|e| {
            tracing::error!(?e, "login query");
            AppError::Internal
        })?
        .ok_or(AppError::Unauthorized)?;

    let hash = row.password_hash.ok_or(AppError::Unauthorized)?;

    let ok = password::verify_password(&input.password, &hash).map_err(|e| {
        tracing::error!(?e, "verify password");
        AppError::Internal
    })?;

    if !ok {
        return Err(AppError::Unauthorized);
    }

    let tokens = jwt::issue_token_pair(row.id, secret).map_err(|e| {
        tracing::error!(?e, "jwt sign");
        AppError::Internal
    })?;

    Ok(AuthTokensResponse::from(tokens))
}

pub async fn signup(
    pool: &PgPool,
    input: SignupInput,
    secret: &str,
) -> Result<AuthTokensResponse, AppError> {
    if input.password.len() < 8 {
        return Err(AppError::BadRequest(Some(
            "password must be at least 8 chars",
        )));
    }
    let email = input.email.trim().to_lowercase();
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest(Some("name is required")));
    }

    let password_hash = password::hash_password(&input.password).map_err(|e| {
        tracing::error!(?e, "signup hash");
        AppError::Internal
    })?;

    let user_id = match auth_repo::insert_signup_user(pool, &email, name, &password_hash).await {
        Ok(id) => id,
        Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return Err(AppError::Conflict(Some("email already registered")));
                }
            }
            tracing::error!(?e, "signup insert");
            return Err(AppError::Internal);
        }
    };

    let tokens = jwt::issue_token_pair(user_id, secret).map_err(|e| {
        tracing::error!(?e, "signup jwt");
        AppError::Internal
    })?;

    Ok(AuthTokensResponse::from(tokens))
}

pub async fn refresh(
    pool: &PgPool,
    refresh_token: &str,
    secret: &str,
) -> Result<AuthTokensResponse, AppError> {
    let claims = jwt::verify(refresh_token, secret).map_err(|_| AppError::Unauthorized)?;

    if claims.token_type != jwt::TokenType::Refresh {
        return Err(AppError::Unauthorized);
    }

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    let user = auth_repo::find_active_user_by_id(pool, user_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "refresh query");
            AppError::Internal
        })?
        .ok_or(AppError::Unauthorized)?;

    let tokens = jwt::issue_token_pair(user, secret).map_err(|e| {
        tracing::error!(?e, "jwt refresh sign");
        AppError::Internal
    })?;

    Ok(AuthTokensResponse::from(tokens))
}
