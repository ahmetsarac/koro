use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Serialize;

use crate::{auth_user::AuthUser, state::AppState};

#[derive(Serialize)]
pub struct UserOrganization {
    pub id: uuid::Uuid,
    pub name: String,
    pub slug: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct MeResponse {
    pub id: uuid::Uuid,
    pub email: String,
    pub name: String,
    pub organizations: Vec<UserOrganization>,
}

pub async fn get_me(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> impl IntoResponse {
    let user = match sqlx::query!(
        r#"SELECT id, email, name FROM users WHERE id = $1"#,
        user_id
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(u)) => u,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("get_me user query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let orgs = match sqlx::query!(
        r#"
        SELECT o.id, o.name, o.slug, om.org_role
        FROM organizations o
        JOIN org_members om ON om.org_id = o.id
        WHERE om.user_id = $1
        ORDER BY o.name ASC
        "#,
        user_id
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows
            .into_iter()
            .map(|r| UserOrganization {
                id: r.id,
                name: r.name,
                slug: r.slug,
                role: r.org_role,
            })
            .collect(),
        Err(e) => {
            eprintln!("get_me orgs query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    (
        StatusCode::OK,
        Json(MeResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            organizations: orgs,
        }),
    )
        .into_response()
}
