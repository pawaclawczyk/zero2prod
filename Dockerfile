## Base
FROM lukemathwalker/cargo-chef:latest-rust-1.86.0 AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y

## Planner
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

## Builder
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLS_OFFLINE=true
RUN cargo build --release

## Runner
FROM debian:bullseye-slim AS runner
WORKDIR /APP
RUN apt update -y \
    && apt install -y --no-install-recommends openssl ca-certificates \
    && apt autoremove -y \
    && apt clean \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/zero2prod .
COPY etc/ etc/
ENV APP_ENVIRONMENT=production

ENTRYPOINT ["./zero2prod"]
