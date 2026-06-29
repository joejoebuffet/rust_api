# --- Stage 1: Build Stage ---
FROM rust:1.85-slim AS builder

WORKDIR /app

# Install compilation dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml ./

# Trick to cache dependencies if you want, or just copy src directly
COPY src ./src

# Build production binary
RUN cargo build --release

# --- Stage 2: Runtime Stage ---
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# OpenSSL is required at runtime by sqlx native-tls features
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/rust_api /app/rust_api

EXPOSE 8081

CMD ["/app/rust_api"]