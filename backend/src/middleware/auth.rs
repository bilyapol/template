use crate::db;
use crate::error::AppError;
use crate::middleware::jwt;
use crate::state::AppState;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use uuid::Uuid;

/// Extractor that requires a valid JWT cookie. Use in handler signatures:
/// `async fn my_handler(auth: AuthUser, ...) -> ...`
pub struct AuthUser {
    pub user_id: Uuid,
}

impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let token = extract_cookie(parts, "auth_token").ok_or(AppError::Unauthorized)?;
        let claims = jwt::verify(&token, &app_state.config)?;
        Ok(AuthUser {
            user_id: claims.sub,
        })
    }
}

/// Extractor that requires the authenticated user to have the 'admin' role.
pub struct AdminUser {
    pub user_id: Uuid,
}

impl<S> FromRequestParts<S> for AdminUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let token = extract_cookie(parts, "auth_token").ok_or(AppError::Unauthorized)?;
        let claims = jwt::verify(&token, &app_state.config)?;
        let user = db::users::get_by_id(&app_state.db, claims.sub).await?;

        if user.role != "admin" {
            return Err(AppError::Forbidden("Forbidden".into()));
        }

        Ok(AdminUser {
            user_id: claims.sub,
        })
    }
}

fn extract_cookie(parts: &Parts, name: &str) -> Option<String> {
    let cookie_header = parts.headers.get("cookie")?.to_str().ok()?;
    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if let Some(value) = cookie.strip_prefix(&format!("{name}=")) {
            return Some(value.to_string());
        }
    }
    None
}
