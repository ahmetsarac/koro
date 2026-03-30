#!/usr/bin/env bash
# Varsayılan docker-compose.yml: postgres → migrate → sqlx-prepare → up --build
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
docker compose up -d postgres
docker compose run --rm migrate
docker compose run --rm sqlx-prepare
docker compose up --build -d
