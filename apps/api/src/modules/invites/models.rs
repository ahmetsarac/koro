use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub struct CreateInviteRequest {
    pub email: String,
    pub role: String,
}

#[derive(Serialize, ToSchema)]
pub struct CreateInviteResponse {
    pub invite_url: String,
}

#[derive(Serialize, ToSchema)]
pub struct GetInviteResponse {
    pub org_name: String,
    pub email: String,
    pub role: String,
    pub expires_at: String,
}

#[derive(Deserialize, ToSchema)]
pub struct AcceptInviteRequest {
    pub name: String,
    pub password: String,
}

pub struct AcceptInviteInput {
    pub name: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct AcceptInviteResponse {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub org_role: String,
}
