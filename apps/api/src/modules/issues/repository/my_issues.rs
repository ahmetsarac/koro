use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};

use crate::modules::issues::models::{
    ListMyIssuesQuery, MyIssueFacets, MyIssueSortBy, MyIssuesCursor, SortDirection,
};

#[derive(FromRow)]
pub struct MyIssueRow {
    pub id: uuid::Uuid,
    pub project_key: String,
    pub key_seq: i32,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub fn cursor_from_row(sort_by: MyIssueSortBy, row: &MyIssueRow) -> MyIssuesCursor {
    match sort_by {
        MyIssueSortBy::CreatedAt => MyIssuesCursor {
            id: row.id,
            sort_text: None,
            sort_int: None,
            sort_timestamp: Some(row.created_at),
        },
        MyIssueSortBy::UpdatedAt => MyIssuesCursor {
            id: row.id,
            sort_text: None,
            sort_int: None,
            sort_timestamp: Some(row.updated_at),
        },
        MyIssueSortBy::KeySeq => MyIssuesCursor {
            id: row.id,
            sort_text: None,
            sort_int: Some(row.key_seq),
            sort_timestamp: None,
        },
        MyIssueSortBy::Title => MyIssuesCursor {
            id: row.id,
            sort_text: Some(row.title.clone()),
            sort_int: None,
            sort_timestamp: None,
        },
        MyIssueSortBy::Status => MyIssuesCursor {
            id: row.id,
            sort_text: Some(row.status.clone()),
            sort_int: None,
            sort_timestamp: None,
        },
        MyIssueSortBy::Priority => MyIssuesCursor {
            id: row.id,
            sort_text: Some(row.priority.clone()),
            sort_int: None,
            sort_timestamp: None,
        },
    }
}

#[derive(FromRow)]
struct FacetRow {
    value: String,
    count: i64,
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

pub fn apply_my_issues_cursor_filter(
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

pub async fn fetch_facets(
    pool: &PgPool,
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

pub async fn count_filtered(
    pool: &PgPool,
    user_id: uuid::Uuid,
    query: &ListMyIssuesQuery,
) -> Result<i64, sqlx::Error> {
    let filter_type = query.filter_type.as_deref();
    let mut count_query = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM issues i WHERE ");
    push_my_issues_user_filter(&mut count_query, user_id, filter_type);
    apply_my_issues_filters(&mut count_query, query, None);
    count_query.build_query_scalar::<i64>().fetch_one(pool).await
}

/// Inner `Err` is a bad cursor message (HTTP 400); outer `Err` is database.
pub async fn fetch_item_rows(
    pool: &PgPool,
    user_id: uuid::Uuid,
    query: &ListMyIssuesQuery,
    sort_by: MyIssueSortBy,
    sort_dir: SortDirection,
    limit_plus_one: i64,
    offset: i64,
    decoded_cursor: Option<&MyIssuesCursor>,
) -> Result<Result<Vec<MyIssueRow>, &'static str>, sqlx::Error> {
    let filter_type = query.filter_type.as_deref();
    let mut items_query = QueryBuilder::<Postgres>::new(
        "SELECT i.id, p.project_key, i.key_seq, i.title, i.status, i.priority, i.created_at, i.updated_at \
         FROM issues i \
         JOIN projects p ON p.id = i.project_id \
         WHERE ",
    );
    push_my_issues_user_filter(&mut items_query, user_id, filter_type);
    apply_my_issues_filters(&mut items_query, query, None);

    if let Some(cursor) = decoded_cursor {
        if let Err(msg) = apply_my_issues_cursor_filter(&mut items_query, sort_by, sort_dir, cursor) {
            return Ok(Err(msg));
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
        .push_bind(limit_plus_one);

    if decoded_cursor.is_none() {
        items_query.push(" OFFSET ").push_bind(offset);
    }

    let rows = items_query.build_query_as::<MyIssueRow>().fetch_all(pool).await?;
    Ok(Ok(rows))
}
