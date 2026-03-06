use crate::{http::user_id_from_headers, invite};
use chrono::{Duration, Utc};

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateInviteRequest {
    pub email: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct CreateInviteResponse {
    pub invite_url: String,
}

pub async fn create_invite(
    Path(org_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateInviteRequest>,
) -> impl IntoResponse {
    let user_id = match user_id_from_headers(&headers) {
        Ok(id) => id,
        Err(sc) => return sc.into_response(),
    };

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
            eprintln!("create_invite admin check error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !is_admin {
        return StatusCode::FORBIDDEN.into_response();
    }

    let raw_token = invite::generate_token();
    let token_hash = invite::hash_token(&raw_token);

    let expires_at = Utc::now() + Duration::days(7);

    let insert = sqlx::query!(
        r#"
    INSERT INTO user_invites (org_id, email, invited_role, token_hash, expires_at, invited_by)
    VALUES ($1, $2, $3, $4, $5, $6)
    "#,
        org_id,
        req.email,
        req.role,
        token_hash,
        expires_at,
        user_id
    )
    .execute(&state.db)
    .await;

    if let Err(e) = insert {
        eprintln!("create_invite insert error: {e:?}");
        return StatusCode::BAD_REQUEST.into_response(); // role check veya constraint patlayabilir
    }

    let invite_url = format!("http://localhost:3001/invites/{}", raw_token);

    (
        StatusCode::CREATED,
        Json(CreateInviteResponse { invite_url }),
    )
        .into_response()
}

#[derive(Serialize)]
pub struct GetInviteResponse {
    pub org_name: String,
    pub email: String,
    pub role: String,
    pub expires_at: String,
}

pub async fn get_invite(
    Path(token): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let token_hash = invite::hash_token(&token);

    let row = sqlx::query!(
        r#"
        SELECT ui.email, ui.invited_role, ui.expires_at, ui.used_at, o.name as org_name
        FROM user_invites ui
        JOIN organizations o ON o.id = ui.org_id
        WHERE ui.token_hash = $1
        "#,
        token_hash
    )
    .fetch_optional(&state.db)
    .await;

    let row = match row {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("get_invite query error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // used kontrol
    if row.used_at.is_some() {
        return StatusCode::GONE.into_response(); // "used"
    }

    // expire kontrol
    if row.expires_at < Utc::now() {
        return StatusCode::GONE.into_response(); // "expired"
    }

    (
        StatusCode::OK,
        Json(GetInviteResponse {
            org_name: row.org_name,
            email: row.email,
            role: row.invited_role,
            expires_at: row.expires_at.to_rfc3339(),
        }),
    )
        .into_response()
}

#[derive(Deserialize)]
pub struct AcceptInviteRequest {
    pub name: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AcceptInviteResponse {
    pub user_id: uuid::Uuid,
    pub org_id: uuid::Uuid,
    pub org_role: String,
}

pub async fn accept_invite(
    Path(token): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<AcceptInviteRequest>,
) -> impl IntoResponse {
    if req.password.len() < 8 {
        return (StatusCode::BAD_REQUEST, "password must be at least 8 chars").into_response();
    }

    let token_hash = invite::hash_token(&token);

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // 1) invite'ı çek (FOR UPDATE: aynı anda iki accept olmasın)
    let inv = match sqlx::query!(
        r#"
        SELECT id, org_id, email, invited_role, expires_at, used_at
        FROM user_invites
        WHERE token_hash = $1
        FOR UPDATE
        "#,
        token_hash
    )
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("accept_invite fetch invite error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if inv.used_at.is_some() {
        return StatusCode::GONE.into_response();
    }
    if inv.expires_at < Utc::now() {
        return StatusCode::GONE.into_response();
    }

    // 2) user var mı?
    let existing_user = sqlx::query!(
        r#"SELECT id, password_hash FROM users WHERE email = $1 AND is_active = true"#,
        inv.email
    )
    .fetch_optional(&mut *tx)
    .await;

    let (user_id, had_password) = match existing_user {
        Ok(Some(u)) => (u.id, u.password_hash.is_some()),
        Ok(None) => {
            // create user (password_hash şimdilik sonra set edeceğiz)
            let u = match sqlx::query!(
                r#"
                INSERT INTO users (email, name, platform_role)
                VALUES ($1, $2, 'user')
                RETURNING id
                "#,
                inv.email,
                req.name
            )
            .fetch_one(&mut *tx)
            .await
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("accept_invite create user error: {e:?}");
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            };
            (u.id, false)
        }
        Err(e) => {
            eprintln!("accept_invite user lookup error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // 3) password set (eğer zaten password varsa istersen 409 yapabiliriz)
    if had_password {
        // aynı email zaten sistemde kayıtlıysa ve şifresi varsa, invite'ı kabul ettirip
        // sadece org membership eklemek de isteyebilirsin.
        // MVP için: yine de membership ekleyelim ama şifreyi değiştirmeyelim.
    } else {
        let hash = match crate::auth::hash_password(&req.password) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("accept_invite hash error: {e:?}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

        if let Err(e) = sqlx::query!(
            r#"UPDATE users SET password_hash = $1, name = $2 WHERE id = $3"#,
            hash,
            req.name,
            user_id
        )
        .execute(&mut *tx)
        .await
        {
            eprintln!("accept_invite set password error: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    // 4) org membership ekle (invite role'ü ile)
    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO org_members (org_id, user_id, org_role)
        VALUES ($1, $2, $3)
        ON CONFLICT (org_id, user_id) DO NOTHING
        "#,
        inv.org_id,
        user_id,
        inv.invited_role
    )
    .execute(&mut *tx)
    .await
    {
        eprintln!("accept_invite insert org_member error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // 5) invite used yap
    if let Err(e) = sqlx::query!(
        r#"UPDATE user_invites SET used_at = now() WHERE id = $1"#,
        inv.id
    )
    .execute(&mut *tx)
    .await
    {
        eprintln!("accept_invite mark used error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    if let Err(e) = tx.commit().await {
        eprintln!("accept_invite commit error: {e:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (
        StatusCode::OK,
        Json(AcceptInviteResponse {
            user_id,
            org_id: inv.org_id,
            org_role: inv.invited_role,
        }),
    )
        .into_response()
}
