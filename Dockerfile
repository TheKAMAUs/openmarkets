# =========================
# 1) PREP STAGE (cargo-chef)
# =========================
FROM rust:1.92-bullseye AS chef

WORKDIR /app
ARG DATABASE_URL

RUN cargo install cargo-chef
COPY . .

RUN cargo chef prepare --recipe-path recipe.json


# =========================
# 2) BUILDER STAGE
# =========================
FROM rust:1.92-bullseye AS builder

# Install build env + protobuf
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    zlib1g-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN cargo install cargo-chef

COPY --from=chef /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

ENV DATABASE_URL=$DATABASE_URL
ENV SQLX_OFFLINE=true

COPY . .

RUN cargo build --release --workspace --bins
RUN ls -la /app/target/release


# =========================
# 3) RUNTIME STAGE
# =========================
FROM debian:bullseye-slim AS runtime

WORKDIR /usr/local/bin

# FIX TLS ERROR — install system CA certificates
RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    && update-ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries from builder
COPY --from=builder /app/target/release/grpc-service .
COPY --from=builder /app/target/release/order-service .
COPY --from=builder /app/target/release/service-api .
COPY --from=builder /app/target/release/websocket-service .

RUN chmod +x grpc-service order-service service-api websocket-service

# Default service to run (change this if needed)
CMD ["./grpc-service"]