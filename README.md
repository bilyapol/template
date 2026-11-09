# Template

A blank full-stack starter: **Rust (Axum)** backend + **React (TypeScript)** frontend + PostgreSQL.
Google OAuth when you need it, dev-mode login when you don't. One Docker image to deploy.

Based on [rust4all-template](https://github.com/rust4ai/rust4all-template) (MIT), cleaned up and hardened for production.

---

## 30-Second Start

```bash
git clone https://github.com/<you>/template.git my-app
cd my-app
cp .env.example .env
./dev.sh
```

Open [http://localhost:5173](http://localhost:5173). Click "Dev Login." You're in.

`dev.sh` handles everything: starts PostgreSQL (Docker), the Rust backend, and the React dev server.

### What You Need

- [Rust](https://rustup.rs/) 1.85+ (edition 2024)
- [Node.js](https://nodejs.org/) 20+
- [Docker](https://docker.com/) (for PostgreSQL)

---

## What's Inside

```
template/
├── backend/                 Rust API server (Axum 0.8)
│   ├── src/
│   │   ├── main.rs          Server setup, routing, middleware
│   │   ├── config.rs        Zero-config env loading
│   │   ├── error.rs         Typed errors → HTTP status codes
│   │   ├── controllers/     Route handlers (auth, user, docs, health)
│   │   ├── db/              SQL queries (users)
│   │   └── middleware/      JWT auth extractors, rate-limit key
│   └── migrations/          Auto-run on startup
├── frontend/                React 19 SPA
│   └── src/
│       ├── api/             Typed HTTP client
│       ├── stores/          Zustand state management
│       └── components/      Pages, layout, UI
├── docs/                    Markdown documentation (served in-app)
├── .github/workflows/       CI: fmt, clippy, tests, lint, build
├── dev.sh                   One-command dev setup
├── Dockerfile               Multi-stage production build
├── docker-compose.yml       Local PostgreSQL
└── railway.toml             Railway deployment config
```

---

## Stack

| Layer | Tech | Why |
|-------|------|-----|
| **API** | Axum 0.8 + Tokio | Type-safe, async, fast |
| **Database** | PostgreSQL + SQLx | Raw SQL, no ORM |
| **Auth** | Google OAuth / dev login + JWT | Production & dev-friendly |
| **Frontend** | React 19 + TypeScript + Vite 8 | Fast builds, hot reload |
| **Styling** | Tailwind CSS 4 | Utility-first, dark mode |
| **State** | Zustand | Minimal boilerplate |
| **Deploy** | Docker + Railway | Single image, zero config |

---

## Features

### Zero-Config Development
Every config field has a default. The app boots with an empty `.env`. Dev-mode email login means no Google Cloud Console needed to start building.

### Production Auth
When you're ready, add Google OAuth credentials and the app auto-switches to real OIDC authentication with CSRF protection, nonce verification, and HttpOnly JWT cookies.

### Security Baked In
- `APP_ENV=production` refuses to start with a weak `JWT_SECRET` or non-https URL, disables dev login, and marks cookies `Secure`
- Generic 500 responses (details are logged, never sent to the client)
- Rate limiting per client IP, not spoofable unless you opt in with `TRUST_PROXY=true` behind a proxy
- Security headers (CSP, HSTS, X-Frame-Options, Referrer-Policy, Permissions-Policy)
- CORS configured per environment
- Non-root Docker user
- Graceful shutdown for zero-downtime deploys

### Built-In Documentation
Three doc pages served from markdown files in `docs/`, rendered in-app with a sidebar nav. The landing page links directly to them.

---

## Auth Modes

| Mode | When | How |
|------|------|-----|
| **Dev** | Development and no Google credentials | Enter any email to log in instantly |
| **Google** | OAuth credentials configured | Full OIDC flow with CSRF + nonce |
| **None** | Production without Google credentials | Login is disabled |

The frontend auto-detects the mode via `GET /api/auth/mode`.

---

## Deploy to Railway

1. Push to GitHub
2. Create a project in [Railway](https://railway.app)
3. Add PostgreSQL (Railway provides it)
4. Connect your repo
5. Set env vars: `DATABASE_URL`, `JWT_SECRET` (`openssl rand -hex 32`), `FRONTEND_URL` (https), `TRUST_PROXY=true`, Google OAuth creds
6. Deploy

The included `railway.toml` configures health checks and restart policy automatically.

---

## Extending

### Add a resource

```
1. backend/migrations/002_widgets.sql        → CREATE TABLE
2. backend/src/db/widgets.rs                 → queries
3. backend/src/controllers/widgets.rs        → handlers
4. backend/src/main.rs                       → routes
5. frontend/src/api/widgets.ts               → API client
6. frontend/src/stores/widgets.store.ts      → state
7. frontend/src/components/pages/Widgets.tsx → UI
```

### Add auth to a route

```rust
pub async fn my_handler(
    auth: AuthUser,  // ← add this, route is now protected
    State(state): State<AppState>,
) -> Result<Json<MyData>, AppError> {
    // auth.user_id is available
}
```

### Add a background task

```rust
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        // your work here
    }
});
```

---

## Documentation

The template includes in-app documentation:

- [**Axum Backend**](/docs/backend) — Architecture, auth, database, error handling
- [**React Frontend**](/docs/frontend) — Vite, Zustand, API client, routing
- [**Deployment**](/docs/deployment) — Docker, Railway, environment variables

Or read the markdown files directly in [`docs/`](./docs/).

---

## Checks

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test
cd frontend && npm run lint && npm run build
```

The same checks run in GitHub Actions on every push and pull request.

---

## License

MIT — see the original [rust4all-template](https://github.com/rust4ai/rust4all-template).
