# Builder: SQLX_OFFLINE — önce `docker compose --profile tools run --rm sqlx-prepare` ile .sqlx üretin.
# Runtime: yalnızca binary.

FROM rust:1-bookworm AS builder
WORKDIR /app

ENV SQLX_OFFLINE=true

COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx
COPY src ./src

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
