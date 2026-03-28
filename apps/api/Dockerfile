# Builder: internet + erişilebilir Postgres (migration'lı) gerekir — sqlx makroları şemayı doğrular.
# Runtime: yalnızca statik binary — ağ dışına çıkmaz.

FROM rust:1-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

ARG DATABASE_URL
ENV DATABASE_URL=${DATABASE_URL}

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
