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

# El ile veya eski akışla migrate edilmiş DB: tablolar var, _koro_applied_migrations boş → dosyaları
# tekrar çalıştırmak CREATE hatası verir. Tüm migration adlarını kayda ekler (şema eşleşiyorsa).
track_count=$(psql -h "$PGHOST" -U "$PGUSER" -d "$PGDATABASE" -tAc \
  "SELECT count(*)::text FROM _koro_applied_migrations")
has_users=$(psql -h "$PGHOST" -U "$PGUSER" -d "$PGDATABASE" -tAc \
  "SELECT CASE WHEN to_regclass('public.users') IS NOT NULL THEN '1' ELSE '0' END")

if [[ "$track_count" == "0" && "$has_users" == "1" ]]; then
  if [[ "${MIGRATE_STRICT:-}" == "1" ]]; then
    echo "migrate-compose: Takip boş ama public.users var; MIGRATE_STRICT=1 — durduruluyor." >&2
    echo "migrate-compose: Postgres volume silin veya STRICT kaldırın (otomatik kayıt eşlemesi)." >&2
    exit 1
  fi
  echo "migrate-compose: Mevcut şema + boş takip — migration dosyaları zaten uygulanmış varsayılıyor (yalnızca kayıt)."
  for f in "${files[@]}"; do
    base=$(basename "$f")
    psql -h "$PGHOST" -U "$PGUSER" -d "$PGDATABASE" -v ON_ERROR_STOP=1 \
      -c "INSERT INTO _koro_applied_migrations (filename) VALUES ('$base') ON CONFLICT DO NOTHING"
  done
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
