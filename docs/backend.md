# Axum Backend

The backend is a Rust API server built with [Axum](https://github.com/tokio-rs/axum) 0.8, the async web framework from the Tokio project.

## Project Structure

```
backend/
├── src/
│   ├── main.rs              # Server startup, routing, middleware stack
│   ├── lib.rs               # Module declarations
│   ├── config.rs            # AppConfig loaded from environment variables
│   ├── state.rs             # AppState shared across all handlers
│   ├── error.rs             # AppError enum with HTTP status mapping
│   ├── controllers/         # HTTP route handlers
│   │   ├── auth.rs          # Google OAuth + dev-mode login
│   │   ├── docs.rs          # In-app documentation pages
│   │   ├── health.rs        # Health and readiness checks
│   │   └── user.rs          # Current user endpoint
│   ├── db/                  # Database access layer
│   │   └── users.rs         # User queries (upsert, get)
│   └── middleware/          # Request extractors
│       ├── auth.rs          # AuthUser / AdminUser extractors
│       ├── jwt.rs           # JWT sign and verify
│       └── rate_limit.rs    # Client-IP key for the rate limiter
└── migrations/              # SQL migrations (auto-run on startup)
    └── 001_users.sql
```

## How It Works

### Configuration

All config comes from environment variables, loaded via `envy` into a typed struct:

```rust
// config.rs
#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub app_env: String,            // "development" (default) or "production"
    pub database_url: String,       // default: local docker postgres
    pub jwt_secret: String,         // default: dev secret
    pub google_client_id: Option<String>,  // optional
    // ...
}
```

Every field has a sensible default. The app boots with zero configuration in development.

With `APP_ENV=production`, `AppConfig::validate()` refuses to start unless `JWT_SECRET` is at least 32 characters (not the dev default) and `FRONTEND_URL` uses `https://`. Production also disables dev login and marks all cookies `Secure`.

### Application State

State is shared across all handlers via Axum's `State` extractor:

```rust
// state.rs
pub struct AppState {
    pub db: PgPool,                              // database connection pool
    pub config: AppConfig,                       // configuration
    pub http: reqwest::Client,                   // HTTP client for external APIs
    pub oidc_metadata: Option<CoreProviderMetadata>,  // Google OIDC (when configured)
}
```

### Error Handling

All errors flow through a single `AppError` enum that maps to HTTP status codes:

```rust
// error.rs
pub enum AppError {
    Unauthorized,           // 401
    Forbidden(String),      // 403
    NotFound(String),       // 404
    Validation(String),     // 422
    Database(sqlx::Error),  // 500
    Internal(anyhow::Error), // 500
}
```

Every handler returns `Result<..., AppError>`, and the `IntoResponse` impl converts errors to JSON responses automatically. For 500 errors the details are logged server-side and the client only sees `"Internal server error"`.

### Authentication

The template uses Axum's extractor pattern for auth. Add `AuthUser` to any handler signature and it's protected:

```rust
pub async fn me(
    auth: AuthUser,        // <-- this line protects the route
    State(state): State<AppState>,
) -> Result<Json<db::users::User>, AppError> {
    let user = db::users::get_by_id(&state.db, auth.user_id).await?;
    Ok(Json(user))
}
```

The `AuthUser` extractor reads the `auth_token` cookie, verifies the JWT signature, and extracts the user ID. If the token is missing or invalid, it returns 401 automatically.

For admin-only routes, use `AdminUser` instead — it does the same check plus verifies `role = 'admin'` in the database.

### Database

Queries use [SQLx](https://github.com/launchbadge/sqlx) with raw SQL (no ORM), checked at runtime via `query_as`. Results are deserialized into structs via `FromRow`:

```rust
#[derive(sqlx::FromRow, Serialize)]
pub struct Widget {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
}

pub async fn list_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<Widget>, AppError> {
    let widgets = sqlx::query_as::<_, Widget>(
        "SELECT * FROM widgets WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(widgets)
}
```

Always scope queries by `user_id` so users can only touch their own rows.

Migrations run automatically on startup via `sqlx::migrate!("./migrations")`.

### Rate Limiting

The API is rate-limited per client IP using `tower-governor` (bursts of 60, refilling 1 per second). Health check endpoints are outside the rate limiter so deployment health probes always work.

By default the client IP is the TCP peer address, which can't be spoofed. Behind a reverse proxy (Railway, Fly, nginx) every request comes from the proxy, so set `TRUST_PROXY=true` to read `X-Forwarded-For` instead. Only do this when the proxy is the sole way to reach the server.

### Security Headers

Every response includes security headers:
- `Content-Security-Policy` (only same-origin scripts; no framing)
- `Strict-Transport-Security` (HSTS)
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `Referrer-Policy: strict-origin-when-cross-origin`
- `Permissions-Policy` (disables camera, microphone, geolocation)

### Graceful Shutdown

The server handles `SIGTERM` and `Ctrl+C`, allowing in-flight requests to complete before shutting down. This is critical for zero-downtime deployments on platforms like Railway.

## Adding a New Resource

1. Create a migration: `backend/migrations/002_widgets.sql`
2. Add a DB module: `backend/src/db/widgets.rs` with your queries
3. Register in `backend/src/db/mod.rs`: `pub mod widgets;`
4. Add a controller: `backend/src/controllers/widgets.rs` with handlers
5. Register in `backend/src/controllers/mod.rs`: `pub mod widgets;`
6. Add routes in `backend/src/main.rs`

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `axum` | Web framework |
| `tokio` | Async runtime |
| `sqlx` | PostgreSQL driver |
| `serde` / `serde_json` | Serialization |
| `jsonwebtoken` | JWT auth |
| `openidconnect` | Google OIDC |
| `tower-governor` | Rate limiting |
| `tower-http` | CORS, compression, static files |
| `tracing` | Structured logging |
| `thiserror` / `anyhow` | Error handling |
