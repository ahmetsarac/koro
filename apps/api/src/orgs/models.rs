use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateOrgRequest {
    pub name: String,
    pub slug: String,
}

pub struct CreateOrgInput<'a> {
    pub name: &'a str,
    pub slug: &'a str,
}

#[derive(Serialize)]
pub struct CreateOrgResult {
    pub org_id: Uuid,
    pub name: String,
    pub slug: String,
}
