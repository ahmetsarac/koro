use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};
use std::collections::BTreeMap;
use std::collections::HashMap;

use crate::{auth_user::AuthUser, state::AppState};

#[derive(serde::Deserialize)]
pub struct CreateIssueRequest {
    pub title: String,
    pub description: Option<String>,
    pub assignee_id: Option<uuid::Uuid>,
}
#[derive(Serialize)]
pub struct CreateIssueResponse {
    pub issue_id: uuid::Uuid,
    pub display_key: String, // e.g. APP-1
    pub title: String,
}

pub async fn create_issue(
    Path(project_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateIssueRequest>,
) -> impl IntoResponse {
    if req.title.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "title is required").into_response();
    }

    // 1) project member check
    let is_member = match sqlx::query_scalar::<_, i32>(
        r#"
        SELECT 1
        FROM project_members
        WHERE project_id = $1 AND user_id = $2
        "#,
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("create_issue member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    // 2) tx start
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // 3) lock project row, read org_id + project_key + seq
    let p = match sqlx::query!(
        r#"
        SELECT org_id, project_key, next_issue_seq
        FROM projects
        WHERE id = $1
        FOR UPDATE
        "#,
        project_id
    )
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("create_issue select project error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let key_seq = p.next_issue_seq;

    // 3.5) optional assignee check (must be project member)
    if let Some(assignee_id) = req.assignee_id {
        let ok = match sqlx::query_scalar::<_, i32>(
            r#"
            SELECT 1
            FROM project_members
            WHERE project_id = $1 AND user_id = $2
            "#,
        )
        .bind(project_id)
        .bind(assignee_id)
        .fetch_optional(&mut *tx)
        .await
        {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(e) => {
                eprintln!("create_issue assignee member check error: {e:?}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

        if !ok {
            return (StatusCode::BAD_REQUEST, "assignee must be a project member").into_response();
        }
    }

    // 4) insert issue
    let issue = match sqlx::query!(
        r#"
        INSERT INTO issues (org_id, project_id, key_seq, title, description, reporter_id, assignee_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, title
        "#,
        p.org_id,
        project_id,
        key_seq,
        req.title,
        req.description,
        user_id,
        req.assignee_id
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(i) => i,
        Err(e) => {
            eprintln!("create_issue insert error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // 5) bump seq
    if let Err(e) = sqlx::query!(
        r#"UPDATE projects SET next_issue_seq = next_issue_seq + 1 WHERE id = $1"#,
        project_id
    )
    .execute(&mut *tx)
    .await
    {
        eprintln!("create_issue bump seq error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // 6) commit
    if let Err(e) = tx.commit().await {
        eprintln!("create_issue commit error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let display_key = format!("{}-{}", p.project_key, key_seq);

    (
        StatusCode::CREATED,
        Json(CreateIssueResponse {
            issue_id: issue.id,
            display_key,
            title: issue.title,
        }),
    )
        .into_response()
}

#[derive(Serialize)]
pub struct IssueListItem {
    pub issue_id: uuid::Uuid,
    pub display_key: String,
    pub title: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct ListIssuesResponse {
    pub items: Vec<IssueListItem>,
}

#[derive(Serialize)]
pub struct ListProjectIssuesResponse {
    pub items: Vec<IssueListItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
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
    fn as_sql(self) -> &'static str {
        match self {
            Self::CreatedAt => "i.created_at",
            Self::UpdatedAt => "i.updated_at",
            Self::KeySeq => "i.key_seq",
            Self::Title => "i.title",
            Self::Status => "i.status",
            Self::Priority => "i.priority",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
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
    fn as_sql(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }

    fn comparison_sql(self) -> &'static str {
        match self {
            Self::Asc => ">",
            Self::Desc => "<",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct MyIssuesCursor {
    id: uuid::Uuid,
    sort_text: Option<String>,
    sort_int: Option<i32>,
    sort_timestamp: Option<DateTime<Utc>>,
}

#[derive(FromRow)]
struct MyIssueRow {
    id: uuid::Uuid,
    project_key: String,
    key_seq: i32,
    title: String,
    status: String,
    priority: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl MyIssuesCursor {
    fn from_row(sort_by: MyIssueSortBy, row: &MyIssueRow) -> Self {
        match sort_by {
            MyIssueSortBy::CreatedAt => Self {
                id: row.id,
                sort_text: None,
                sort_int: None,
                sort_timestamp: Some(row.created_at),
            },
            MyIssueSortBy::UpdatedAt => Self {
                id: row.id,
                sort_text: None,
                sort_int: None,
                sort_timestamp: Some(row.updated_at),
            },
            MyIssueSortBy::KeySeq => Self {
                id: row.id,
                sort_text: None,
                sort_int: Some(row.key_seq),
                sort_timestamp: None,
            },
            MyIssueSortBy::Title => Self {
                id: row.id,
                sort_text: Some(row.title.clone()),
                sort_int: None,
                sort_timestamp: None,
            },
            MyIssueSortBy::Status => Self {
                id: row.id,
                sort_text: Some(row.status.clone()),
                sort_int: None,
                sort_timestamp: None,
            },
            MyIssueSortBy::Priority => Self {
                id: row.id,
                sort_text: Some(row.priority.clone()),
                sort_int: None,
                sort_timestamp: None,
            },
        }
    }
}

#[derive(Deserialize)]
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
}

#[derive(Serialize)]
pub struct MyIssueItem {
    pub id: uuid::Uuid,
    pub display_key: String,
    pub title: String,
    pub status: String,
    pub priority: String,
}

impl MyIssueItem {
    fn from_row(row: MyIssueRow) -> Self {
        Self {
            id: row.id,
            display_key: format!("{}-{}", row.project_key, row.key_seq),
            title: row.title,
            status: row.status,
            priority: row.priority,
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct MyIssueFacets {
    pub status: HashMap<String, i64>,
    pub priority: HashMap<String, i64>,
}

#[derive(FromRow)]
struct FacetRow {
    value: String,
    count: i64,
}

#[derive(Serialize)]
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

fn apply_my_issues_filters(
    builder: &mut QueryBuilder<Postgres>,
    query: &ListMyIssuesQuery,
    exclude_facet: Option<&str>,
) {
    if exclude_facet != Some("status") {
        if let Some(s) = query.status.as_deref() {
            let values: Vec<&str> = s.split(',').map(str::trim).filter(|x| !x.is_empty()).collect();
            if !values.is_empty() {
                builder.push(" AND i.status IN (");
                for (i, v) in values.iter().enumerate() {
                    if i > 0 {
                        builder.push(", ");
                    }
                    builder.push_bind((*v).to_string());
                }
                builder.push(")");
            }
        }
    }

    if exclude_facet != Some("priority") {
        if let Some(s) = query.priority.as_deref() {
            let values: Vec<&str> = s.split(',').map(str::trim).filter(|x| !x.is_empty()).collect();
            if !values.is_empty() {
                builder.push(" AND i.priority IN (");
                for (i, v) in values.iter().enumerate() {
                    if i > 0 {
                        builder.push(", ");
                    }
                    builder.push_bind((*v).to_string());
                }
                builder.push(")");
            }
        }
    }

    if let Some(search) = query.q.as_deref() {
        if !search.trim().is_empty() {
            builder
                .push(" AND (i.title ILIKE '%' || ")
                .push_bind(search.to_string())
                .push(" || '%' OR COALESCE(i.description, '') ILIKE '%' || ")
                .push_bind(search.to_string())
                .push(" || '%')");
        }
    }
}

fn apply_my_issues_cursor_filter(
    builder: &mut QueryBuilder<Postgres>,
    sort_by: MyIssueSortBy,
    sort_dir: SortDirection,
    cursor: &MyIssuesCursor,
) -> Result<(), &'static str> {
    let comparison = sort_dir.comparison_sql();

    match sort_by {
        MyIssueSortBy::KeySeq => {
            let value = cursor
                .sort_int
                .ok_or("cursor is missing integer sort value")?;
            builder
                .push(" AND ((i.key_seq ")
                .push(comparison)
                .push(" ")
                .push_bind(value)
                .push(") OR (i.key_seq = ")
                .push_bind(value)
                .push(" AND i.id ")
                .push(comparison)
                .push(" ")
                .push_bind(cursor.id)
                .push("))");
        }
        MyIssueSortBy::CreatedAt | MyIssueSortBy::UpdatedAt => {
            let value = cursor
                .sort_timestamp
                .ok_or("cursor is missing timestamp sort value")?;
            let col = sort_by.as_sql();
            builder
                .push(" AND ((")
                .push(col)
                .push(" ")
                .push(comparison)
                .push(" ")
                .push_bind(value)
                .push(") OR (")
                .push(col)
                .push(" = ")
                .push_bind(value)
                .push(" AND i.id ")
                .push(comparison)
                .push(" ")
                .push_bind(cursor.id)
                .push("))");
        }
        MyIssueSortBy::Title | MyIssueSortBy::Status | MyIssueSortBy::Priority => {
            let value = cursor
                .sort_text
                .as_deref()
                .ok_or("cursor is missing text sort value")?
                .to_string();
            let col = sort_by.as_sql();
            builder
                .push(" AND ((")
                .push(col)
                .push(" ")
                .push(comparison)
                .push(" ")
                .push_bind(value.clone())
                .push(") OR (")
                .push(col)
                .push(" = ")
                .push_bind(value)
                .push(" AND i.id ")
                .push(comparison)
                .push(" ")
                .push_bind(cursor.id)
                .push("))");
        }
    }

    Ok(())
}

fn push_my_issues_user_filter(
    builder: &mut QueryBuilder<Postgres>,
    user_id: uuid::Uuid,
    filter_type: Option<&str>,
) {
    match filter_type {
        Some("created") => {
            builder.push("i.reporter_id = ");
            builder.push_bind(user_id);
        }
        _ => {
            builder.push("i.assignee_id = ");
            builder.push_bind(user_id);
        }
    }
}

async fn fetch_my_issues_facets(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    query: &ListMyIssuesQuery,
) -> Result<MyIssueFacets, sqlx::Error> {
    let mut facets = MyIssueFacets::default();
    let filter_type = query.filter_type.as_deref();

    let mut status_builder = QueryBuilder::<Postgres>::new(
        "SELECT i.status as value, COUNT(*) as count FROM issues i WHERE ",
    );
    push_my_issues_user_filter(&mut status_builder, user_id, filter_type);
    apply_my_issues_filters(&mut status_builder, query, Some("status"));
    status_builder.push(" GROUP BY i.status");
    let rows: Vec<FacetRow> = status_builder
        .build_query_as::<FacetRow>()
        .fetch_all(pool)
        .await?;
    for row in rows {
        facets.status.insert(row.value, row.count);
    }

    let mut priority_builder = QueryBuilder::<Postgres>::new(
        "SELECT i.priority as value, COUNT(*) as count FROM issues i WHERE ",
    );
    push_my_issues_user_filter(&mut priority_builder, user_id, filter_type);
    apply_my_issues_filters(&mut priority_builder, query, Some("priority"));
    priority_builder.push(" GROUP BY i.priority");
    let rows: Vec<FacetRow> = priority_builder
        .build_query_as::<FacetRow>()
        .fetch_all(pool)
        .await?;
    for row in rows {
        facets.priority.insert(row.value, row.count);
    }

    Ok(facets)
}

pub async fn list_my_issues(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(query): Query<ListMyIssuesQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).clamp(1, 1_000);
    let offset = query.offset.unwrap_or(0).max(0);
    let sort_by = query.sort_by.unwrap_or_default();
    let sort_dir = query.sort_dir.unwrap_or_default();

    if query.cursor.is_some() && query.offset.is_some() {
        return (
            StatusCode::BAD_REQUEST,
            "use either cursor or offset, not both",
        )
            .into_response();
    }

    let decoded_cursor = match query.cursor.as_deref() {
        Some(raw) => match serde_json::from_str::<MyIssuesCursor>(raw) {
            Ok(cursor) => Some(cursor),
            Err(_) => return (StatusCode::BAD_REQUEST, "invalid cursor").into_response(),
        },
        None => None,
    };

    let filter_type = query.filter_type.as_deref();

    let mut count_query = QueryBuilder::<Postgres>::new(
        "SELECT COUNT(*) FROM issues i WHERE ",
    );
    push_my_issues_user_filter(&mut count_query, user_id, filter_type);
    apply_my_issues_filters(&mut count_query, &query, None);

    let total = match count_query
        .build_query_scalar::<i64>()
        .fetch_one(&state.db)
        .await
    {
        Ok(total) => total,
        Err(error) => {
            eprintln!("list_my_issues count error: {error:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let facets = match fetch_my_issues_facets(&state.db, user_id, &query).await {
        Ok(f) => f,
        Err(error) => {
            eprintln!("list_my_issues facet count error: {error:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut items_query = QueryBuilder::<Postgres>::new(
        "SELECT i.id, p.project_key, i.key_seq, i.title, i.status, i.priority, i.created_at, i.updated_at \
         FROM issues i \
         JOIN projects p ON p.id = i.project_id \
         WHERE ",
    );
    push_my_issues_user_filter(&mut items_query, user_id, filter_type);
    apply_my_issues_filters(&mut items_query, &query, None);

    if let Some(cursor) = decoded_cursor.as_ref() {
        if let Err(error) = apply_my_issues_cursor_filter(&mut items_query, sort_by, sort_dir, cursor) {
            return (StatusCode::BAD_REQUEST, error).into_response();
        }
    }

    items_query
        .push(" ORDER BY ")
        .push(sort_by.as_sql())
        .push(" ")
        .push(sort_dir.as_sql())
        .push(", i.id ")
        .push(sort_dir.as_sql())
        .push(" LIMIT ")
        .push_bind(limit + 1);

    if decoded_cursor.is_none() {
        items_query.push(" OFFSET ").push_bind(offset);
    }

    let rows = match items_query
        .build_query_as::<MyIssueRow>()
        .fetch_all(&state.db)
        .await
    {
        Ok(rows) => rows,
        Err(error) => {
            eprintln!("list_my_issues query error: {error:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let has_more = rows.len() as i64 > limit;
    let mut rows = rows;

    if has_more {
        rows.pop();
    }

    let next_cursor = if has_more {
        match rows.last() {
            Some(last_row) => {
                match serde_json::to_string(&MyIssuesCursor::from_row(sort_by, last_row)) {
                    Ok(cursor) => Some(cursor),
                    Err(error) => {
                        eprintln!("list_my_issues cursor encode error: {error:?}");
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                }
            }
            None => None,
        }
    } else {
        None
    };

    let items = rows.into_iter().map(MyIssueItem::from_row).collect();

    (
        StatusCode::OK,
        Json(ListMyIssuesResponse {
            items,
            total,
            limit,
            offset,
            next_cursor,
            has_more,
            sort_by,
            sort_dir,
            facets,
        }),
    )
        .into_response()
}

pub async fn list_issues(
    Path(project_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(q): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // project member check
    let is_member = match sqlx::query_scalar::<_, i32>(
        r#"
        SELECT 1
        FROM project_members
        WHERE project_id = $1 AND user_id = $2
        "#,
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("list_issues member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    // project_key lazım (display_key için)
    let project_key =
        match sqlx::query_scalar::<_, String>("SELECT project_key FROM projects WHERE id = $1")
            .bind(project_id)
            .fetch_optional(&state.db)
            .await
        {
            Ok(Some(k)) => k,
            Ok(None) => return StatusCode::NOT_FOUND.into_response(),
            Err(e) => {
                eprintln!("list_issues project_key error: {e:?}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

    let status_filter: Option<String> = q.get("status").cloned();

    let q_filter: Option<String> = q.get("q").cloned();

    // assignee=me | assignee=<uuid>
    let assignee_filter: Option<uuid::Uuid> = match q.get("assignee").map(|s| s.as_str()) {
        Some("me") => Some(user_id),
        Some(raw) => raw.parse::<uuid::Uuid>().ok(),
        None => None,
    };

    // limit/offset
    let limit: i64 = q
        .get("limit")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(50)
        .clamp(1, 200);

    let offset: i64 = q
        .get("offset")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0)
        .max(0);

    let rows = match sqlx::query!(
        r#"
        SELECT id, key_seq, title, status
        FROM issues
        WHERE project_id = $1
            AND ($2::text IS NULL OR status = $2)
            AND ($3::uuid IS NULL OR assignee_id = $3)
            AND (
                $4::text IS NULL
                OR title ILIKE '%' || $4 || '%'
                OR COALESCE(description, '') ILIKE '%' || $4 || '%'
            )
        ORDER BY key_seq DESC
        LIMIT $5
        OFFSET $6
        "#,
        project_id,
        status_filter,
        assignee_filter,
        q_filter,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("list_issues query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let items = rows
        .into_iter()
        .map(|r| IssueListItem {
            issue_id: r.id,
            display_key: format!("{}-{}", project_key, r.key_seq),
            title: r.title,
            status: r.status,
        })
        .collect();

    (StatusCode::OK, Json(ListIssuesResponse { items })).into_response()
}

pub async fn list_project_issues_by_key(
    Path((org_slug, project_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(q): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // Find project by org_slug and project_key
    let project = match sqlx::query!(
        r#"
        SELECT p.id, p.project_key
        FROM projects p
        JOIN organizations o ON o.id = p.org_id
        WHERE o.slug = $1 AND p.project_key = $2
        "#,
        org_slug,
        project_key
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(p)) => p,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("list_project_issues_by_key project lookup error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let project_id = project.id;

    // project member check
    let is_member = match sqlx::query_scalar::<_, i32>(
        r#"
        SELECT 1
        FROM project_members
        WHERE project_id = $1 AND user_id = $2
        "#,
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("list_project_issues_by_key member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    let status_filter: Option<String> = q.get("status").cloned();
    let q_filter: Option<String> = q.get("q").cloned();

    let assignee_filter: Option<uuid::Uuid> = match q.get("assignee").map(|s| s.as_str()) {
        Some("me") => Some(user_id),
        Some(raw) => raw.parse::<uuid::Uuid>().ok(),
        None => None,
    };

    let limit: i64 = q
        .get("limit")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(50)
        .clamp(1, 200);

    let offset: i64 = q
        .get("offset")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0)
        .max(0);

    // Get total count
    let total: i64 = match sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM issues
        WHERE project_id = $1
            AND ($2::text IS NULL OR status = $2)
            AND ($3::uuid IS NULL OR assignee_id = $3)
            AND (
                $4::text IS NULL
                OR title ILIKE '%' || $4 || '%'
                OR COALESCE(description, '') ILIKE '%' || $4 || '%'
            )
        "#,
        project_id,
        status_filter.clone(),
        assignee_filter,
        q_filter.clone(),
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("list_project_issues_by_key count error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let rows = match sqlx::query!(
        r#"
        SELECT id, key_seq, title, status
        FROM issues
        WHERE project_id = $1
            AND ($2::text IS NULL OR status = $2)
            AND ($3::uuid IS NULL OR assignee_id = $3)
            AND (
                $4::text IS NULL
                OR title ILIKE '%' || $4 || '%'
                OR COALESCE(description, '') ILIKE '%' || $4 || '%'
            )
        ORDER BY key_seq DESC
        LIMIT $5
        OFFSET $6
        "#,
        project_id,
        status_filter,
        assignee_filter,
        q_filter,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("list_project_issues_by_key query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let items: Vec<IssueListItem> = rows
        .into_iter()
        .map(|r| IssueListItem {
            issue_id: r.id,
            display_key: format!("{}-{}", project.project_key, r.key_seq),
            title: r.title,
            status: r.status,
        })
        .collect();

    let fetched = items.len() as i64;
    let has_more = offset + fetched < total;

    (
        StatusCode::OK,
        Json(ListProjectIssuesResponse {
            items,
            total,
            limit,
            offset,
            has_more,
        }),
    )
        .into_response()
}

#[derive(serde::Deserialize)]
pub struct UpdateIssueStatusRequest {
    pub status: String,
}

#[derive(serde::Serialize)]
pub struct UpdateIssueStatusResponse {
    pub issue_id: uuid::Uuid,
    pub status: String,
}

pub async fn update_issue_status(
    Path(issue_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<UpdateIssueStatusRequest>,
) -> impl IntoResponse {
    let status = req.status;

    let allowed = ["backlog", "todo", "in_progress", "done"];
    if !allowed.contains(&status.as_str()) {
        return (StatusCode::BAD_REQUEST, "invalid status").into_response();
    }

    // issue -> project_id çek
    let issue_row = match sqlx::query!(r#"SELECT project_id FROM issues WHERE id = $1"#, issue_id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("update_issue_status select issue error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // project member check
    let is_member = match sqlx::query_scalar::<_, i32>(
        r#"
        SELECT 1
        FROM project_members
        WHERE project_id = $1 AND user_id = $2
        "#,
    )
    .bind(issue_row.project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("update_issue_status member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    // update
    if let Err(e) = sqlx::query!(
        r#"UPDATE issues SET status = $1 WHERE id = $2"#,
        status,
        issue_id
    )
    .execute(&state.db)
    .await
    {
        eprintln!("update_issue_status update error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (
        StatusCode::OK,
        Json(UpdateIssueStatusResponse { issue_id, status }),
    )
        .into_response()
}

#[derive(serde::Deserialize)]
pub struct UpdateIssueRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
}

#[derive(serde::Serialize)]
pub struct UpdateIssueResponse {
    pub issue_id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub priority: String,
}

pub async fn update_issue(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<UpdateIssueRequest>,
) -> impl IntoResponse {
    // Parse issue key (e.g., "APP-123")
    let (project_key, key_seq) = match issue_key.split_once('-') {
        Some((pk, seq_str)) => match seq_str.parse::<i32>() {
            Ok(seq) => (pk, seq),
            Err(_) => return (StatusCode::BAD_REQUEST, "invalid issue key").into_response(),
        },
        None => return (StatusCode::BAD_REQUEST, "invalid issue key").into_response(),
    };

    // Resolve org
    let org_id = match sqlx::query_scalar::<_, uuid::Uuid>(
        r#"SELECT id FROM organizations WHERE slug = $1"#,
    )
    .bind(&org_slug)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("update_issue org resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Resolve issue
    let issue_row = match sqlx::query!(
        r#"
        SELECT i.id as issue_id, i.project_id, i.title, i.description, i.priority
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        WHERE p.org_id = $1 AND p.project_key = $2 AND i.key_seq = $3
        "#,
        org_id,
        project_key,
        key_seq
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("update_issue issue resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Check project membership
    let is_member = match sqlx::query_scalar::<_, i32>(
        r#"SELECT 1 FROM project_members WHERE project_id = $1 AND user_id = $2"#,
    )
    .bind(issue_row.project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("update_issue member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    // Validate priority if provided
    let allowed_priorities = ["critical", "high", "medium", "low"];
    if let Some(ref p) = req.priority {
        if !allowed_priorities.contains(&p.as_str()) {
            return (StatusCode::BAD_REQUEST, "invalid priority").into_response();
        }
    }

    // Update issue
    let new_title = req.title.unwrap_or(issue_row.title);
    let new_description = req.description.or(issue_row.description);
    let new_priority = req.priority.unwrap_or(issue_row.priority);

    if let Err(e) = sqlx::query!(
        r#"UPDATE issues SET title = $1, description = $2, priority = $3 WHERE id = $4"#,
        new_title,
        new_description,
        new_priority,
        issue_row.issue_id
    )
    .execute(&state.db)
    .await
    {
        eprintln!("update_issue update error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (
        StatusCode::OK,
        Json(UpdateIssueResponse {
            issue_id: issue_row.issue_id,
            title: new_title,
            description: new_description,
            priority: new_priority,
        }),
    )
        .into_response()
}

#[derive(serde::Serialize)]
pub struct BoardResponse {
    pub columns: BTreeMap<String, Vec<IssueListItem>>,
}

pub async fn get_board(
    Path(project_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl IntoResponse {
    // member check
    let is_member = match sqlx::query_scalar::<_, i32>(
        r#"SELECT 1 FROM project_members WHERE project_id = $1 AND user_id = $2"#,
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("get_board member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    // project_key
    let project_key =
        match sqlx::query_scalar::<_, String>("SELECT project_key FROM projects WHERE id = $1")
            .bind(project_id)
            .fetch_optional(&state.db)
            .await
        {
            Ok(Some(k)) => k,
            Ok(None) => return StatusCode::NOT_FOUND.into_response(),
            Err(e) => {
                eprintln!("get_board project_key error: {e:?}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

    // tüm issue'ları çek
    let rows = match sqlx::query!(
        r#"
        SELECT id, key_seq, title, status
        FROM issues
        WHERE project_id = $1
        ORDER BY key_seq DESC
        "#,
        project_id
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("get_board query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // kolonları hazırla (boş da olsa dönsün)
    let mut columns: BTreeMap<String, Vec<IssueListItem>> = BTreeMap::new();
    for s in ["backlog", "todo", "in_progress", "done"] {
        columns.insert(s.to_string(), Vec::new());
    }

    for r in rows {
        let item = IssueListItem {
            issue_id: r.id,
            display_key: format!("{}-{}", project_key, r.key_seq),
            title: r.title,
            status: r.status.clone(),
        };

        columns.entry(r.status).or_default().push(item);
    }

    (StatusCode::OK, Json(BoardResponse { columns })).into_response()
}

pub async fn get_board_by_key(
    Path((org_slug, project_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl IntoResponse {
    // Find project by org_slug and project_key
    let project = match sqlx::query!(
        r#"
        SELECT p.id, p.project_key
        FROM projects p
        JOIN organizations o ON o.id = p.org_id
        WHERE o.slug = $1 AND p.project_key = $2
        "#,
        org_slug,
        project_key
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(p)) => p,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("get_board_by_key project lookup error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let project_id = project.id;

    // member check
    let is_member = match sqlx::query_scalar::<_, i32>(
        r#"SELECT 1 FROM project_members WHERE project_id = $1 AND user_id = $2"#,
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("get_board_by_key member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    // tüm issue'ları çek (board_order'a göre sırala)
    let rows = match sqlx::query!(
        r#"
        SELECT id, key_seq, title, status
        FROM issues
        WHERE project_id = $1
        ORDER BY board_order ASC, key_seq DESC
        "#,
        project_id
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("get_board_by_key query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // kolonları hazırla (boş da olsa dönsün)
    let mut columns: BTreeMap<String, Vec<IssueListItem>> = BTreeMap::new();
    for s in ["backlog", "todo", "in_progress", "done"] {
        columns.insert(s.to_string(), Vec::new());
    }

    for r in rows {
        let item = IssueListItem {
            issue_id: r.id,
            display_key: format!("{}-{}", project.project_key, r.key_seq),
            title: r.title,
            status: r.status.clone(),
        };

        columns.entry(r.status).or_default().push(item);
    }

    (StatusCode::OK, Json(BoardResponse { columns })).into_response()
}

#[derive(serde::Deserialize)]
pub struct UpdateIssueBoardPositionRequest {
    pub status: String,
    pub position: i32,
}

pub async fn update_issue_board_position(
    Path(issue_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<UpdateIssueBoardPositionRequest>,
) -> impl IntoResponse {
    let allowed_statuses = ["backlog", "todo", "in_progress", "done"];
    if !allowed_statuses.contains(&req.status.as_str()) {
        return (StatusCode::BAD_REQUEST, "invalid status").into_response();
    }

    // Get issue and project_id
    let issue_row = match sqlx::query!(
        r#"SELECT project_id, status as old_status FROM issues WHERE id = $1"#,
        issue_id
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("update_issue_board_position select issue error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // member check
    let is_member = match sqlx::query_scalar::<_, i32>(
        r#"SELECT 1 FROM project_members WHERE project_id = $1 AND user_id = $2"#,
    )
    .bind(issue_row.project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("update_issue_board_position member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    // Update issue status and board_order
    if let Err(e) = sqlx::query!(
        r#"
        UPDATE issues
        SET status = $1, board_order = $2, updated_at = NOW()
        WHERE id = $3
        "#,
        req.status,
        req.position,
        issue_id
    )
    .execute(&state.db)
    .await
    {
        eprintln!("update_issue_board_position update error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "issue_id": issue_id,
            "status": req.status,
            "position": req.position
        })),
    )
        .into_response()
}

#[derive(serde::Serialize)]
pub struct IssueDetailResponse {
    pub issue_id: uuid::Uuid,
    pub display_key: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub assignee_id: Option<uuid::Uuid>,
    pub assignee_name: Option<String>,
    pub project_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

fn parse_issue_key(key: &str) -> Option<(&str, i32)> {
    let (project_key, seq_str) = key.split_once('-')?;
    let seq = seq_str.parse::<i32>().ok()?;
    Some((project_key, seq))
}

pub async fn get_issue(
    Path(issue_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl IntoResponse {
    // issue + project_key + project_id çek
    let row = match sqlx::query!(
        r#"
        SELECT
            i.id,
            i.project_id,
            i.key_seq,
            i.title,
            i.description,
            i.status,
            i.priority,
            i.assignee_id,
            i.created_at,
            i.updated_at,
            p.project_key,
            u.name as "assignee_name?"
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        LEFT JOIN users u ON u.id = i.assignee_id
        WHERE i.id = $1
        "#,
        issue_id
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("get_issue query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // authz: project member mı?
    let is_member = match sqlx::query_scalar::<_, i32>(
        r#"SELECT 1 FROM project_members WHERE project_id = $1 AND user_id = $2"#,
    )
    .bind(row.project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("get_issue member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    let display_key = format!("{}-{}", row.project_key, row.key_seq);

    (
        StatusCode::OK,
        Json(IssueDetailResponse {
            issue_id: row.id,
            display_key,
            title: row.title,
            description: row.description,
            status: row.status,
            priority: row.priority,
            assignee_id: row.assignee_id,
            assignee_name: row.assignee_name,
            project_id: row.project_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }),
    )
        .into_response()
}

pub async fn get_issue_by_key(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl IntoResponse {
    let (project_key, key_seq) = match parse_issue_key(&issue_key) {
        Some(v) => v,
        None => return (StatusCode::BAD_REQUEST, "invalid issue key").into_response(),
    };

    // issue + org match + project_key match
    let row = match sqlx::query!(
        r#"
        SELECT
            i.id,
            i.project_id,
            i.key_seq,
            i.title,
            i.description,
            i.status,
            i.priority,
            i.assignee_id,
            i.created_at,
            i.updated_at,
            p.project_key,
            o.id as org_id,
            u.name as "assignee_name?"
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        JOIN organizations o ON o.id = p.org_id
        LEFT JOIN users u ON u.id = i.assignee_id
        WHERE o.slug = $1
          AND p.project_key = $2
          AND i.key_seq = $3
        "#,
        org_slug,
        project_key,
        key_seq
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("get_issue_by_key query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // authz: member mi?
    let is_member = match sqlx::query_scalar::<_, i32>(
        r#"SELECT 1 FROM project_members WHERE project_id = $1 AND user_id = $2"#,
    )
    .bind(row.project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("get_issue_by_key member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    let display_key = format!("{}-{}", row.project_key, row.key_seq);

    (
        StatusCode::OK,
        Json(IssueDetailResponse {
            issue_id: row.id,
            display_key,
            title: row.title,
            description: row.description,
            status: row.status,
            priority: row.priority,
            assignee_id: row.assignee_id,
            assignee_name: row.assignee_name,
            project_id: row.project_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }),
    )
        .into_response()
}

#[derive(Deserialize)]
pub struct AssignIssueRequest {
    pub user_id: uuid::Uuid,
}

pub async fn assign_issue(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(_actor_id): AuthUser,
    Json(req): Json<AssignIssueRequest>,
) -> impl IntoResponse {
    let (project_key, key_seq) = match parse_issue_key(&issue_key) {
        Some(v) => v,
        None => return (StatusCode::BAD_REQUEST, "invalid issue key").into_response(),
    };

    // org_id
    let org_id =
        match sqlx::query_scalar::<_, uuid::Uuid>("SELECT id FROM organizations WHERE slug = $1")
            .bind(&org_slug)
            .fetch_optional(&state.db)
            .await
        {
            Ok(Some(id)) => id,
            Ok(None) => return StatusCode::NOT_FOUND.into_response(),
            Err(e) => {
                eprintln!("assign_issue org resolve error: {e:?}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

    // issue_id + project_id
    let issue_row = match sqlx::query!(
        r#"
        SELECT i.id as issue_id, i.project_id
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        WHERE p.org_id = $1 AND p.project_key = $2 AND i.key_seq = $3
        "#,
        org_id,
        project_key,
        key_seq
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("assign_issue issue resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // enforce: assignee project member mı?
    let is_project_member = match sqlx::query_scalar::<_, i32>(
        r#"SELECT 1 FROM project_members WHERE project_id = $1 AND user_id = $2"#,
    )
    .bind(issue_row.project_id)
    .bind(req.user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("assign_issue member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_project_member {
        return (StatusCode::BAD_REQUEST, "assignee must be a project member").into_response();
    }

    // update
    if let Err(e) = sqlx::query!(
        r#"UPDATE issues SET assignee_id = $1 WHERE id = $2"#,
        req.user_id,
        issue_row.issue_id
    )
    .execute(&state.db)
    .await
    {
        eprintln!("assign_issue update error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    StatusCode::NO_CONTENT.into_response()
}

pub async fn assign_me(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(actor_id): AuthUser,
) -> impl IntoResponse {
    // reuse patch logic without body: set assignee_id = actor_id
    let (project_key, key_seq) = match parse_issue_key(&issue_key) {
        Some(v) => v,
        None => return (StatusCode::BAD_REQUEST, "invalid issue key").into_response(),
    };

    let org_id =
        match sqlx::query_scalar::<_, uuid::Uuid>("SELECT id FROM organizations WHERE slug = $1")
            .bind(&org_slug)
            .fetch_optional(&state.db)
            .await
        {
            Ok(Some(id)) => id,
            Ok(None) => return StatusCode::NOT_FOUND.into_response(),
            Err(e) => {
                eprintln!("assign_me org resolve error: {e:?}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

    let issue_row = match sqlx::query!(
        r#"
        SELECT i.id as issue_id, i.project_id
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        WHERE p.org_id = $1 AND p.project_key = $2 AND i.key_seq = $3
        "#,
        org_id,
        project_key,
        key_seq
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("assign_me issue resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // enforce: actor project member mı
    let is_project_member = match sqlx::query_scalar::<_, i32>(
        r#"SELECT 1 FROM project_members WHERE project_id = $1 AND user_id = $2"#,
    )
    .bind(issue_row.project_id)
    .bind(actor_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("assign_me member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_project_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    if let Err(e) = sqlx::query!(
        r#"UPDATE issues SET assignee_id = $1 WHERE id = $2"#,
        actor_id,
        issue_row.issue_id
    )
    .execute(&state.db)
    .await
    {
        eprintln!("assign_me update error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let payload = serde_json::json!({ "assignee_id": actor_id });

    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO issue_events (org_id, issue_id, actor_id, event_type, payload)
        VALUES ($1,$2,$3,$4,$5)
        "#,
        org_id,
        issue_row.issue_id,
        actor_id, // assign_issue’de actor yoksa AuthUser'dan al
        "assigned",
        payload
    )
    .execute(&state.db)
    .await
    {
        eprintln!("assigned event insert error: {e:?}");
    }

    StatusCode::NO_CONTENT.into_response()
}

pub async fn unassign_issue(
    Path((org_slug, issue_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(actor_id): AuthUser,
) -> impl IntoResponse {
    let (project_key, key_seq) = match parse_issue_key(&issue_key) {
        Some(v) => v,
        None => return (StatusCode::BAD_REQUEST, "invalid issue key").into_response(),
    };

    let org_id =
        match sqlx::query_scalar::<_, uuid::Uuid>("SELECT id FROM organizations WHERE slug = $1")
            .bind(&org_slug)
            .fetch_optional(&state.db)
            .await
        {
            Ok(Some(id)) => id,
            Ok(None) => return StatusCode::NOT_FOUND.into_response(),
            Err(e) => {
                eprintln!("unassign org resolve error: {e:?}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

    let issue_id = match sqlx::query_scalar::<_, uuid::Uuid>(
        r#"
        SELECT i.id
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        WHERE p.org_id = $1 AND p.project_key = $2 AND i.key_seq = $3
        "#,
    )
    .bind(org_id)
    .bind(project_key)
    .bind(key_seq)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("unassign issue resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if let Err(e) = sqlx::query!(
        r#"UPDATE issues SET assignee_id = NULL WHERE id = $1"#,
        issue_id
    )
    .execute(&state.db)
    .await
    {
        eprintln!("unassign update error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let payload = serde_json::json!({});

    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO issue_events (org_id, issue_id, actor_id, event_type, payload)
        VALUES ($1,$2,$3,$4,$5)
        "#,
        org_id,
        issue_id,
        actor_id,
        "unassigned",
        payload
    )
    .execute(&state.db)
    .await
    {
        eprintln!("unassigned event insert error: {e:?}");
    }

    StatusCode::NO_CONTENT.into_response()
}
