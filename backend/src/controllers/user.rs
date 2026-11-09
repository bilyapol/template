use crate::db;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::Json;
use axum::extract::State;

pub async fn me(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<db::users::User>, AppError> {
    let user = db::users::get_by_id(&state.db, auth.user_id).await?;
    Ok(Json(user))
}
