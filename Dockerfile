## Base
FROM lukemathwalker/cargo-chef:latest-rust-1.86.0-alpine AS chef

WORKDIR /app

## Planner
FROM chef AS planner

COPY Cargo.lock .
COPY Cargo.toml .

RUN cargo chef prepare --recipe-path recipe.json

## Builder
FROM chef AS builder

RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig \
    clang \
    lld \
    curl \
    make \
    bash

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

# Build for musl target (default in Alpine)
RUN cargo build --release

FROM scratch

ENV APP_ENVIRONMENT=production

WORKDIR /app

COPY --from=builder /app/target/release/zero2prod .
COPY configuration.yaml .
COPY production.yaml .

ENTRYPOINT ["/app/zero2prod"]
