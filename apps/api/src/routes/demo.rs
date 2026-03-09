use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};

use crate::state::AppState;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DemoTaskSortBy {
    SortOrder,
    CreatedAt,
    Id,
    Title,
    Status,
    Label,
    Priority,
}

impl Default for DemoTaskSortBy {
    fn default() -> Self {
        Self::SortOrder
    }
}

impl DemoTaskSortBy {
    fn as_sql(self) -> &'static str {
        match self {
            Self::SortOrder => "sort_order",
            Self::CreatedAt => "created_at",
            Self::Id => "id",
            Self::Title => "title",
            Self::Status => "status",
            Self::Label => "label",
            Self::Priority => "priority",
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
        Self::Asc
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
struct DemoTasksCursor {
    id: String,
    sort_text: Option<String>,
    sort_int: Option<i32>,
    sort_timestamp: Option<DateTime<Utc>>,
}

impl DemoTasksCursor {
    fn from_row(sort_by: DemoTaskSortBy, row: &DemoTaskRow) -> Self {
        match sort_by {
            DemoTaskSortBy::SortOrder => Self {
                id: row.id.clone(),
                sort_text: None,
                sort_int: Some(row.sort_order),
                sort_timestamp: None,
            },
            DemoTaskSortBy::CreatedAt => Self {
                id: row.id.clone(),
                sort_text: None,
                sort_int: None,
                sort_timestamp: Some(row.created_at),
            },
            DemoTaskSortBy::Id => Self {
                id: row.id.clone(),
                sort_text: Some(row.id.clone()),
                sort_int: None,
                sort_timestamp: None,
            },
            DemoTaskSortBy::Title => Self {
                id: row.id.clone(),
                sort_text: Some(row.title.clone()),
                sort_int: None,
                sort_timestamp: None,
            },
            DemoTaskSortBy::Status => Self {
                id: row.id.clone(),
                sort_text: Some(row.status.clone()),
                sort_int: None,
                sort_timestamp: None,
            },
            DemoTaskSortBy::Label => Self {
                id: row.id.clone(),
                sort_text: Some(row.label.clone()),
                sort_int: None,
                sort_timestamp: None,
            },
            DemoTaskSortBy::Priority => Self {
                id: row.id.clone(),
                sort_text: Some(row.priority.clone()),
                sort_int: None,
                sort_timestamp: None,
            },
        }
    }
}

#[derive(Deserialize)]
pub struct ListDemoTasksQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub cursor: Option<String>,
    pub q: Option<String>,
    pub status: Option<String>,
    pub label: Option<String>,
    pub priority: Option<String>,
    pub sort_by: Option<DemoTaskSortBy>,
    pub sort_dir: Option<SortDirection>,
}

#[derive(Serialize, FromRow)]
pub struct DemoTaskItem {
    pub id: String,
    pub title: String,
    pub status: String,
    pub label: String,
    pub priority: String,
}

#[derive(FromRow)]
struct DemoTaskRow {
    id: String,
    title: String,
    status: String,
    label: String,
    priority: String,
    sort_order: i32,
    created_at: DateTime<Utc>,
}

impl From<DemoTaskRow> for DemoTaskItem {
    fn from(row: DemoTaskRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            status: row.status,
            label: row.label,
            priority: row.priority,
        }
    }
}

#[derive(Serialize)]
pub struct ListDemoTasksResponse {
    pub items: Vec<DemoTaskItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub next_cursor: Option<String>,
    pub has_more: bool,
    pub sort_by: DemoTaskSortBy,
    pub sort_dir: SortDirection,
}

