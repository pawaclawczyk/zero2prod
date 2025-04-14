FROM rust:1.86.0-bookworm
WORKDIR /app
RUN cargo install sqlx-cli
COPY migrations/ migrations/

ENTRYPOINT ["sqlx", "migrate", "run"]
