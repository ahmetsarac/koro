use axum::{Json, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;
use std::collections::BTreeMap;

use crate::modules::{
    events::repository as events_repo,
    orgs::repository as orgs_repo,
};

use super::{
    models::{
        parse_issue_key, AssignIssueRequest, BoardColumnDef, BoardResponse,
        CreateIssueRequest, CreateIssueResponse, CreateWorkflowStatusRequest, DeleteWorkflowStatusQuery,
        IssueDetailResponse, IssueListItem, ListIssuesQuery, ListIssuesResponse, ListMyIssuesQuery,
        ListMyIssuesResponse, ListProjectIssuesResponse, ListWorkflowStatusesResponse, MyIssueItem,
        MyIssuesCursor,
        PatchWorkflowStatusRequest, UpdateIssueBoardPositionRequest,
        UpdateIssueBoardPositionResponse, UpdateIssueRequest, UpdateIssueResponse,
        UpdateIssueStatusRequest, UpdateIssueStatusResponse, WorkflowStatusesByCategory,
        WorkflowStatusItem,
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

    let priority = req.priority.as_deref().unwrap_or("none");

    let workflow_status_id = if let Some(wid) = req.workflow_status_id {
        match issues_repo::status_belongs_to_project(&mut *tx, project_id, wid).await {
            Ok(true) => wid,
            Ok(false) => {
                return (StatusCode::BAD_REQUEST, "workflow_status_id not in project").into_response();
            }
            Err(e) => {
                eprintln!("create_issue workflow check error: {e:?}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    } else if let Some(ref slug) = req.workflow_status_slug {
        match issues_repo::resolve_status_id_by_slug(&mut *tx, project_id, slug).await {
            Ok(Some(id)) => id,
            Ok(None) => {
                return (StatusCode::BAD_REQUEST, "unknown workflow_status_slug").into_response();
            }
            Err(e) => {
                eprintln!("create_issue workflow slug error: {e:?}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    } else {
        match issues_repo::default_workflow_status_id_for_project(&mut *tx, project_id).await {
            Ok(Some(id)) => id,
            Ok(None) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, "project has no default workflow status")
                    .into_response();
            }
            Err(e) => {
                eprintln!("create_issue default workflow error: {e:?}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    };

    let (issue_id, title) = match issues_repo::insert_issue_returning_id_title(
        &mut tx,
        p.org_id,
        project_id,
        key_seq,
        req.title.clone(),
        req.description.clone(),
        user_id,
        req.assignee_id,
        workflow_status_id,
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
        project_id: row.project_id,
        display_key: format!("{}-{}", row.project_key, row.key_seq),
        title: row.title,
        status: row.status_slug.clone(),
        workflow_status_id: row.workflow_status_id,
        status_name: row.status_name,
        status_category: row.status_category,
        is_blocked: row.is_blocked,
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

    let status_filter: Option<uuid::Uuid> = q.status.as_deref().and_then(|s| s.parse().ok());
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
            status: r.status_slug.clone(),
            workflow_status_id: r.workflow_status_id,
            status_name: r.status_name,
            status_category: r.status_category,
            is_blocked: r.is_blocked,
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

    let status_filter: Option<uuid::Uuid> = q.status.as_deref().and_then(|s| s.parse().ok());
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
            status: r.status_slug.clone(),
            workflow_status_id: r.workflow_status_id,
            status_name: r.status_name,
            status_category: r.status_category,
            is_blocked: r.is_blocked,
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

    let workflow_status_id = if let Some(wid) = req.workflow_status_id {
        wid
    } else if let Some(ref cat) = req.category {
        match issues_repo::first_status_id_in_category_for_project(pool, project_id, cat).await {
            Ok(Some(id)) => id,
            Ok(None) => {
                return (StatusCode::BAD_REQUEST, "unknown category for project").into_response();
            }
            Err(e) => {
                eprintln!("update_issue_status category resolve error: {e:?}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    } else {
        return (StatusCode::BAD_REQUEST, "workflow_status_id or category required").into_response();
    };

    match issues_repo::status_belongs_to_project(pool, project_id, workflow_status_id).await {
        Ok(true) => {}
        Ok(false) => return (StatusCode::BAD_REQUEST, "status not in project").into_response(),
        Err(e) => {
            eprintln!("update_issue_status belongs check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    if let Err(e) = issues_repo::set_issue_workflow_status(pool, issue_id, workflow_status_id).await {
        eprintln!("update_issue_status update error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let slug = match issues_repo::get_workflow_status(pool, workflow_status_id).await {
        Ok(Some(w)) => w.slug,
        Ok(None) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Err(e) => {
            eprintln!("update_issue_status reload slug error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    (
        StatusCode::OK,
        Json(UpdateIssueStatusResponse {
            issue_id,
            workflow_status_id,
            status: slug,
        }),
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

fn summary_to_list_item(project_key: &str, r: issues_repo::IssueSummaryRow) -> IssueListItem {
    IssueListItem {
        issue_id: r.id,
        display_key: format!("{}-{}", project_key, r.key_seq),
        title: r.title,
        status: r.status_slug.clone(),
        workflow_status_id: r.workflow_status_id,
        status_name: r.status_name,
        status_category: r.status_category,
        is_blocked: r.is_blocked,
    }
}

async fn build_board_response(
    pool: &PgPool,
    project_id: uuid::Uuid,
    project_key: &str,
    rows: Vec<issues_repo::IssueSummaryRow>,
) -> Result<BoardResponse, sqlx::Error> {
    let defs = issues_repo::list_workflow_statuses_for_project_ordered(pool, project_id).await?;
    let column_definitions: Vec<BoardColumnDef> = defs
        .iter()
        .map(|w| BoardColumnDef {
            id: w.id,
            name: w.name.clone(),
            slug: w.slug.clone(),
            category: w.category.clone(),
            position: w.position,
        })
        .collect();
    let mut items_by_column_id: BTreeMap<String, Vec<IssueListItem>> = BTreeMap::new();
    for d in &defs {
        items_by_column_id.insert(d.id.to_string(), Vec::new());
    }
    for r in rows {
        let item = summary_to_list_item(project_key, r);
        let k = item.workflow_status_id.to_string();
        items_by_column_id.entry(k).or_default().push(item);
    }
    Ok(BoardResponse {
        column_definitions,
        items_by_column_id,
    })
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

    let board = match build_board_response(pool, project_id, &project_key, rows).await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("get_board build error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    (StatusCode::OK, Json(board)).into_response()
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

    let board = match build_board_response(pool, project.id, &project.project_key, rows).await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("get_board_by_key build error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    (StatusCode::OK, Json(board)).into_response()
}

pub async fn update_issue_board_position(
    pool: &PgPool,
    issue_id: uuid::Uuid,
    user_id: uuid::Uuid,
    req: UpdateIssueBoardPositionRequest,
) -> impl IntoResponse + use<> {
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

    match issues_repo::status_belongs_to_project(pool, project_id, req.workflow_status_id).await {
        Ok(true) => {}
        Ok(false) => return (StatusCode::BAD_REQUEST, "status not in project").into_response(),
        Err(e) => {
            eprintln!("update_issue_board_position belongs error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    if let Err(e) = issues_repo::update_issue_status_and_board_order(
        pool,
        issue_id,
        req.workflow_status_id,
        req.position,
    )
    .await
    {
        eprintln!("update_issue_board_position update error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let slug = match issues_repo::get_workflow_status(pool, req.workflow_status_id).await {
        Ok(Some(w)) => w.slug,
        _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    (
        StatusCode::OK,
        Json(UpdateIssueBoardPositionResponse {
            issue_id,
            workflow_status_id: req.workflow_status_id,
            status: slug,
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
        status: row.status_slug.clone(),
        workflow_status_id: row.workflow_status_id,
        status_name: row.status_name,
        status_category: row.status_category,
        is_blocked: row.is_blocked,
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

// --- Workflow status settings ----------------------------------------------

const WORKFLOW_CATEGORIES: &[&str] = &[
    "backlog",
    "unstarted",
    "started",
    "completed",
    "canceled",
];

fn slugify_workflow_name(name: &str) -> String {
    let s: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    let t = s.trim_matches('_').replace("__", "_");
    if t.is_empty() {
        "status".to_string()
    } else {
        t
    }
}

pub async fn list_workflow_statuses(
    pool: &PgPool,
    org_slug: String,
    project_key: String,
    user_id: uuid::Uuid,
) -> impl IntoResponse + use<> {
    let project = match issues_repo::find_project_by_org_slug_and_key(pool, &org_slug, &project_key).await {
        Ok(Some(p)) => p,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("list_workflow_statuses project error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let is_member = match events_repo::is_project_member(pool, project.id, user_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("list_workflow_statuses member error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    let rows = match issues_repo::list_workflow_statuses_for_project_ordered(pool, project.id).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("list_workflow_statuses query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut groups: Vec<WorkflowStatusesByCategory> = WORKFLOW_CATEGORIES
        .iter()
        .map(|c| WorkflowStatusesByCategory {
            category: (*c).to_string(),
            statuses: Vec::new(),
        })
        .collect();

    for w in rows {
        let item = WorkflowStatusItem {
            id: w.id,
            category: w.category.clone(),
            name: w.name,
            slug: w.slug,
            position: w.position,
            is_default: w.is_default,
        };
        if let Some(g) = groups.iter_mut().find(|g| g.category == item.category) {
            g.statuses.push(item);
        }
    }

    (
        StatusCode::OK,
        Json(ListWorkflowStatusesResponse { groups }),
    )
        .into_response()
}

pub async fn create_workflow_status(
    pool: &PgPool,
    org_slug: String,
    project_key: String,
    user_id: uuid::Uuid,
    req: CreateWorkflowStatusRequest,
) -> impl IntoResponse + use<> {
    if !WORKFLOW_CATEGORIES.contains(&req.category.as_str()) {
        return (StatusCode::BAD_REQUEST, "invalid category").into_response();
    }
    if req.name.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "name is required").into_response();
    }

    let project = match issues_repo::find_project_by_org_slug_and_key(pool, &org_slug, &project_key).await {
        Ok(Some(p)) => p,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("create_workflow_status project error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let can = match issues_repo::user_can_manage_workflow_statuses(pool, project.id, user_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("create_workflow_status perm error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !can {
        return StatusCode::FORBIDDEN.into_response();
    }

    let base_slug = req
        .slug
        .as_deref()
        .map(slugify_workflow_name)
        .unwrap_or_else(|| slugify_workflow_name(&req.name));

    let mut slug = base_slug.clone();
    let mut n = 0u32;
    loop {
        let taken = match issues_repo::slug_in_use(pool, project.id, &slug, None).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("create_workflow_status slug check error: {e:?}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        if !taken {
            break;
        }
        n += 1;
        slug = format!("{base_slug}_{n}");
    }

    let max_pos = match issues_repo::max_position_for_project_and_category(
        pool,
        project.id,
        req.category.as_str(),
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("create_workflow_status max pos error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let position = max_pos.map(|m| m + 1).unwrap_or(0);

    let name_trim = req.name.trim().to_string();
    let row = match issues_repo::insert_workflow_status(
        pool,
        project.id,
        &req.category,
        &name_trim,
        &slug,
        position,
        false,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("create_workflow_status insert error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    (
        StatusCode::CREATED,
        Json(WorkflowStatusItem {
            id: row.id,
            category: row.category,
            name: row.name,
            slug: row.slug,
            position: row.position,
            is_default: row.is_default,
        }),
    )
        .into_response()
}

pub async fn patch_workflow_status(
    pool: &PgPool,
    org_slug: String,
    project_key: String,
    status_id: uuid::Uuid,
    user_id: uuid::Uuid,
    req: PatchWorkflowStatusRequest,
) -> impl IntoResponse + use<> {
    let project = match issues_repo::find_project_by_org_slug_and_key(pool, &org_slug, &project_key).await {
        Ok(Some(p)) => p,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("patch_workflow_status project error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let can = match issues_repo::user_can_manage_workflow_statuses(pool, project.id, user_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("patch_workflow_status perm error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !can {
        return StatusCode::FORBIDDEN.into_response();
    }

    let current = match issues_repo::get_workflow_status(pool, status_id).await {
        Ok(Some(w)) => w,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("patch_workflow_status load error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if current.project_id != project.id {
        return StatusCode::NOT_FOUND.into_response();
    }

    let new_slug = req.slug.as_deref().map(slugify_workflow_name);
    if let Some(ref s) = new_slug {
        match issues_repo::slug_in_use(pool, project.id, s, Some(status_id)).await {
            Ok(true) => return (StatusCode::BAD_REQUEST, "slug already in use").into_response(),
            Ok(false) => {}
            Err(e) => {
                eprintln!("patch_workflow_status slug check error: {e:?}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }

    let updated = match issues_repo::update_workflow_status_fields(
        pool,
        status_id,
        req.name.as_deref().map(str::trim),
        new_slug.as_deref(),
        req.position,
    )
    .await
    {
        Ok(u) => u,
        Err(e) => {
            eprintln!("patch_workflow_status update error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let Some(mut row) = updated else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if req.is_default == Some(true) {
        let mut tx = match pool.begin().await {
            Ok(tx) => tx,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        if let Err(e) =
            issues_repo::set_workflow_status_default_tx(&mut tx, status_id, project.id).await
        {
            eprintln!("patch_workflow_status default tx error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        if let Err(e) = tx.commit().await {
            eprintln!("patch_workflow_status commit error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        row = match issues_repo::get_workflow_status(pool, status_id).await {
            Ok(Some(w)) => w,
            _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
    }

    (
        StatusCode::OK,
        Json(WorkflowStatusItem {
            id: row.id,
            category: row.category,
            name: row.name,
            slug: row.slug,
            position: row.position,
            is_default: row.is_default,
        }),
    )
        .into_response()
}

pub async fn delete_workflow_status(
    pool: &PgPool,
    org_slug: String,
    project_key: String,
    status_id: uuid::Uuid,
    user_id: uuid::Uuid,
    q: DeleteWorkflowStatusQuery,
) -> impl IntoResponse + use<> {
    let project = match issues_repo::find_project_by_org_slug_and_key(pool, &org_slug, &project_key).await {
        Ok(Some(p)) => p,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("delete_workflow_status project error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let can = match issues_repo::user_can_manage_workflow_statuses(pool, project.id, user_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("delete_workflow_status perm error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !can {
        return StatusCode::FORBIDDEN.into_response();
    }

    let current = match issues_repo::get_workflow_status(pool, status_id).await {
        Ok(Some(w)) => w,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("delete_workflow_status load error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if current.project_id != project.id {
        return StatusCode::NOT_FOUND.into_response();
    }

    if q.reassign_to == status_id {
        return (StatusCode::BAD_REQUEST, "reassign_to must differ from deleted status").into_response();
    }

    match issues_repo::status_belongs_to_project(pool, project.id, q.reassign_to).await {
        Ok(true) => {}
        Ok(false) => return (StatusCode::BAD_REQUEST, "reassign_to not in project").into_response(),
        Err(e) => {
            eprintln!("delete_workflow_status reassign check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if let Err(e) = issues_repo::reassign_issues_status(&mut *tx, status_id, q.reassign_to).await {
        eprintln!("delete_workflow_status reassign error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let cnt = match issues_repo::count_issues_on_status(&mut *tx, status_id).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("delete_workflow_status count error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if cnt > 0 {
        return (StatusCode::INTERNAL_SERVER_ERROR, "issues still on status").into_response();
    }

    match issues_repo::delete_workflow_status(&mut *tx, status_id).await {
        Ok(0) => return StatusCode::NOT_FOUND.into_response(),
        Ok(_) => {}
        Err(e) => {
            eprintln!("delete_workflow_status delete error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    if let Err(e) = tx.commit().await {
        eprintln!("delete_workflow_status commit error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    StatusCode::NO_CONTENT.into_response()
}
