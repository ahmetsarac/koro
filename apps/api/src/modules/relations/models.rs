use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub struct CreateRelationRequest {
    pub relation_type: String,
    pub target_issue_key: String,
}

#[derive(Serialize, ToSchema)]
pub struct CreateRelationResponse {
    pub relation_id: Uuid,
}

#[derive(Serialize, ToSchema)]
pub struct RelationItem {
    pub relation_id: Uuid,
    pub issue_key: String,
    pub title: String,
}

#[derive(Serialize, ToSchema)]
pub struct GetRelationsResponse {
    pub blocking: Vec<RelationItem>,
    pub blocked_by: Vec<RelationItem>,
    pub related: Vec<RelationItem>,
    pub duplicates: Vec<RelationItem>,
    pub duplicated_by: Vec<RelationItem>,
}
