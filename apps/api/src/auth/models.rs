use serde::{Deserialize, Serialize};

use crate::auth::jwt;

#[derive(Serialize)]
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

#[derive(Deserialize)]
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
