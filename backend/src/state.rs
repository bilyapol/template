use crate::config::AppConfig;
use openidconnect::core::CoreProviderMetadata;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: AppConfig,
    pub http: reqwest::Client,
    /// None when Google OAuth is not configured (dev mode).
    pub oidc_metadata: Option<CoreProviderMetadata>,
}
