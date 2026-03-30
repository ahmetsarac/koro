#!/usr/bin/env bash
# Tek komut: local compose için sıra — postgres → migrate → sqlx-prepare → up --build
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
COMPOSE=(docker compose -f docker-compose.local.yml)

"${COMPOSE[@]}" up -d postgres
"${COMPOSE[@]}" run --rm migrate
"${COMPOSE[@]}" run --rm sqlx-prepare
"${COMPOSE[@]}" up --build -d
