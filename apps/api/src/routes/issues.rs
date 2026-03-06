use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
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

#[derive(serde::Serialize)]
pub struct IssueDetailResponse {
    pub issue_id: uuid::Uuid,
    pub display_key: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub assignee_id: Option<uuid::Uuid>,
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
            i.assignee_id,
            i.created_at,
            i.updated_at,
            p.project_key
        FROM issues i
        JOIN projects p ON p.id = i.project_id
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
            assignee_id: row.assignee_id,
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
            i.assignee_id,
            i.created_at,
            i.updated_at,
            p.project_key,
            o.id as org_id
        FROM issues i
        JOIN projects p ON p.id = i.project_id
        JOIN organizations o ON o.id = p.org_id
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
            assignee_id: row.assignee_id,
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
