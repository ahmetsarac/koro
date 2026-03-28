use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

// --- "My issues" list query / cursor (API + pagination) --------------------

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MyIssueSortBy {
    CreatedAt,
    UpdatedAt,
    KeySeq,
    Title,
    Status,
    Priority,
}

impl Default for MyIssueSortBy {
    fn default() -> Self {
        Self::UpdatedAt
    }
}

impl MyIssueSortBy {
    pub fn as_sql(self) -> &'static str {
        match self {
            Self::CreatedAt => "i.created_at",
            Self::UpdatedAt => "i.updated_at",
            Self::KeySeq => "i.key_seq",
            Self::Title => "i.title",
            Self::Status => "pws.slug",
            Self::Priority => "i.priority",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl Default for SortDirection {
    fn default() -> Self {
        Self::Desc
    }
}

impl SortDirection {
    pub fn as_sql(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }

    pub fn comparison_sql(self) -> &'static str {
        match self {
            Self::Asc => ">",
            Self::Desc => "<",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MyIssuesCursor {
    pub id: uuid::Uuid,
    pub sort_text: Option<String>,
    pub sort_int: Option<i32>,
    pub sort_timestamp: Option<DateTime<Utc>>,
}

/// Pagination and filters for project-scoped issue lists (`/projects/.../issues`).
#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ListIssuesQuery {
    /// Filter by `workflow_status_id` (UUID).
    pub status: Option<String>,
    pub q: Option<String>,
    /// Pass a user UUID, or the string `me` for the current user.
    pub assignee: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ListMyIssuesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub cursor: Option<String>,
    pub q: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub sort_by: Option<MyIssueSortBy>,
    pub sort_dir: Option<SortDirection>,
    pub filter_type: Option<String>,
    /// Comma-separated: `blocked` (incoming `blocks` relation), `blocking` (outgoing). OR within the set when both present.
    pub relations: Option<String>,
    /// When set, only issues in projects belonging to this organization (user must be an org member).
    pub org_slug: Option<String>,
}

/// Parse `PROJECTKEY-123` into project key prefix and numeric sequence.
pub fn parse_issue_key(key: &str) -> Option<(&str, i32)> {
    let (project_key, seq_str) = key.split_once('-')?;
    let seq = seq_str.parse::<i32>().ok()?;
    Some((project_key, seq))
}

#[derive(Deserialize, ToSchema)]
pub struct CreateIssueRequest {
    pub title: String,
    pub description: Option<String>,
    pub assignee_id: Option<Uuid>,
    pub priority: Option<String>,
    pub workflow_status_id: Option<Uuid>,
    /// Slug within the project (e.g. `todo`). Used when `workflow_status_id` is omitted.
    pub workflow_status_slug: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct CreateIssueResponse {
    pub issue_id: Uuid,
    pub display_key: String,
    pub title: String,
}

#[derive(Serialize, ToSchema, Clone)]
pub struct IssueListItem {
    pub issue_id: Uuid,
    pub display_key: String,
    pub title: String,
    /// Workflow status slug (stable within project).
    pub status: String,
    pub workflow_status_id: Uuid,
    pub status_name: String,
    pub status_category: String,
    /// Derived: another issue has a `blocks` relation to this issue.
    pub is_blocked: bool,
}

#[derive(Serialize, ToSchema)]
pub struct ListIssuesResponse {
    pub items: Vec<IssueListItem>,
}

#[derive(Serialize, ToSchema)]
pub struct ListProjectIssuesResponse {
    pub items: Vec<IssueListItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}

#[derive(Serialize, ToSchema)]
pub struct MyIssueItem {
    pub id: Uuid,
    pub project_id: Uuid,
    pub display_key: String,
    pub title: String,
    pub status: String,
    pub workflow_status_id: Uuid,
    pub status_name: String,
    pub status_category: String,
    /// Derived: another issue has a `blocks` relation to this issue.
    pub is_blocked: bool,
    pub priority: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StatusFacetEntry {
    pub workflow_status_id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub slug: String,
    pub category: String,
    pub position: i32,
    pub count: i64,
}

#[derive(Debug, Default, Serialize, ToSchema)]
pub struct RelationsFacetCounts {
    pub blocked: i64,
    pub blocking: i64,
}

#[derive(Debug, Default, Serialize, ToSchema)]
pub struct MyIssueFacets {
    pub status: Vec<StatusFacetEntry>,
    pub priority: HashMap<String, i64>,
    pub relations: RelationsFacetCounts,
}

#[derive(Serialize, ToSchema)]
pub struct ListMyIssuesResponse {
    pub items: Vec<MyIssueItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub next_cursor: Option<String>,
    pub has_more: bool,
    pub sort_by: MyIssueSortBy,
    pub sort_dir: SortDirection,
    pub facets: MyIssueFacets,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateIssueStatusRequest {
    pub workflow_status_id: Option<Uuid>,
    /// When set without `workflow_status_id`, use the first workflow row in this category for the issue's project.
    pub category: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct UpdateIssueStatusResponse {
    pub issue_id: Uuid,
    pub workflow_status_id: Uuid,
    pub status: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateIssueRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct UpdateIssueResponse {
    pub issue_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub priority: String,
}

#[derive(Serialize, ToSchema, Clone)]
pub struct BoardColumnDef {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub category: String,
    pub position: i32,
}

#[derive(Serialize, ToSchema)]
pub struct BoardResponse {
    pub column_definitions: Vec<BoardColumnDef>,
    pub items_by_column_id: BTreeMap<String, Vec<IssueListItem>>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateIssueBoardPositionRequest {
    pub workflow_status_id: Uuid,
    pub position: i32,
}

#[derive(Serialize, ToSchema)]
pub struct UpdateIssueBoardPositionResponse {
    pub issue_id: Uuid,
    pub workflow_status_id: Uuid,
    pub status: String,
    pub position: i32,
}

#[derive(Serialize, ToSchema)]
pub struct IssueDetailResponse {
    pub issue_id: Uuid,
    pub display_key: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub workflow_status_id: Uuid,
    pub status_name: String,
    pub status_category: String,
    /// Derived: another issue has a `blocks` relation to this issue.
    pub is_blocked: bool,
    pub priority: String,
    pub assignee_id: Option<Uuid>,
    pub assignee_name: Option<String>,
    pub project_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct AssignIssueRequest {
    pub user_id: Uuid,
}

// --- Project workflow settings ---------------------------------------------

#[derive(Serialize, ToSchema, Clone)]
pub struct WorkflowStatusItem {
    pub id: Uuid,
    pub category: String,
    pub name: String,
    pub slug: String,
    pub position: i32,
    pub is_default: bool,
}

#[derive(Serialize, ToSchema)]
pub struct WorkflowStatusesByCategory {
    pub category: String,
    pub statuses: Vec<WorkflowStatusItem>,
}

#[derive(Serialize, ToSchema)]
pub struct ListWorkflowStatusesResponse {
    pub groups: Vec<WorkflowStatusesByCategory>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateWorkflowStatusRequest {
    pub category: String,
    pub name: String,
    pub slug: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct PatchWorkflowStatusRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub position: Option<i32>,
    pub is_default: Option<bool>,
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct DeleteWorkflowStatusQuery {
    pub reassign_to: Uuid,
}
