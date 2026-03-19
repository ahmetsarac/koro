use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct UserOrganization {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub role: String,
}

#[derive(Serialize, ToSchema)]
pub struct MeResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub organizations: Vec<UserOrganization>,
}

#[derive(Deserialize, ToSchema)]
pub struct SetupRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct SetupResponse {
    pub user_id: Uuid,
    pub email: String,
    pub platform_role: &'static str,
}
