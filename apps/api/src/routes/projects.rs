use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::{auth_user::AuthUser, state::AppState};

#[derive(Deserialize)]
pub struct ListMyProjectsQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub q: Option<String>,
}

#[derive(Serialize)]
pub struct ProjectItem {
    pub id: uuid::Uuid,
    pub project_key: String,
    pub name: String,
    pub description: Option<String>,
    pub org_id: uuid::Uuid,
    pub org_name: String,
    pub org_slug: String,
    pub issue_count: i64,
    pub member_count: i64,
    pub my_role: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct ListMyProjectsResponse {
    pub items: Vec<ProjectItem>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
    pub has_more: bool,
}

pub async fn list_projects(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(query): Query<ListMyProjectsQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).min(100).max(1);
    let offset = query.offset.unwrap_or(0).max(0);
    let search = query.q.as_deref().unwrap_or("").trim();

    let search_pattern = if search.is_empty() {
        "%".to_string()
    } else {
        format!("%{}%", search.to_lowercase())
    };

    let total = match sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(DISTINCT p.id)
        FROM projects p
        JOIN project_members pm ON pm.project_id = p.id
        WHERE pm.user_id = $1
          AND (LOWER(p.name) LIKE $2 OR LOWER(p.project_key) LIKE $2)
        "#,
    )
    .bind(user_id)
    .bind(&search_pattern)
    .fetch_one(&state.db)
    .await
    {
        Ok(t) => t,
        Err(e) => {
            eprintln!("list_my_projects count error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let rows = match sqlx::query!(
        r#"
        SELECT 
            p.id,
            p.project_key,
            p.name,
            p.description,
            p.org_id,
            p.created_at,
            o.name AS org_name,
            o.slug AS org_slug,
            pm.project_role AS my_role,
            (SELECT COUNT(*) FROM issues WHERE project_id = p.id) AS "issue_count!",
            (SELECT COUNT(*) FROM project_members WHERE project_id = p.id) AS "member_count!"
        FROM projects p
        JOIN project_members pm ON pm.project_id = p.id AND pm.user_id = $1
        JOIN organizations o ON o.id = p.org_id
        WHERE (LOWER(p.name) LIKE $2 OR LOWER(p.project_key) LIKE $2)
        ORDER BY p.name ASC
        LIMIT $3 OFFSET $4
        "#,
        user_id,
        search_pattern,
        limit as i64,
        offset as i64
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("list_my_projects query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let items: Vec<ProjectItem> = rows
        .into_iter()
        .map(|r| ProjectItem {
            id: r.id,
            project_key: r.project_key,
            name: r.name,
            description: r.description,
            org_id: r.org_id,
            org_name: r.org_name,
            org_slug: r.org_slug,
            issue_count: r.issue_count,
            member_count: r.member_count,
            my_role: r.my_role,
            created_at: r.created_at,
        })
        .collect();

    let has_more = (offset as i64 + items.len() as i64) < total;

    (
        StatusCode::OK,
        Json(ListMyProjectsResponse {
            items,
            total,
            limit,
            offset,
            has_more,
        }),
    )
        .into_response()
}

#[derive(Serialize)]
pub struct GetProjectResponse {
    pub id: uuid::Uuid,
    pub project_key: String,
    pub name: String,
    pub description: Option<String>,
    pub org_id: uuid::Uuid,
    pub org_name: String,
    pub org_slug: String,
    pub issue_count: i64,
    pub member_count: i64,
    pub my_role: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn get_project(
    Path((org_slug, project_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl IntoResponse {
    let org_id =
        match sqlx::query_scalar::<_, uuid::Uuid>("SELECT id FROM organizations WHERE slug = $1")
            .bind(&org_slug)
            .fetch_optional(&state.db)
            .await
        {
            Ok(Some(id)) => id,
            Ok(None) => return StatusCode::NOT_FOUND.into_response(),
            Err(e) => {
                eprintln!("get_project org resolve error: {e:?}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

    let row = match sqlx::query!(
        r#"
        SELECT 
            p.id,
            p.project_key,
            p.name,
            p.description,
            p.org_id,
            p.created_at,
            o.name AS org_name,
            o.slug AS org_slug,
            pm.project_role AS my_role,
            (SELECT COUNT(*) FROM issues WHERE project_id = p.id) AS "issue_count!",
            (SELECT COUNT(*) FROM project_members WHERE project_id = p.id) AS "member_count!"
        FROM projects p
        JOIN project_members pm ON pm.project_id = p.id AND pm.user_id = $1
        JOIN organizations o ON o.id = p.org_id
        WHERE p.org_id = $2 AND UPPER(p.project_key) = UPPER($3)
        "#,
        user_id,
        org_id,
        project_key
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("get_project query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    (
        StatusCode::OK,
        Json(GetProjectResponse {
            id: row.id,
            project_key: row.project_key,
            name: row.name,
            description: row.description,
            org_id: row.org_id,
            org_name: row.org_name,
            org_slug: row.org_slug,
            issue_count: row.issue_count,
            member_count: row.member_count,
            my_role: row.my_role,
            created_at: row.created_at,
        }),
    )
        .into_response()
}

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub project_key: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct CreateProjectResponse {
    pub project_id: uuid::Uuid,
    pub project_key: String,
    pub name: String,
}

pub async fn create_project(
    Path(org_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateProjectRequest>,
) -> impl IntoResponse {
    // project key normalize
    let project_key = req.project_key.to_uppercase();

    if project_key.len() < 2 || project_key.len() > 6 {
        return (StatusCode::BAD_REQUEST, "invalid project_key").into_response();
    }

    // org admin mı?
    let is_admin = match sqlx::query_scalar::<_, i32>(
        r#"
        SELECT 1
        FROM org_members
        WHERE org_id = $1 AND user_id = $2 AND org_role = 'org_admin'
        "#,
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("create_project admin check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_admin {
        return StatusCode::FORBIDDEN.into_response();
    }

    // transaction
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // project insert
    let project = match sqlx::query!(
        r#"
        INSERT INTO projects (org_id, project_key, name, description)
        VALUES ($1,$2,$3,$4)
        RETURNING id, project_key, name
        "#,
        org_id,
        project_key,
        req.name,
        req.description
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("create_project insert error: {e:?}");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    // creator otomatik project_manager
    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO project_members (project_id, user_id, project_role)
        VALUES ($1,$2,'project_manager')
        "#,
        project.id,
        user_id
    )
    .execute(&mut *tx)
    .await
    {
        eprintln!("create_project member insert error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    if let Err(e) = tx.commit().await {
        eprintln!("create_project commit error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (
        StatusCode::CREATED,
        Json(CreateProjectResponse {
            project_id: project.id,
            project_key: project.project_key,
            name: project.name,
        }),
    )
        .into_response()
}

#[derive(serde::Serialize)]
pub struct ProjectMemberItem {
    pub user_id: uuid::Uuid,
    pub name: String,
    pub email: String,
    pub project_role: String,
}

#[derive(serde::Serialize)]
pub struct ListProjectMembersResponse {
    pub items: Vec<ProjectMemberItem>,
}

pub async fn list_project_members(
    Path((org_slug, project_key)): Path<(String, String)>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl IntoResponse {
    // org resolve
    let org_id =
        match sqlx::query_scalar::<_, uuid::Uuid>("SELECT id FROM organizations WHERE slug = $1")
            .bind(&org_slug)
            .fetch_optional(&state.db)
            .await
        {
            Ok(Some(id)) => id,
            Ok(None) => return StatusCode::NOT_FOUND.into_response(),
            Err(e) => {
                eprintln!("list_project_members org resolve error: {e:?}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

    // project resolve
    let project_id = match sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT id FROM projects WHERE org_id = $1 AND project_key = $2",
    )
    .bind(org_id)
    .bind(project_key.to_uppercase())
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("list_project_members project resolve error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // authz: actor project member mı? (en azından member)
    let is_member = match sqlx::query_scalar::<_, i32>(
        "SELECT 1 FROM project_members WHERE project_id = $1 AND user_id = $2",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            eprintln!("list_project_members membership check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_member {
        return StatusCode::FORBIDDEN.into_response();
    }

    // list
    let rows = match sqlx::query!(
        r#"
        SELECT pm.user_id, pm.project_role, u.email, u.name
        FROM project_members pm
        JOIN users u ON u.id = pm.user_id
        WHERE pm.project_id = $1
        ORDER BY u.name ASC
        "#,
        project_id
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("list_project_members query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let items = rows
        .into_iter()
        .map(|r| ProjectMemberItem {
            user_id: r.user_id,
            name: r.name,
            email: r.email,
            project_role: r.project_role,
        })
        .collect();

    (StatusCode::OK, Json(ListProjectMembersResponse { items })).into_response()
}
