ARG RUST_VERSION=1.85
FROM rust:${RUST_VERSION}-slim-bullseye AS base
RUN apt update && apt -y install wget pkg-config libssl-dev && \
  rustup target add wasm32-unknown-unknown && \
  mkdir -p /app

FROM base AS deps
WORKDIR /app
RUN mkdir -p crates/api/src crates/backend/src crates/frontend/src crates/infrastructure/src crates/discovery/src crates/shared/src crates/tasks/src tests
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/api/Cargo.toml ./crates/api/
COPY crates/backend/Cargo.toml ./crates/backend/
COPY crates/frontend/Cargo.toml ./crates/frontend/
COPY crates/infrastructure/Cargo.toml ./crates/infrastructure/
COPY crates/discovery/Cargo.toml ./crates/discovery/
COPY crates/shared/Cargo.toml ./crates/shared/
COPY crates/tasks/Cargo.toml ./crates/tasks/
COPY tests/Cargo.toml ./tests/
COPY rust-toolchain.toml ./
ENV SQLX_OFFLINE=true
RUN echo 'fn main() {}' > crates/api/src/main.rs && cp crates/api/src/main.rs crates/frontend/src/main.rs && \
  cp crates/api/src/main.rs crates/tasks/src/main.rs && \
  touch crates/api/src/lib.rs crates/backend/src/lib.rs crates/frontend/src/lib.rs crates/infrastructure/src/lib.rs crates/discovery/src/lib.rs crates/shared/src/lib.rs && \
  cargo build --release --workspace

FROM deps AS build
WORKDIR /app
COPY crates crates/
COPY migrations migrations/
COPY .sqlx .sqlx/
COPY .env .env
# Build the application
ENV SQLX_OFFLINE=true
ENV RUSTFLAGS="-Zlocation-detail=none"
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/app/target \
  cargo build --release --workspace && \
  cp /app/target/release/api ./easm

FROM cgr.dev/chainguard/glibc-dynamic
WORKDIR /app
COPY --from=build --chown=nonroot:nonroot /app/easm /app/easm
COPY --from=build --chown=nonroot:nonroot /app/.env /app/.env
ENV RUST_LOG="info,tower_http=info,axum:rejection=info"
USER nonroot
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 CMD wget --no-verbose --tries=1 --spider http://localhost:8080/api/v1/health || exit 1
CMD ["/app/easm"]
