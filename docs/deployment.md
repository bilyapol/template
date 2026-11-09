# Deployment

The template is designed for simple, production-ready deployment. One Docker image serves both the API and the frontend.

## Architecture in Production

```
┌─────────────────────────────────────┐
│          Docker Container           │
│                                     │
│   Axum Binary (single process)      │
│   ├── /api/*     → API handlers     │
│   └── /*         → frontend/dist/   │
│                                     │
│   Listens on 0.0.0.0:3000           │
└──────────────┬──────────────────────┘
               │
               ▼
        PostgreSQL (external)
```

No Nginx, no reverse proxy, no separate frontend server. The Axum binary serves the static frontend files via `tower-http::services::ServeDir` with SPA fallback.

## Dockerfile

The multi-stage Dockerfile produces a minimal image (~50MB):

```dockerfile
# Stage 1: Build Rust backend
FROM rust:1.91-bookworm AS rust-builder
COPY . .
RUN cargo build --release --package backend

# Stage 2: Build frontend
FROM node:20-bookworm AS node-builder
COPY frontend/ .
RUN npm ci && npm run build

# Stage 3: Runtime (only the binary + static files)
FROM debian:bookworm-slim
COPY --from=rust-builder /app/target/release/backend ./
COPY backend/migrations ./migrations
COPY --from=node-builder /app/frontend/dist /app/frontend/dist
```

Key details:
- `SQLX_OFFLINE=true` during build so SQLx doesn't need a live database
- Non-root user (`appuser`, uid 1001) for security
- `APP_ENV=production` is baked into the image, so production safety checks are always on
- Only `ca-certificates` and `libssl3` in the runtime image

## Deploy to Railway

The template includes a `railway.toml` for one-click deployment:

```toml
[build]
builder = "dockerfile"
dockerfilePath = "Dockerfile"

[deploy]
healthcheckPath = "/api/health"
healthcheckTimeout = 30
restartPolicyType = "on_failure"
restartPolicyMaxRetries = 3
```

### Steps

1. Push your repo to GitHub
2. Create a new project in [Railway](https://railway.app)
3. Add a **PostgreSQL** service (Railway provides it)
4. Connect your GitHub repo
5. Set environment variables:

| Variable | Value |
|----------|-------|
| `DATABASE_URL` | From Railway's Postgres service (auto-linked) |
| `JWT_SECRET` | 32+ random characters: `openssl rand -hex 32` |
| `TRUST_PROXY` | `true` (Railway runs a proxy in front of your app) |
| `FRONTEND_URL` | Your Railway app URL (e.g., `https://my-app.up.railway.app`) |
| `GOOGLE_CLIENT_ID` | From Google Cloud Console |
| `GOOGLE_CLIENT_SECRET` | From Google Cloud Console |
| `GOOGLE_REDIRECT_URI` | `https://my-app.up.railway.app/api/auth/google/callback` |

6. Deploy. Railway builds the Docker image and starts the container.

### Health Checks

Railway uses the `/api/health` endpoint to determine if the container is ready:

- `GET /api/health` — always returns `{"status": "ok"}` (process is alive)
- `GET /api/ready` — runs `SELECT 1` against the database (database is connected)

## Deploy to Other Platforms

### Fly.io

```bash
fly launch
fly secrets set DATABASE_URL=... JWT_SECRET=... TRUST_PROXY=true GOOGLE_CLIENT_ID=...
fly deploy
```

### Generic Docker

```bash
docker build -t my-app:latest .
docker run -p 3000:3000 \
  -e DATABASE_URL=postgres://... \
  -e JWT_SECRET=$(openssl rand -hex 32) \
  -e FRONTEND_URL=https://yourdomain.com \
  my-app:latest
```

### VPS / Bare Metal

```bash
# On the server:
cargo build --release --package backend
cd frontend && npm ci && npm run build && cd ..
cd backend && APP_ENV=production ../target/release/backend
```

Set environment variables via `.env` file or systemd unit.

## Database Migrations

Migrations run automatically on startup:

```rust
sqlx::migrate!("./migrations")
    .run(&db)
    .await
    .expect("Failed to run migrations");
```

Add new migrations as numbered SQL files in `backend/migrations/`:

```
002_widgets.sql
003_add_status_to_widgets.sql
```

They run in order, and SQLx tracks which have already been applied.

## Environment Variables Reference

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `APP_ENV` | No | `development` | `production` enables safety checks (set in the Docker image) |
| `DATABASE_URL` | No | `postgres://app:app@localhost:5433/app` | PostgreSQL connection string |
| `JWT_SECRET` | **Prod** | insecure dev value | Secret for signing JWTs (32+ chars in production) |
| `JWT_EXPIRY_SECONDS` | No | `86400` (24h) | How long JWTs are valid |
| `GOOGLE_CLIENT_ID` | No | — | Enables Google OAuth when set |
| `GOOGLE_CLIENT_SECRET` | No | — | Google OAuth secret |
| `GOOGLE_REDIRECT_URI` | No | — | OAuth callback URL |
| `TRUST_PROXY` | No | `false` | Read client IP from `X-Forwarded-For` (only behind a proxy) |
| `FRONTEND_URL` | **Prod** | `http://localhost:5173` | CORS origin (`https://` required in production) |
| `PORT` | No | `3000` | Server listen port |

## Graceful Shutdown

The server handles `SIGTERM` (sent by Docker/Railway on deploy) and `Ctrl+C`:

1. Stops accepting new connections
2. Waits for in-flight requests to complete
3. Exits cleanly

This ensures zero-downtime deployments when Railway rolls out a new version.

## Security Checklist for Production

- [ ] Set a strong, unique `JWT_SECRET` (the app refuses to start without one)
- [ ] Set `TRUST_PROXY=true` only if you run behind a reverse proxy
- [ ] Configure `FRONTEND_URL` to your actual domain (CORS)
- [ ] Set up Google OAuth with production redirect URI (dev login is disabled in production, so without it nobody can log in)
- [ ] Use HTTPS (Railway/Fly handle this automatically)
- [ ] Review database access (use connection pooling for high traffic)
