use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateInviteRequest {
    pub email: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct CreateInviteResponse {
    pub invite_url: String,
}

#[derive(Serialize)]
pub struct GetInviteResponse {
    pub org_name: String,
    pub email: String,
    pub role: String,
    pub expires_at: String,
}

#[derive(Deserialize)]
pub struct AcceptInviteRequest {
    pub name: String,
    pub password: String,
}

pub struct AcceptInviteInput {
    pub name: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AcceptInviteResponse {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub org_role: String,
}
