use serde::Deserialize;

const DEV_JWT_SECRET: &str = "dev-secret-change-me-in-prod";
const MIN_JWT_SECRET_LEN: usize = 32;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    /// "development" (default) or "production". Production enforces safe settings.
    #[serde(default = "default_app_env")]
    pub app_env: String,

    #[serde(default = "default_database_url")]
    pub database_url: String,
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
    #[serde(default = "default_jwt_expiry")]
    pub jwt_expiry_seconds: u64,

    // Google OAuth — optional. When absent, dev-mode login is enabled (development only).
    #[serde(default)]
    pub google_client_id: Option<String>,
    #[serde(default)]
    pub google_client_secret: Option<String>,
    #[serde(default)]
    pub google_redirect_uri: Option<String>,

    /// Trust X-Forwarded-For / X-Real-IP for rate limiting. Only enable behind a
    /// reverse proxy (Railway, Fly, nginx) — otherwise clients can spoof their IP.
    #[serde(default)]
    pub trust_proxy: bool,

    #[serde(default = "default_frontend_url")]
    pub frontend_url: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_app_env() -> String {
    "development".into()
}
fn default_database_url() -> String {
    "postgres://app:app@localhost:5433/app".into()
}
fn default_jwt_secret() -> String {
    DEV_JWT_SECRET.into()
}
fn default_jwt_expiry() -> u64 {
    86400
}
fn default_frontend_url() -> String {
    "http://localhost:5173".into()
}
fn default_port() -> u16 {
    3000
}

impl AppConfig {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env()
    }

    pub fn is_production(&self) -> bool {
        self.app_env.eq_ignore_ascii_case("production")
    }

    /// Returns true when Google OAuth credentials are fully configured.
    pub fn google_oauth_enabled(&self) -> bool {
        self.google_client_id.is_some()
            && self.google_client_secret.is_some()
            && self.google_redirect_uri.is_some()
    }

    /// Dev login (email only, no password) is never available in production.
    pub fn dev_login_enabled(&self) -> bool {
        !self.is_production() && !self.google_oauth_enabled()
    }

    /// Cookie attribute suffix: adds `Secure` when served over HTTPS.
    pub fn cookie_secure_flag(&self) -> &'static str {
        if self.is_production() || self.frontend_url.starts_with("https") {
            "; Secure"
        } else {
            ""
        }
    }

    /// Rejects unsafe settings in production. Call once at startup.
    pub fn validate(&self) -> Result<(), String> {
        if !self.is_production() {
            return Ok(());
        }
        if self.jwt_secret == DEV_JWT_SECRET {
            return Err("JWT_SECRET must be set in production".into());
        }
        if self.jwt_secret.len() < MIN_JWT_SECRET_LEN {
            return Err(format!(
                "JWT_SECRET must be at least {MIN_JWT_SECRET_LEN} characters in production"
            ));
        }
        if !self.frontend_url.starts_with("https://") {
            return Err("FRONTEND_URL must use https:// in production".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(app_env: &str) -> AppConfig {
        AppConfig {
            app_env: app_env.into(),
            database_url: default_database_url(),
            jwt_secret: default_jwt_secret(),
            jwt_expiry_seconds: default_jwt_expiry(),
            google_client_id: None,
            google_client_secret: None,
            google_redirect_uri: None,
            trust_proxy: false,
            frontend_url: default_frontend_url(),
            port: default_port(),
        }
    }

    #[test]
    fn development_defaults_are_valid() {
        let c = config("development");
        assert!(c.validate().is_ok());
        assert!(c.dev_login_enabled());
        assert_eq!(c.cookie_secure_flag(), "");
    }

    #[test]
    fn production_rejects_default_secret() {
        let mut c = config("production");
        c.frontend_url = "https://example.com".into();
        assert!(c.validate().is_err());
    }

    #[test]
    fn production_rejects_short_secret() {
        let mut c = config("production");
        c.frontend_url = "https://example.com".into();
        c.jwt_secret = "too-short".into();
        assert!(c.validate().is_err());
    }

    #[test]
    fn production_rejects_http_frontend() {
        let mut c = config("production");
        c.jwt_secret = "x".repeat(MIN_JWT_SECRET_LEN);
        assert!(c.validate().is_err());
    }

    #[test]
    fn production_with_strong_secret_is_valid() {
        let mut c = config("production");
        c.frontend_url = "https://example.com".into();
        c.jwt_secret = "x".repeat(MIN_JWT_SECRET_LEN);
        assert!(c.validate().is_ok());
        assert!(!c.dev_login_enabled());
        assert_eq!(c.cookie_secure_flag(), "; Secure");
    }
}
