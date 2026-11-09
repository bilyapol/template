# Template

A blank full-stack starter: **Rust (Axum)** backend + **React (TypeScript)** frontend + PostgreSQL.
One Docker image to deploy.

Based on [rust4all-template](https://github.com/rust4ai/rust4all-template) (MIT), cleaned up and hardened for production.

---

## 30-Second Start

Click **Use this template** on GitHub, or from the command line:

```bash
gh repo create my-app --template bilyapol/template --private --clone
cd my-app
cp .env.example .env
./dev.sh
```

Open [http://localhost:5173](http://localhost:5173).

`dev.sh` handles everything: starts PostgreSQL (Docker), the Rust backend, and the React dev server.

Each project gets its own database volume, but they all use port 5433, so stop one project's
database (`docker compose stop db`) before starting another's.

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
│   │   ├── state.rs         Shared app state (DB pool, config)
│   │   └── controllers/     Route handlers (health)
│   └── migrations/          Auto-run on startup
├── frontend/                React 19 SPA
│   └── src/
│       ├── api/             Typed HTTP client
│       └── components/      Layout and pages
├── .github/workflows/       CI: fmt, clippy, tests, lint, build
├── dev.sh                   One-command dev setup
├── Dockerfile               Multi-stage production build
└── docker-compose.yml       Local PostgreSQL
```

---

## Stack

| Layer | Tech |
|-------|------|
| **API** | Axum 0.8 + Tokio |
| **Database** | PostgreSQL + SQLx (raw SQL, no ORM) |
| **Frontend** | React 19 + TypeScript + Vite 8 |
| **Styling** | Tailwind CSS 4 |
| **Deploy** | Single Docker image |

---

## Features

- Zero-config development: every setting has a default, the app boots with an empty `.env`
- Generic 500 responses (details are logged, never sent to the client)
- Security headers (CSP, HSTS, X-Frame-Options, Referrer-Policy, Permissions-Policy)
- `/api/health` (liveness) and `/api/ready` (database check)
- JSON 404 for unknown API routes; everything else serves the SPA
- Non-root Docker user and graceful shutdown

---

## Extending

### Add a resource

```
1. backend/migrations/001_widgets.sql        → CREATE TABLE
2. backend/src/controllers/widgets.rs        → handlers + queries
3. backend/src/main.rs                       → routes
4. frontend/src/api/widgets.ts               → API client
5. frontend/src/components/pages/Widgets.tsx → UI
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

## Checks

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test
cd frontend && npm run lint && npm run build
```

The same checks run in GitHub Actions on every push and pull request.

---

## License

MIT — see the original [rust4all-template](https://github.com/rust4ai/rust4all-template).
