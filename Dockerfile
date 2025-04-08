FROM rust:1.86.0-alpine AS builder

RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig \
    clang \
    lld \
    curl \
    make \
    bash

WORKDIR /app

COPY . .

# Build for musl target (default in Alpine)
RUN cargo build --release

FROM scratch

ENV APP_ENVIRONMENT=production

WORKDIR /app

COPY --from=builder /app/target/release/zero2prod .
COPY --from=builder /app/configuration.yaml .
COPY --from=builder /app/production.yaml .

ENTRYPOINT ["/app/zero2prod"]
