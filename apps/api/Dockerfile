# Builder: SQLX_OFFLINE — .sqlx içinde sqlx-prepare çıktısı (.json) olmalı.
# Boş klasörde build: scripts/up-local.ps1 / up-local.sh (postgres → migrate → sqlx-prepare → build).
# Runtime: yalnızca binary.

FROM rust:1-bookworm AS builder
WORKDIR /app

ENV SQLX_OFFLINE=true

COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx
COPY src ./src

RUN cache=$(find .sqlx -type f -name '*.json' 2>/dev/null | wc -l); \
    if [ "$cache" -eq 0 ]; then \
      echo "HATA: apps/api/.sqlx altında .json yok (sqlx-prepare çalışmadı)."; \
      echo "Çözüm: PowerShell: .\\scripts\\up-local.ps1    veya"; \
      echo "  docker compose -f docker-compose.local.yml up -d postgres"; \
      echo "  docker compose -f docker-compose.local.yml run --rm migrate"; \
      echo "  docker compose -f docker-compose.local.yml run --rm sqlx-prepare"; \
      echo "  sonra: docker compose -f docker-compose.local.yml build api"; \
      exit 1; \
    fi

RUN cargo build --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/api /app/api

ENV PORT=3001
EXPOSE 3001

USER nobody
CMD ["/app/api"]
