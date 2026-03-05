use axum::{
    extract::State,
    http::{StatusCode}, 
    response::IntoResponse, 
    Json, 
};
use serde::{Deserialize, Serialize};

use crate::{state::AppState, auth_user::AuthUser};

#[derive(Deserialize)]
pub struct CreateOrgRequest {
    pub name: String,
    pub slug: String,
}

#[derive(Serialize)]
pub struct CreateOrgResponse {
    pub org_id: uuid::Uuid,
    pub name: String,
    pub slug: String,
}

pub async fn create_org(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateOrgRequest>,
) -> impl IntoResponse {
   
    // POLICY: şu an güvenli başlangıç -> sadece platform_admin
    // Self-serve'e geçince burayı gevşeteceğiz (tek yerden).
    let platform_role = match sqlx::query_scalar::<_, String>(
        "SELECT platform_role FROM users WHERE id = $1 AND is_active = true",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(e) => {
            eprintln!("create_org role check error {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if platform_role != "platform_admin" {
        return StatusCode::FORBIDDEN.into_response();
    }

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let org = match sqlx::query!(
        r#"
        INSERT INTO organizations (name, slug, created_by)
        VALUES ($1, $2, $3)
        RETURNING id, name, slug
        "#,
        req.name,
        req.slug,
        user_id
    )
    .fetch_one(&mut *tx)
    .await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("create_org insert error {e:?}");
            // slug unique vs. için şimdilik 400 diyelim (sonra daha iyi mapleriz)
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO org_members (org_id, user_id, org_role)
        VALUES ($1, $2, 'org_admin')
        "#,
        org.id,
        user_id
    )
    .execute(&mut *tx)
    .await
    {
        eprintln!("create_org insert org_members error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    if let Err(e) = tx.commit().await {
        eprintln!("create_org commit error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (
        StatusCode::CREATED,
        Json(CreateOrgResponse {
            org_id: org.id,
            name: org.name,
            slug: org.slug,
        }),
    )
        .into_response()
}
