#!/usr/bin/env bash
# Start the Rust API and the Vite dev server together.
# Ctrl+C stops both. API_HOST, API_PORT, FRONTEND_HOST, and FRONTEND_PORT
# override the defaults (127.0.0.1:3000 and 127.0.0.1:5173).

set -euo pipefail
set -m

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export API_HOST="${API_HOST:-127.0.0.1}"
export API_PORT="${API_PORT:-3000}"
export FRONTEND_HOST="${FRONTEND_HOST:-127.0.0.1}"
export FRONTEND_PORT="${FRONTEND_PORT:-5173}"

if [[ ! -d node_modules ]]; then
  echo "node_modules is missing. From the repo root, run: npm ci" >&2
  exit 1
fi

api_pid=""
web_pid=""

cleanup() {
  local status=$?
  trap - EXIT INT TERM
  if [[ -n "$api_pid" ]]; then
    kill -TERM -- "-$api_pid" 2>/dev/null || kill -TERM "$api_pid" 2>/dev/null || true
  fi
  if [[ -n "$web_pid" ]]; then
    kill -TERM -- "-$web_pid" 2>/dev/null || kill -TERM "$web_pid" 2>/dev/null || true
  fi
  sleep 0.3
  if [[ -n "$api_pid" ]]; then
    kill -KILL -- "-$api_pid" 2>/dev/null || true
  fi
  if [[ -n "$web_pid" ]]; then
    kill -KILL -- "-$web_pid" 2>/dev/null || true
  fi
  exit "$status"
}
trap cleanup EXIT INT TERM

echo "API  http://${API_HOST}:${API_PORT}"
echo "UI   http://${FRONTEND_HOST}:${FRONTEND_PORT}"

(cd "$ROOT/backend" && exec cargo run) &
api_pid=$!

echo "Waiting for the API to listen..."
ready=0
for _ in $(seq 1 180); do
  if (echo >/dev/tcp/"$API_HOST"/"$API_PORT") >/dev/null 2>&1; then
    ready=1
    break
  fi
  if ! kill -0 "$api_pid" 2>/dev/null; then
    echo "The API process exited before it started listening." >&2
    wait "$api_pid" || true
    exit 1
  fi
  sleep 1
done
if [[ "$ready" -ne 1 ]]; then
  echo "The API did not listen on ${API_HOST}:${API_PORT} within 180s." >&2
  exit 1
fi

npm run dev &
web_pid=$!

wait -n "$api_pid" "$web_pid"
