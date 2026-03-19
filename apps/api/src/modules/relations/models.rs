use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateRelationRequest {
    pub relation_type: String,
    pub target_issue_key: String,
}

#[derive(Serialize)]
pub struct CreateRelationResponse {
    pub relation_id: Uuid,
}

#[derive(Serialize)]
pub struct RelationItem {
    pub issue_key: String,
    pub title: String,
}

#[derive(Serialize)]
pub struct GetRelationsResponse {
    pub blocking: Vec<RelationItem>,
    pub blocked_by: Vec<RelationItem>,
    pub related: Vec<RelationItem>,
    pub duplicates: Vec<RelationItem>,
    pub duplicated_by: Vec<RelationItem>,
}
