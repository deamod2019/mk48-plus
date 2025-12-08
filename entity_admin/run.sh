#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_PORT="${BACKEND_PORT:-9000}"
BACKEND_HOST="${BACKEND_HOST:-0.0.0.0}"
FRONTEND_PORT="${FRONTEND_PORT:-5179}"
FRONTEND_HOST="${FRONTEND_HOST:-0.0.0.0}"

backend() {
  cd "$ROOT_DIR/backend"
  if [ ! -d node_modules ]; then
    npm install
  fi
  PORT="$BACKEND_PORT" HOST="$BACKEND_HOST" npm run dev
}

frontend() {
  cd "$ROOT_DIR/frontend"
  if [ ! -d node_modules ]; then
    npm install
  fi
  # 监听 FRONTEND_HOST:FRONTEND_PORT，默认 0.0.0.0:5179
  npm run dev -- --host "$FRONTEND_HOST" --port "$FRONTEND_PORT"
}

trap 'kill 0' INT TERM

backend &
backend_pid=$!
frontend &
frontend_pid=$!

echo "后台 PID: $backend_pid, 前端 PID: $frontend_pid"
wait
