use backend::config;
use backend::controllers;
use backend::state::AppState;

use axum::{
    Router,
    http::{HeaderValue, Method, header::CONTENT_TYPE},
    middleware,
    routing::{any, get, post},
};
use backend::middleware::rate_limit::ClientIpKeyExtractor;
use openidconnect::{IssuerUrl, core::CoreProviderMetadata};
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = config::AppConfig::from_env().expect("Failed to load config");
    if let Err(e) = config.validate() {
        panic!("Invalid configuration: {e}");
    }
    let db = PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");

    let http_client = reqwest::Client::new();

    // Only discover Google OIDC if credentials are configured
    let oidc_metadata = if config.google_oauth_enabled() {
        tracing::info!("Google OAuth enabled");
        let issuer =
            IssuerUrl::new("https://accounts.google.com".to_string()).expect("Invalid issuer URL");
        let metadata = CoreProviderMetadata::discover_async(issuer, &http_client)
            .await
            .expect("Failed to discover Google OIDC configuration");
        Some(metadata)
    } else if config.dev_login_enabled() {
        tracing::warn!("Google OAuth not configured — dev-mode login enabled");
        None
    } else {
        tracing::warn!("Google OAuth not configured — login is disabled in production");
        None
    };

    let state = AppState {
        db,
        config: config.clone(),
        http: http_client,
        oidc_metadata,
    };

    let frontend_dist =
        ServeDir::new("../frontend/dist").fallback(ServeFile::new("../frontend/dist/index.html"));

    let cors_origin = config
        .frontend_url
        .parse::<HeaderValue>()
        .expect("Invalid FRONTEND_URL");

    // Rate limiting: 60 requests per minute per IP
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(60)
            .key_extractor(ClientIpKeyExtractor {
                trust_proxy: config.trust_proxy,
            })
            .finish()
            .expect("Failed to build rate limiter config"),
    );

    // Rate-limited API routes
    let mut api_routes = Router::new()
        // Auth (always available)
        .route("/api/auth/logout", post(controllers::auth::logout))
        .route("/api/auth/mode", get(controllers::auth::auth_mode))
        // User
        .route("/api/user/me", get(controllers::user::me))
        // Docs
        .route("/api/docs", get(controllers::docs::list_docs))
        .route("/api/docs/{slug}", get(controllers::docs::get_doc));

    if config.google_oauth_enabled() {
        // Google OAuth routes
        api_routes = api_routes
            .route("/api/auth/google", get(controllers::auth::google_login))
            .route(
                "/api/auth/google/callback",
                get(controllers::auth::google_callback),
            );
    } else if config.dev_login_enabled() {
        // Dev-mode login (email-only, no password, no external deps)
        api_routes = api_routes.route("/api/auth/dev-login", post(controllers::auth::dev_login));
    }

    let api_routes = api_routes.layer(GovernorLayer::new(governor_conf));

    let app = Router::new()
        // Health routes — outside rate limiter
        .route("/api/health", get(controllers::health::health))
        .route("/api/ready", get(controllers::health::ready))
        .merge(api_routes)
        // Unknown API paths get a JSON 404 instead of the SPA's index.html
        .route("/api/{*path}", any(api_not_found))
        .with_state(state)
        .layer(middleware::from_fn(security_headers))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin(cors_origin)
                .allow_credentials(true)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ])
                .allow_headers([CONTENT_TYPE]),
        )
        // Serve frontend static files (fallback)
        .fallback_service(frontend_dist);

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .expect("Server error");

    tracing::info!("Server shut down gracefully");
}

async fn api_not_found() -> backend::error::AppError {
    backend::error::AppError::NotFound("API route not found".into())
}

async fn security_headers(
    req: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> axum::response::Response {
    let mut res = next.run(req).await;
    let h = res.headers_mut();
    h.insert("x-content-type-options", "nosniff".parse().unwrap());
    h.insert("x-frame-options", "DENY".parse().unwrap());
    h.insert(
        "content-security-policy",
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
         img-src 'self' data: https:; connect-src 'self'; frame-ancestors 'none'; \
         base-uri 'self'; form-action 'self'; object-src 'none'"
            .parse()
            .unwrap(),
    );
    h.insert(
        "strict-transport-security",
        "max-age=63072000; includeSubDomains".parse().unwrap(),
    );
    h.insert(
        "referrer-policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    h.insert(
        "permissions-policy",
        "camera=(), microphone=(), geolocation=()".parse().unwrap(),
    );
    res
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => tracing::info!("Received Ctrl+C, starting graceful shutdown"),
        () = terminate => tracing::info!("Received SIGTERM, starting graceful shutdown"),
    }
}