pub async fn list_demo_tasks(
    State(state): State<AppState>,
    Query(query): Query<ListDemoTasksQuery>,
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
        Some(raw) => match serde_json::from_str::<DemoTasksCursor>(raw) {
            Ok(cursor) => Some(cursor),
            Err(_) => return (StatusCode::BAD_REQUEST, "invalid cursor").into_response(),
        },
        None => None,
    };

    let mut count_query =
        QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM demo_tasks WHERE 1=1");
    apply_filters(&mut count_query, &query);

    let total = match count_query
        .build_query_scalar::<i64>()
        .fetch_one(&state.db)
        .await
    {
        Ok(total) => total,
        Err(error) => {
            eprintln!("list_demo_tasks count error: {error:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut items_query = QueryBuilder::<Postgres>::new(
        "SELECT id, title, status, label, priority, sort_order, created_at FROM demo_tasks WHERE 1=1",
    );
    apply_filters(&mut items_query, &query);

    if let Some(cursor) = decoded_cursor.as_ref() {
        if let Err(error) = apply_cursor_filter(&mut items_query, sort_by, sort_dir, cursor) {
            return (StatusCode::BAD_REQUEST, error).into_response();
        }
    }

    items_query
        .push(" ORDER BY ")
        .push(sort_by.as_sql())
        .push(" ")
        .push(sort_dir.as_sql())
        .push(", id ")
        .push(sort_dir.as_sql())
        .push(" LIMIT ")
        .push_bind(limit + 1);

    if decoded_cursor.is_none() {
        items_query.push(" OFFSET ").push_bind(offset);
    }

    let rows = match items_query
        .build_query_as::<DemoTaskRow>()
        .fetch_all(&state.db)
        .await
    {
        Ok(rows) => rows,
        Err(error) => {
            eprintln!("list_demo_tasks query error: {error:?}");
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
                match serde_json::to_string(&DemoTasksCursor::from_row(sort_by, last_row)) {
                    Ok(cursor) => Some(cursor),
                    Err(error) => {
                        eprintln!("list_demo_tasks cursor encode error: {error:?}");
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                }
            }
            None => None,
        }
    } else {
        None
    };

    let items = rows.into_iter().map(DemoTaskItem::from).collect();

    (
        StatusCode::OK,
        Json(ListDemoTasksResponse {
            items,
            total,
            limit,
            offset,
            next_cursor,
            has_more,
            sort_by,
            sort_dir,
        }),
    )
        .into_response()
}

fn apply_filters(builder: &mut QueryBuilder<Postgres>, query: &ListDemoTasksQuery) {
    if let Some(status) = query.status.as_deref() {
        builder.push(" AND status = ").push_bind(status.to_string());
    }

    if let Some(label) = query.label.as_deref() {
        builder.push(" AND label = ").push_bind(label.to_string());
    }

    if let Some(priority) = query.priority.as_deref() {
        builder
            .push(" AND priority = ")
            .push_bind(priority.to_string());
    }

    if let Some(search) = query.q.as_deref() {
        builder
            .push(" AND (id ILIKE '%' || ")
            .push_bind(search.to_string())
            .push(" || '%' OR title ILIKE '%' || ")
            .push_bind(search.to_string())
            .push(" || '%' OR label ILIKE '%' || ")
            .push_bind(search.to_string())
            .push(" || '%')");
    }
}

fn apply_cursor_filter(
    builder: &mut QueryBuilder<Postgres>,
    sort_by: DemoTaskSortBy,
    sort_dir: SortDirection,
    cursor: &DemoTasksCursor,
) -> Result<(), &'static str> {
    let comparison = sort_dir.comparison_sql();

    match sort_by {
        DemoTaskSortBy::SortOrder => {
            let value = cursor
                .sort_int
                .ok_or("cursor is missing integer sort value")?;
            builder
                .push(" AND ((")
                .push(sort_by.as_sql())
                .push(" ")
                .push(comparison)
                .push(" ")
                .push_bind(value)
                .push(") OR (")
                .push(sort_by.as_sql())
                .push(" = ")
                .push_bind(value)
                .push(" AND id ")
                .push(comparison)
                .push(" ")
                .push_bind(cursor.id.clone())
                .push("))");
        }
        DemoTaskSortBy::CreatedAt => {
            let value = cursor
                .sort_timestamp
                .ok_or("cursor is missing timestamp sort value")?
                .to_owned();
            builder
                .push(" AND ((")
                .push(sort_by.as_sql())
                .push(" ")
                .push(comparison)
                .push(" ")
                .push_bind(value)
                .push(") OR (")
                .push(sort_by.as_sql())
                .push(" = ")
                .push_bind(value.clone())
                .push(" AND id ")
                .push(comparison)
                .push(" ")
                .push_bind(cursor.id.clone())
                .push("))");
        }
        DemoTaskSortBy::Id
        | DemoTaskSortBy::Title
        | DemoTaskSortBy::Status
        | DemoTaskSortBy::Label
        | DemoTaskSortBy::Priority => {
            let value = cursor
                .sort_text
                .as_deref()
                .ok_or("cursor is missing text sort value")?
                .to_string();
            builder
                .push(" AND ((")
                .push(sort_by.as_sql())
                .push(" ")
                .push(comparison)
                .push(" ")
                .push_bind(value.clone())
                .push(") OR (")
                .push(sort_by.as_sql())
                .push(" = ")
                .push_bind(value)
                .push(" AND id ")
                .push(comparison)
                .push(" ")
                .push_bind(cursor.id.clone())
                .push("))");
        }
    }

    Ok(())
}
