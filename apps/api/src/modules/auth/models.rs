use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::modules::auth::jwt;

#[derive(Serialize, ToSchema)]
pub struct AuthTokensResponse {
    pub access_token: String,
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

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct SignupRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

pub struct LoginInput {
    pub email: String,
    pub password: String,
}

pub struct SignupInput {
    pub email: String,
    pub name: String,
    pub password: String,
}
