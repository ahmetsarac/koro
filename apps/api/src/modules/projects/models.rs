use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ListMyProjectsQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub q: Option<String>,
    /// When set, only projects in this organization (user must be an org member).
    pub org_slug: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ProjectItem {
    pub id: Uuid,
    pub project_key: String,
    pub name: String,
    pub description: Option<String>,
    pub org_id: Uuid,
    pub org_name: String,
    pub org_slug: String,
    pub issue_count: i64,
    pub member_count: i64,
    pub my_role: String,
    pub created_at: DateTime<Utc>,
    pub viewed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, ToSchema)]
pub struct ListMyProjectsResponse {
    pub items: Vec<ProjectItem>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
    pub has_more: bool,
}

#[derive(Serialize, ToSchema)]
pub struct GetProjectResponse {
    pub id: Uuid,
    pub project_key: String,
    pub name: String,
    pub description: Option<String>,
    pub org_id: Uuid,
    pub org_name: String,
    pub org_slug: String,
    pub issue_count: i64,
    pub member_count: i64,
    pub my_role: String,
    pub created_at: DateTime<Utc>,
    pub viewed_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateProjectRequest {
    pub project_key: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct CreateProjectResponse {
    pub project_id: Uuid,
    pub project_key: String,
    pub name: String,
}

#[derive(Deserialize, ToSchema)]
pub struct PatchProjectRequest {
    pub name: String,
}

#[derive(Serialize, ToSchema)]
pub struct PatchProjectResponse {
    pub id: Uuid,
    pub project_key: String,
    pub name: String,
}

#[derive(Serialize, ToSchema)]
pub struct ProjectMemberItem {
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
    pub project_role: String,
}

#[derive(Serialize, ToSchema)]
pub struct ListProjectMembersResponse {
    pub items: Vec<ProjectMemberItem>,
}
