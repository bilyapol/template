use crate::db;
use crate::error::AppError;
use crate::middleware::jwt;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, Nonce, RedirectUrl,
    Scope, TokenResponse, core::CoreClient,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Deserialize)]
pub struct GoogleUserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
}

// ---------------------------------------------------------------------------
// Auth mode endpoint — tells the frontend which login method is available
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct AuthMode {
    pub mode: &'static str, // "google", "dev" or "none"
}

pub async fn auth_mode(State(state): State<AppState>) -> Json<AuthMode> {
    let mode = if state.config.google_oauth_enabled() {
        "google"
    } else if state.config.dev_login_enabled() {
        "dev"
    } else {
        "none"
    };
    Json(AuthMode { mode })
}

// ---------------------------------------------------------------------------
// Dev-mode login — zero external dependencies, just pick an email
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct DevLoginRequest {
    pub email: String,
    pub name: Option<String>,
}

pub async fn dev_login(
    State(state): State<AppState>,
    Json(body): Json<DevLoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    if !state.config.dev_login_enabled() {
        return Err(AppError::Forbidden("Dev login is disabled".into()));
    }

    let email = body.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return Err(AppError::Validation("Invalid email".into()));
    }

    let name = body
        .name
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| email.split('@').next().unwrap_or("User").to_string());

    let user = db::users::upsert_by_email(&state.db, &email, &name).await?;
    let token = jwt::sign(&user.id, &state.config)?;

    Ok((
        [(
            header::SET_COOKIE,
            format!(
                "auth_token={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age={}{}",
                state.config.jwt_expiry_seconds,
                state.config.cookie_secure_flag()
            ),
        )],
        Json(serde_json::json!({"ok": true})),
    ))
}

// ---------------------------------------------------------------------------
// Google OAuth login
// ---------------------------------------------------------------------------

pub async fn google_login(State(state): State<AppState>) -> Result<Response, AppError> {
    let meta = state
        .oidc_metadata
        .as_ref()
        .ok_or_else(|| AppError::Validation("Google OAuth not configured".into()))?;

    let client = CoreClient::from_provider_metadata(
        meta.clone(),
        ClientId::new(state.config.google_client_id.clone().unwrap()),
        Some(ClientSecret::new(
            state.config.google_client_secret.clone().unwrap(),
        )),
    )
    .set_redirect_uri(
        RedirectUrl::new(state.config.google_redirect_uri.clone().unwrap())
            .expect("Invalid redirect URI"),
    )
    .set_auth_uri(meta.authorization_endpoint().clone())
    .set_token_uri(
        meta.token_endpoint()
            .expect("Google OIDC missing token endpoint")
            .clone(),
    );

    let secure_flag = state.config.cookie_secure_flag();
    let (auth_url, csrf_token, nonce) = client
        .authorize_url(
            AuthenticationFlow::<openidconnect::core::CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    Ok(Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header(header::LOCATION, auth_url.as_str())
        .header(
            header::SET_COOKIE,
            format!(
                "oauth_state={}; HttpOnly; SameSite=Lax; Path=/; Max-Age=600{secure_flag}",
                csrf_token.secret()
            ),
        )
        .header(
            header::SET_COOKIE,
            format!(
                "oauth_nonce={}; HttpOnly; SameSite=Lax; Path=/; Max-Age=600{secure_flag}",
                nonce.secret()
            ),
        )
        .body(axum::body::Body::empty())
        .unwrap())
}

pub async fn google_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<CallbackParams>,
) -> Result<impl IntoResponse, AppError> {
    let meta = state
        .oidc_metadata
        .as_ref()
        .ok_or_else(|| AppError::Validation("Google OAuth not configured".into()))?;

    // 1. Verify CSRF state
    let expected_state = extract_cookie_from_headers(&headers, "oauth_state")
        .ok_or_else(|| AppError::Validation("Missing OAuth state cookie".into()))?;
    let received_state = params
        .state
        .as_deref()
        .ok_or_else(|| AppError::Validation("Missing OAuth state parameter".into()))?;
    if expected_state != received_state {
        return Err(AppError::Validation(
            "OAuth state mismatch — possible CSRF".into(),
        ));
    }

    // 2. Get nonce from cookie for ID token verification
    let nonce_str = extract_cookie_from_headers(&headers, "oauth_nonce")
        .ok_or_else(|| AppError::Validation("Missing OAuth nonce cookie".into()))?;
    let nonce = Nonce::new(nonce_str);

    // 3. Build OIDC client
    let client = CoreClient::from_provider_metadata(
        meta.clone(),
        ClientId::new(state.config.google_client_id.clone().unwrap()),
        Some(ClientSecret::new(
            state.config.google_client_secret.clone().unwrap(),
        )),
    )
    .set_redirect_uri(
        RedirectUrl::new(state.config.google_redirect_uri.clone().unwrap())
            .expect("Invalid redirect URI"),
    )
    .set_auth_uri(meta.authorization_endpoint().clone())
    .set_token_uri(
        meta.token_endpoint()
            .expect("Google OIDC missing token endpoint")
            .clone(),
    );

    // 4. Exchange authorization code for tokens
    let token_response = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(&state.http)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Token exchange failed: {e}")))?;

    // 5. Extract and verify ID token
    let id_token = token_response
        .id_token()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Missing ID token in response")))?;

    let claims = id_token
        .claims(&client.id_token_verifier(), &nonce)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("ID token verification failed: {e}")))?;

    // 6. Extract user info from verified claims
    let google_id = claims.subject().to_string();
    let email = claims
        .email()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Email not in ID token")))?
        .to_string();
    let name = claims
        .name()
        .and_then(|n| n.get(None))
        .map(|n| n.to_string())
        .unwrap_or_else(|| email.clone());
    let picture = claims
        .picture()
        .and_then(|p| p.get(None))
        .map(|p| p.to_string());

    let google_user = GoogleUserInfo {
        id: google_id,
        email,
        name,
        picture,
    };

    // 7. Upsert user
    let user = db::users::upsert_google(&state.db, &google_user).await?;

    // 8. Issue JWT
    let token = jwt::sign(&user.id, &state.config)?;

    // 9. Set cookies and redirect to frontend
    let secure_flag = state.config.cookie_secure_flag();

    Ok(Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header(header::LOCATION, &state.config.frontend_url)
        .header(
            header::SET_COOKIE,
            format!(
                "auth_token={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age={}{secure_flag}",
                state.config.jwt_expiry_seconds
            ),
        )
        .header(
            header::SET_COOKIE,
            format!("oauth_state=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0{secure_flag}"),
        )
        .header(
            header::SET_COOKIE,
            format!("oauth_nonce=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0{secure_flag}"),
        )
        .body(axum::body::Body::empty())
        .unwrap())
}

pub async fn logout(State(state): State<AppState>) -> impl IntoResponse {
    let cookie = format!(
        "auth_token=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0{}",
        state.config.cookie_secure_flag()
    );
    ([(header::SET_COOKIE, cookie)], StatusCode::OK)
}

fn extract_cookie_from_headers(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    let prefix = format!("{name}=");
    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if let Some(value) = cookie.strip_prefix(&prefix) {
            return Some(value.to_string());
        }
    }
    None
}
