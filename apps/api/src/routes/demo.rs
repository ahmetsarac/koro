use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListDemoTasksQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub q: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
}

#[derive(Serialize, FromRow)]
pub struct DemoTaskItem {
    pub id: String,
    pub title: String,
    pub status: String,
    pub label: String,
    pub priority: String,
}

#[derive(Serialize)]
pub struct ListDemoTasksResponse {
    pub items: Vec<DemoTaskItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}

pub async fn list_demo_tasks(
    State(state): State<AppState>,
    Query(query): Query<ListDemoTasksQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).clamp(1, 1_000);
    let offset = query.offset.unwrap_or(0).max(0);

    let total = match sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM demo_tasks
        WHERE ($1::text IS NULL OR status = $1)
          AND ($2::text IS NULL OR priority = $2)
          AND (
            $3::text IS NULL
            OR id ILIKE '%' || $3 || '%'
            OR title ILIKE '%' || $3 || '%'
            OR label ILIKE '%' || $3 || '%'
          )
        "#,
    )
    .bind(query.status.as_deref())
    .bind(query.priority.as_deref())
    .bind(query.q.as_deref())
    .fetch_one(&state.db)
    .await
    {
        Ok(total) => total,
        Err(error) => {
            eprintln!("list_demo_tasks count error: {error:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let items = match sqlx::query_as::<_, DemoTaskItem>(
        r#"
        SELECT id, title, status, label, priority
        FROM demo_tasks
        WHERE ($1::text IS NULL OR status = $1)
          AND ($2::text IS NULL OR priority = $2)
          AND (
            $3::text IS NULL
            OR id ILIKE '%' || $3 || '%'
            OR title ILIKE '%' || $3 || '%'
            OR label ILIKE '%' || $3 || '%'
          )
        ORDER BY sort_order ASC
        LIMIT $4
        OFFSET $5
        "#,
    )
    .bind(query.status.as_deref())
    .bind(query.priority.as_deref())
    .bind(query.q.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    {
        Ok(items) => items,
        Err(error) => {
            eprintln!("list_demo_tasks query error: {error:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    (
        StatusCode::OK,
        Json(ListDemoTasksResponse {
            has_more: offset + limit < total,
            items,
            total,
            limit,
            offset,
        }),
    )
        .into_response()
}
