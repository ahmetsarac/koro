#!/usr/bin/env bash
set -euo pipefail

# Sıra önemli; tüm db/migrations/*.sql dosyaları ad sırasıyla uygulanır.
# Windows: scripts/apply-migrations.ps1 kullanın.
# Varsayılan konteyner: docker-compose.yml (koro-postgres).
# Local compose için: POSTGRES_CONTAINER=koro-postgres-local ./scripts/apply-migrations.sh

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CONTAINER="${POSTGRES_CONTAINER:-koro-postgres}"

shopt -s nullglob
files=("$ROOT"/db/migrations/*.sql)
shopt -u nullglob

if [[ ${#files[@]} -eq 0 ]]; then
  echo "No migrations under $ROOT/db/migrations" >&2
  exit 1
fi

IFS=$'\n'
sorted=($(sort <<<"${files[*]}"))
unset IFS

for f in "${sorted[@]}"; do
  echo "==> $(basename "$f")"
  docker exec -i "$CONTAINER" psql -U koro -d koro <"$f"
done

echo "Done. (${#sorted[@]} file(s))"
