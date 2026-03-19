use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const ACCESS_TOKEN_TTL_MINUTES: i64 = 15;
const REFRESH_TOKEN_TTL_DAYS: i64 = 30;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id (uuid string)
    pub exp: usize,  // unix timestamp
    pub token_type: TokenType,
}

#[derive(Debug, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

pub fn issue_token_pair(user_id: Uuid, secret: &str) -> anyhow::Result<TokenPair> {
    Ok(TokenPair {
        access_token: sign_access(user_id, secret)?,
        refresh_token: sign_refresh(user_id, secret)?,
    })
}

pub fn sign_access(user_id: Uuid, secret: &str) -> anyhow::Result<String> {
    sign(
        user_id,
        secret,
        TokenType::Access,
        Duration::minutes(ACCESS_TOKEN_TTL_MINUTES),
    )
}

pub fn sign_refresh(user_id: Uuid, secret: &str) -> anyhow::Result<String> {
    sign(
        user_id,
        secret,
        TokenType::Refresh,
        Duration::days(REFRESH_TOKEN_TTL_DAYS),
    )
}

fn sign(
    user_id: Uuid,
    secret: &str,
    token_type: TokenType,
    ttl: Duration,
) -> anyhow::Result<String> {
    let exp = (Utc::now() + ttl).timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
        token_type,
    };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

pub fn verify(token: &str, secret: &str) -> anyhow::Result<Claims> {
    let data = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(data.claims)
}
