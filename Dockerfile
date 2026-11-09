# Stage 1: Build Rust backend
FROM rust:1.91-bookworm AS rust-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY backend/ backend/
ENV SQLX_OFFLINE=true
RUN cargo build --release --package backend

# Stage 2: Build frontend
FROM node:20-bookworm AS node-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ .
RUN npm run build

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

ENV APP_ENV=production
WORKDIR /app/backend
COPY --from=rust-builder /app/target/release/backend ./backend
COPY backend/migrations ./migrations
COPY --from=node-builder /app/frontend/dist /app/frontend/dist
RUN useradd -r -u 1001 appuser
USER appuser

EXPOSE 3000
CMD ["./backend"]
