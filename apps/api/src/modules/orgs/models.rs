use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub struct CreateOrgRequest {
    pub name: String,
    pub slug: String,
}

pub struct CreateOrgInput<'a> {
    pub name: &'a str,
    pub slug: &'a str,
}

#[derive(Serialize, ToSchema)]
pub struct CreateOrgResult {
    pub org_id: Uuid,
    pub name: String,
    pub slug: String,
}
