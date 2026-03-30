#!/usr/bin/env bash
# docker-compose.local.yml sırayı kendi çözüyor; bu script yalnızca kısayol.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
docker compose -f docker-compose.local.yml up --build -d
