FROM rust:1.86.0-bookworm

RUN cargo install sqlx-cli

COPY migrations/ migrations/

ENTRYPOINT ["sqlx", "migrate", "run"]
