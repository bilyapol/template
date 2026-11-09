#!/usr/bin/env bash
set -e

ROOT="$(cd "$(dirname "$0")" && pwd)"

# Colors
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
YELLOW='\033[0;33m'
GRAY='\033[0;90m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${CYAN}▸ template${PURPLE} dev${NC}"
echo -e "${GRAY}  backend  → http://localhost:${PORT:-3000}${NC}"
echo -e "${GRAY}  frontend → http://localhost:5173  (proxies /api → :${PORT:-3000})${NC}"
echo ""

# Check for .env
if [ ! -f "$ROOT/.env" ]; then
  echo -e "${RED}✗ No .env file found. Copy .env.example and fill it in:${NC}"
  echo "  cp .env.example .env"
  exit 1
fi

# Load .env for the backend
set -a
source "$ROOT/.env"
set +a

# Start local postgres only if DATABASE_URL is not set
if [ -z "$DATABASE_URL" ]; then
  export DATABASE_URL="postgres://app:app@localhost:5433/app"
  if ! pg_isready -h localhost -p 5433 -q 2>/dev/null; then
    echo -e "${YELLOW}▸ starting postgres...${NC}"
    docker compose -f "$ROOT/docker-compose.yml" up db -d --wait
  fi
fi

# Install frontend deps if needed (re-runs when package.json changes)
if [ ! -d "$ROOT/frontend/node_modules" ] || \
   [ "$ROOT/frontend/package.json" -nt "$ROOT/frontend/node_modules" ]; then
  echo -e "${GRAY}  installing frontend deps...${NC}"
  (cd "$ROOT/frontend" && npm install)
fi

cleanup() {
  echo ""
  echo -e "${GRAY}shutting down...${NC}"
  kill -- -$BACKEND_PID -$FRONTEND_PID 2>/dev/null
  wait $BACKEND_PID $FRONTEND_PID 2>/dev/null
  echo -e "${CYAN}▸ done${NC}"
}
# Run in its own process group so we can kill the whole tree
set -m
trap cleanup EXIT INT TERM

# Start backend (Rust/Cargo)
echo -e "${PURPLE}▸ starting backend...${NC}"
(cd "$ROOT/backend" && cargo run 2>&1 | sed "s/^/  ${GRAY}[backend]${NC}  /") &
BACKEND_PID=$!

# Start frontend (Vite dev server with hot reload)
echo -e "${CYAN}▸ starting frontend...${NC}"
(cd "$ROOT/frontend" && npx vite --host 2>&1 | sed "s/^/  ${GRAY}[frontend]${NC} /") &
FRONTEND_PID=$!

# Wait for any to exit
wait -n $BACKEND_PID $FRONTEND_PID
echo -e "${RED}▸ a process exited unexpectedly${NC}"
cleanup
