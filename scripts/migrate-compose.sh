#!/usr/bin/env bash
# Docker migrate servisi (postgres:16). Windows CRLF için compose entrypoint sed ile temizler.
set -euo pipefail

: "${PGHOST:=postgres}"
: "${PGUSER:=koro}"
: "${PGPASSWORD:=koro}"
: "${PGDATABASE:=koro}"
export PGPASSWORD

until pg_isready -h "$PGHOST" -U "$PGUSER" -d "$PGDATABASE" -q; do
  sleep 1
done

psql -h "$PGHOST" -U "$PGUSER" -d "$PGDATABASE" -v ON_ERROR_STOP=1 <<'EOSQL'
CREATE TABLE IF NOT EXISTS _koro_applied_migrations (
  filename TEXT PRIMARY KEY,
  applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
EOSQL

mapfile -t files < <(find /migrations -maxdepth 1 -type f -name '*.sql' | LC_ALL=C sort -V)
if [[ ${#files[@]} -eq 0 ]]; then
  echo "migrate-compose: /migrations altında .sql yok" >&2
  exit 1
fi

for f in "${files[@]}"; do
  base=$(basename "$f")
  if [[ ! "$base" =~ ^[0-9]{4}_.*\.sql$ ]]; then
    echo "migrate-compose: beklenmeyen dosya adı: $base" >&2
    exit 1
  fi
  count=$(psql -h "$PGHOST" -U "$PGUSER" -d "$PGDATABASE" -tAc \
    "SELECT count(*)::text FROM _koro_applied_migrations WHERE filename = '$base'")
  if [[ "$count" == "0" ]]; then
    echo "Applying $base"
    psql -h "$PGHOST" -U "$PGUSER" -d "$PGDATABASE" -v ON_ERROR_STOP=1 -f "$f"
    psql -h "$PGHOST" -U "$PGUSER" -d "$PGDATABASE" -v ON_ERROR_STOP=1 \
      -c "INSERT INTO _koro_applied_migrations (filename) VALUES ('$base')"
  else
    echo "Skip $base"
  fi
done
echo "migrate-compose: tamam (${#files[@]} dosya kontrol edildi)."
