use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default = "default_database_url")]
    pub database_url: String,
    #[serde(default = "default_frontend_url")]
    pub frontend_url: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_database_url() -> String {
    "postgres://app:app@localhost:5433/app".into()
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
}
