use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde_json::{Value, json};

pub async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

pub async fn ready(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(json!({"status": "ready"})))
}
