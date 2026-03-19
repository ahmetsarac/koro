use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::{
    core::AppError,
    invites::repository as invites_repo,
    orgs::repository as orgs_repo,
    projects::models::*,
    projects::repository as projects_repo,
};

pub async fn list_projects(
    pool: &PgPool,
    user_id: Uuid,
    query: ListMyProjectsQuery,
) -> Result<ListMyProjectsResponse, AppError> {
    let limit = query.limit.unwrap_or(50).min(100).max(1);
    let offset = query.offset.unwrap_or(0).max(0);
    let search = query.q.as_deref().unwrap_or("").trim();

    let search_pattern = if search.is_empty() {
        "%".to_string()
    } else {
        format!("%{}%", search.to_lowercase())
    };

    let total = projects_repo::count_member_projects(pool, user_id, &search_pattern)
        .await
        .map_err(|e| {
            tracing::error!(?e, "list_my_projects count");
            AppError::Internal
        })?;

    let rows = projects_repo::list_member_projects(
        pool,
        user_id,
        &search_pattern,
        limit as i64,
        offset as i64,
    )
    .await
    .map_err(|e| {
        tracing::error!(?e, "list_my_projects query");
        AppError::Internal
    })?;

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

    Ok(ListMyProjectsResponse {
        items,
        total,
        limit,
        offset,
        has_more,
    })
}

pub async fn get_project(
    pool: &PgPool,
    user_id: Uuid,
    org_slug: &str,
    project_key: &str,
) -> Result<GetProjectResponse, AppError> {
    let org_id = orgs_repo::find_org_id_by_slug(pool, org_slug)
        .await
        .map_err(|e| {
            tracing::error!(?e, "get_project org resolve");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let row = projects_repo::get_project_for_member(pool, user_id, org_id, project_key)
        .await
        .map_err(|e| {
            tracing::error!(?e, "get_project query");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    Ok(GetProjectResponse {
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
    })
}

pub async fn create_project(
    pool: &PgPool,
    org_id: Uuid,
    user_id: Uuid,
    req: CreateProjectRequest,
) -> Result<CreateProjectResponse, AppError> {
    let project_key = req.project_key.to_uppercase();

    if project_key.len() < 2 || project_key.len() > 6 {
        return Err(AppError::BadRequest(Some("invalid project_key")));
    }

    let is_admin = invites_repo::is_org_admin(pool, org_id, user_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "create_project admin check");
            AppError::Internal
        })?;

    if !is_admin {
        return Err(AppError::Forbidden);
    }

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!(?e, "create_project begin");
        AppError::Internal
    })?;

    let (project_id, pk, name) =
        projects_repo::insert_project_tx(&mut tx, org_id, &project_key, &req.name, req.description.as_deref())
            .await
            .map_err(|e| {
                tracing::error!(?e, "create_project insert");
                AppError::BadRequest(None)
            })?;

    projects_repo::insert_project_manager_tx(&mut tx, project_id, user_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "create_project member insert");
            AppError::Internal
        })?;

    tx.commit().await.map_err(|e| {
        tracing::error!(?e, "create_project commit");
        AppError::Internal
    })?;

    Ok(CreateProjectResponse {
        project_id,
        project_key: pk,
        name,
    })
}

pub async fn list_project_members(
    pool: &PgPool,
    user_id: Uuid,
    org_slug: &str,
    project_key: &str,
) -> Result<ListProjectMembersResponse, AppError> {
    let org_id = orgs_repo::find_org_id_by_slug(pool, org_slug)
        .await
        .map_err(|e| {
            tracing::error!(?e, "list_project_members org resolve");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let project_key_upper = project_key.to_uppercase();
    let project_id = projects_repo::find_project_id_in_org(pool, org_id, &project_key_upper)
        .await
        .map_err(|e| {
            tracing::error!(?e, "list_project_members project resolve");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let is_member = crate::modules::events::repository::is_project_member(pool, project_id, user_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "list_project_members membership check");
            AppError::Internal
        })?;

    if !is_member {
        return Err(AppError::Forbidden);
    }

    let rows = projects_repo::list_project_members_for_project(pool, project_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "list_project_members query");
            AppError::Internal
        })?;

    let items = rows
        .into_iter()
        .map(|r| ProjectMemberItem {
            user_id: r.user_id,
            name: r.name,
            email: r.email,
            project_role: r.project_role,
        })
        .collect();

    Ok(ListProjectMembersResponse { items })
}
