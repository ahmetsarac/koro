use axum::{Json, extract::State, http::StatusCode};

use crate::{
    auth_user::AuthUser, error::AppError, services::users as users_service, state::AppState,
};

pub async fn get_me(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<(StatusCode, Json<users_service::MeResponse>), AppError> {
    let me = users_service::get_me(&state.db, user_id).await?;
    Ok((StatusCode::OK, Json(me)))
}
