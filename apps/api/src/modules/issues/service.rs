use axum::{Json, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;
use std::collections::BTreeMap;

use crate::modules::{
    events::repository as events_repo,
    orgs::repository as orgs_repo,
};

use super::{
    models::{
        parse_issue_key, AssignIssueRequest, BoardResponse, CreateIssueRequest,
        CreateIssueResponse, IssueDetailResponse, IssueListItem, ListIssuesResponse,
        ListIssuesQuery, ListMyIssuesQuery, ListMyIssuesResponse, ListProjectIssuesResponse,
        MyIssueItem, MyIssuesCursor, UpdateIssueBoardPositionRequest,
        UpdateIssueBoardPositionResponse, UpdateIssueRequest,
        UpdateIssueResponse, UpdateIssueStatusRequest, UpdateIssueStatusResponse,
    },
    repository as issues_repo,
};

// --- Create ----------------------------------------------------------------

pub async fn create_issue(
    pool: &PgPool,
    project_id: uuid::Uuid,
    user_id: uuid::Uuid,
    req: CreateIssueRequest,
) -> impl IntoResponse + use<> {
    if req.title.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "title is required").into_response();
    }

    let is_member = match events_repo::is_project_member(pool, project_id, user_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("create_issue member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let p = match issues_repo::lock_project_for_create(&mut tx, project_id).await {
        Ok(Some(row)) => row,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("create_issue select project error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let key_seq = p.next_issue_seq;

    if let Some(assignee_id) = req.assignee_id {
        let ok = match issues_repo::assignee_is_project_member_tx(&mut tx, project_id, assignee_id).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("create_issue assignee member check error: {e:?}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        if !ok {
            return (StatusCode::BAD_REQUEST, "assignee must be a project member").into_response();
        }
    }

    let status = req.status.as_deref().unwrap_or("backlog");
    let priority = req.priority.as_deref().unwrap_or("none");

    let (issue_id, title) = match issues_repo::insert_issue_returning_id_title(
        &mut tx,
        p.org_id,
        project_id,
        key_seq,
        req.title.clone(),
        req.description.clone(),
        user_id,
        req.assignee_id,
        status,
        priority,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("create_issue insert error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if let Err(e) = issues_repo::increment_project_issue_seq(&mut tx, project_id).await {
        eprintln!("create_issue bump seq error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    if let Err(e) = tx.commit().await {
        eprintln!("create_issue commit error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let display_key = format!("{}-{}", p.project_key, key_seq);
    (
        StatusCode::CREATED,
        Json(CreateIssueResponse {
            issue_id,
            display_key,
            title,
        }),
    )
        .into_response()
}

// --- List types ------------------------------------------------------------

fn my_item_from_row(row: issues_repo::MyIssueRow) -> MyIssueItem {
    MyIssueItem {
        id: row.id,
        display_key: format!("{}-{}", row.project_key, row.key_seq),
        title: row.title,
        status: row.status,
        priority: row.priority,
    }
}

pub async fn list_my_issues(
    pool: &PgPool,
    user_id: uuid::Uuid,
    query: ListMyIssuesQuery,
) -> impl IntoResponse + use<> {
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

    let total = match issues_repo::count_filtered(pool, user_id, &query).await {
        Ok(total) => total,
        Err(error) => {
            eprintln!("list_my_issues count error: {error:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let facets_raw = match issues_repo::fetch_facets(pool, user_id, &query).await {
        Ok(f) => f,
        Err(error) => {
            eprintln!("list_my_issues facet count error: {error:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let rows = match issues_repo::fetch_item_rows(
        pool,
        user_id,
        &query,
        sort_by,
        sort_dir,
        limit + 1,
        offset,
        decoded_cursor.as_ref(),
    )
    .await
    {
        Ok(Ok(rows)) => rows,
        Ok(Err(error)) => return (StatusCode::BAD_REQUEST, error).into_response(),
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
                match serde_json::to_string(&issues_repo::cursor_from_row(sort_by, last_row)) {
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

    let items = rows.into_iter().map(my_item_from_row).collect();

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
            facets: facets_raw,
        }),
    )
        .into_response()
}

// --- Project lists ----------------------------------------------------------

pub async fn list_issues(
    pool: &PgPool,
    project_id: uuid::Uuid,
    user_id: uuid::Uuid,
    q: ListIssuesQuery,
) -> impl IntoResponse + use<> {
    let is_member = match events_repo::is_project_member(pool, project_id, user_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("list_issues member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    let project_key = match issues_repo::project_key_by_id(pool, project_id).await {
        Ok(Some(k)) => k,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("list_issues project_key error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let status_filter = q.status.clone();
    let q_filter = q.q.clone();
    let assignee_filter: Option<uuid::Uuid> = match q.assignee.as_deref() {
        Some("me") => Some(user_id),
        Some(raw) => raw.parse::<uuid::Uuid>().ok(),
        None => None,
    };
    let limit: i64 = q.limit.unwrap_or(50).clamp(1, 200);
    let offset: i64 = q.offset.unwrap_or(0).max(0);

    let rows = match issues_repo::list_issue_summaries_for_project(
        pool,
        project_id,
        status_filter,
        assignee_filter,
        q_filter,
        limit,
        offset,
    )
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
    pool: &PgPool,
    org_slug: String,
    project_key: String,
    user_id: uuid::Uuid,
    q: ListIssuesQuery,
) -> impl IntoResponse + use<> {
    let project = match issues_repo::find_project_by_org_slug_and_key(pool, &org_slug, &project_key).await {
        Ok(Some(p)) => p,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("list_project_issues_by_key project lookup error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let project_id = project.id;

    let is_member = match events_repo::is_project_member(pool, project_id, user_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("list_project_issues_by_key member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    let status_filter = q.status.clone();
    let q_filter = q.q.clone();
    let assignee_filter: Option<uuid::Uuid> = match q.assignee.as_deref() {
        Some("me") => Some(user_id),
        Some(raw) => raw.parse::<uuid::Uuid>().ok(),
        None => None,
    };
    let limit: i64 = q.limit.unwrap_or(50).clamp(1, 200);
    let offset: i64 = q.offset.unwrap_or(0).max(0);

    let total: i64 = match issues_repo::count_issues_for_project_filtered(
        pool,
        project_id,
        status_filter.clone(),
        assignee_filter,
        q_filter.clone(),
    )
    .await
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("list_project_issues_by_key count error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let rows = match issues_repo::list_issue_summaries_for_project(
        pool,
        project_id,
        status_filter,
        assignee_filter,
        q_filter,
        limit,
        offset,
    )
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

// --- Status / patch --------------------------------------------------------

pub async fn update_issue_status(
    pool: &PgPool,
    issue_id: uuid::Uuid,
    user_id: uuid::Uuid,
    req: UpdateIssueStatusRequest,
) -> impl IntoResponse + use<> {
    let status = req.status;

    let allowed = ["backlog", "todo", "in_progress", "done"];
    if !allowed.contains(&status.as_str()) {
        return (StatusCode::BAD_REQUEST, "invalid status").into_response();
    }

    let project_id = match issues_repo::find_issue_project_id_only(pool, issue_id).await {
        Ok(Some(pid)) => pid,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("update_issue_status select issue error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let is_member = match events_repo::is_project_member(pool, project_id, user_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("update_issue_status member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    if let Err(e) = issues_repo::set_issue_status(pool, issue_id, &status).await {
        eprintln!("update_issue_status update error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (
        StatusCode::OK,
        Json(UpdateIssueStatusResponse { issue_id, status }),
    )
        .into_response()
}

pub async fn update_issue(
    pool: &PgPool,
    org_slug: String,
    issue_key: String,
    user_id: uuid::Uuid,
    req: UpdateIssueRequest,
) -> impl IntoResponse + use<> {
    let (project_key, key_seq) = match parse_issue_key(&issue_key) {
        Some(v) => v,
        None => return (StatusCode::BAD_REQUEST, "invalid issue key").into_response(),
    };

    let org_id = match orgs_repo::find_org_id_by_slug(pool, &org_slug).await {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("update_issue org resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let issue_row = match issues_repo::find_issue_for_patch_by_org(pool, org_id, project_key, key_seq).await {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("update_issue issue resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let is_member = match events_repo::is_project_member(pool, issue_row.project_id, user_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("update_issue member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    let allowed_priorities = ["critical", "high", "medium", "low"];
    if let Some(ref p) = req.priority {
        if !allowed_priorities.contains(&p.as_str()) {
            return (StatusCode::BAD_REQUEST, "invalid priority").into_response();
        }
    }

    let new_title = req.title.unwrap_or(issue_row.title);
    let new_description = req.description.or(issue_row.description);
    let new_priority = req.priority.unwrap_or(issue_row.priority);

    if let Err(e) = issues_repo::update_issue_title_desc_priority(
        pool,
        issue_row.issue_id,
        &new_title,
        new_description.clone(),
        &new_priority,
    )
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

// --- Board -----------------------------------------------------------------

fn board_columns_from_rows(project_key: &str, rows: Vec<issues_repo::IssueSummaryRow>) -> BTreeMap<String, Vec<IssueListItem>> {
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
    columns
}

pub async fn get_board(
    pool: &PgPool,
    project_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> impl IntoResponse + use<> {
    let is_member = match events_repo::is_project_member(pool, project_id, user_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("get_board member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    let project_key = match issues_repo::project_key_by_id(pool, project_id).await {
        Ok(Some(k)) => k,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("get_board project_key error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let rows = match issues_repo::list_board_issues_by_key_seq(pool, project_id).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("get_board query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let columns = board_columns_from_rows(&project_key, rows);
    (StatusCode::OK, Json(BoardResponse { columns })).into_response()
}

pub async fn get_board_by_key(
    pool: &PgPool,
    org_slug: String,
    project_key: String,
    user_id: uuid::Uuid,
) -> impl IntoResponse + use<> {
    let project = match issues_repo::find_project_by_org_slug_and_key(pool, &org_slug, &project_key).await {
        Ok(Some(p)) => p,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("get_board_by_key project lookup error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let is_member = match events_repo::is_project_member(pool, project.id, user_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("get_board_by_key member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    let rows = match issues_repo::list_board_issues_by_board_order(pool, project.id).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("get_board_by_key query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let columns = board_columns_from_rows(&project.project_key, rows);
    (StatusCode::OK, Json(BoardResponse { columns })).into_response()
}

pub async fn update_issue_board_position(
    pool: &PgPool,
    issue_id: uuid::Uuid,
    user_id: uuid::Uuid,
    req: UpdateIssueBoardPositionRequest,
) -> impl IntoResponse + use<> {
    let allowed_statuses = ["backlog", "todo", "in_progress", "done"];
    if !allowed_statuses.contains(&req.status.as_str()) {
        return (StatusCode::BAD_REQUEST, "invalid status").into_response();
    }

    let project_id = match issues_repo::find_issue_project_id_only(pool, issue_id).await {
        Ok(Some(pid)) => pid,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("update_issue_board_position select issue error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let is_member = match events_repo::is_project_member(pool, project_id, user_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("update_issue_board_position member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    if let Err(e) =
        issues_repo::update_issue_status_and_board_order(pool, issue_id, &req.status, req.position).await
    {
        eprintln!("update_issue_board_position update error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (
        StatusCode::OK,
        Json(UpdateIssueBoardPositionResponse {
            issue_id,
            status: req.status,
            position: req.position,
        }),
    )
        .into_response()
}

// --- Detail ----------------------------------------------------------------

fn detail_from_row(row: issues_repo::IssueFullRow) -> IssueDetailResponse {
    let display_key = format!("{}-{}", row.project_key, row.key_seq);
    IssueDetailResponse {
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
    }
}

pub async fn get_issue(
    pool: &PgPool,
    issue_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> impl IntoResponse + use<> {
    let row = match issues_repo::find_issue_by_id(pool, issue_id).await {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("get_issue query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let is_member = match events_repo::is_project_member(pool, row.project_id, user_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("get_issue member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    let body = detail_from_row(row);
    (StatusCode::OK, Json(body)).into_response()
}

pub async fn get_issue_by_key(
    pool: &PgPool,
    org_slug: String,
    issue_key: String,
    user_id: uuid::Uuid,
) -> impl IntoResponse + use<> {
    let (project_key, key_seq) = match parse_issue_key(&issue_key) {
        Some(v) => v,
        None => return (StatusCode::BAD_REQUEST, "invalid issue key").into_response(),
    };

    let row = match issues_repo::find_issue_by_org_slug_key(pool, &org_slug, project_key, key_seq).await {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("get_issue_by_key query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let is_member = match events_repo::is_project_member(pool, row.project_id, user_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("get_issue_by_key member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    (StatusCode::OK, Json(detail_from_row(row))).into_response()
}

// --- Assign ----------------------------------------------------------------

pub async fn assign_issue(
    pool: &PgPool,
    org_slug: String,
    issue_key: String,
    req: AssignIssueRequest,
) -> impl IntoResponse + use<> {
    let (project_key, key_seq) = match parse_issue_key(&issue_key) {
        Some(v) => v,
        None => return (StatusCode::BAD_REQUEST, "invalid issue key").into_response(),
    };

    let org_id = match orgs_repo::find_org_id_by_slug(pool, &org_slug).await {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("assign_issue org resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let issue_row = match issues_repo::find_issue_id_in_org(pool, org_id, project_key, key_seq).await {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("assign_issue issue resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let assignee_id = req.user_id;
    let is_project_member = match events_repo::is_project_member(pool, issue_row.project_id, assignee_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("assign_issue member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !is_project_member {
        return (StatusCode::BAD_REQUEST, "assignee must be a project member").into_response();
    }

    if let Err(e) = issues_repo::set_issue_assignee(pool, issue_row.issue_id, assignee_id).await {
        eprintln!("assign_issue update error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    StatusCode::NO_CONTENT.into_response()
}

pub async fn assign_me(
    pool: &PgPool,
    org_slug: String,
    issue_key: String,
    actor_id: uuid::Uuid,
) -> impl IntoResponse + use<> {
    let (project_key, key_seq) = match parse_issue_key(&issue_key) {
        Some(v) => v,
        None => return (StatusCode::BAD_REQUEST, "invalid issue key").into_response(),
    };

    let org_id = match orgs_repo::find_org_id_by_slug(pool, &org_slug).await {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("assign_me org resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let issue_row = match issues_repo::find_issue_id_in_org(pool, org_id, project_key, key_seq).await {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("assign_me issue resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let is_project_member = match events_repo::is_project_member(pool, issue_row.project_id, actor_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("assign_me member check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !is_project_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    if let Err(e) = issues_repo::set_issue_assignee(pool, issue_row.issue_id, actor_id).await {
        eprintln!("assign_me update error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let payload = serde_json::json!({ "assignee_id": actor_id });
    if let Err(e) = issues_repo::insert_assigned_event(pool, org_id, issue_row.issue_id, actor_id, payload).await {
        eprintln!("assigned event insert error: {e:?}");
    }

    StatusCode::NO_CONTENT.into_response()
}

pub async fn unassign_issue(
    pool: &PgPool,
    org_slug: String,
    issue_key: String,
    actor_id: uuid::Uuid,
) -> impl IntoResponse + use<> {
    let (project_key, key_seq) = match parse_issue_key(&issue_key) {
        Some(v) => v,
        None => return (StatusCode::BAD_REQUEST, "invalid issue key").into_response(),
    };

    let org_id = match orgs_repo::find_org_id_by_slug(pool, &org_slug).await {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("unassign org resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let issue_id = match issues_repo::find_issue_id_only_in_org(pool, org_id, project_key, key_seq).await {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("unassign issue resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if let Err(e) = issues_repo::clear_issue_assignee(pool, issue_id).await {
        eprintln!("unassign update error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let payload = serde_json::json!({});
    if let Err(e) = issues_repo::insert_unassigned_event(pool, org_id, issue_id, actor_id, payload).await {
        eprintln!("unassigned event insert error: {e:?}");
    }

    StatusCode::NO_CONTENT.into_response()
}
