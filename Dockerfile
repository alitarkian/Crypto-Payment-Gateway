# ─── Stage 1: Build ──────────────────────────────────────────────────────────
FROM rust:1.94-slim-bookworm AS builder

RUN apt-get update && \
    apt-get install -y --no-install-recommends pkg-config libssl-dev curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies: build a stub binary first so cargo fetches & compiles deps
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    cargo build --release && \
    rm -f target/release/deps/crypto_payment_gateway*

# Build the real application
# SQLX_OFFLINE=true uses the .sqlx/ query cache instead of a live DB
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

# ─── Stage 2: Runtime ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy only the binary (migrations are embedded by sqlx::migrate! macro)
COPY --from=builder /app/target/release/crypto-payment-gateway ./

# Run as non-root
RUN useradd -r -s /bin/false gateway && chown gateway:gateway ./crypto-payment-gateway
USER gateway

EXPOSE 8080

CMD ["./crypto-payment-gateway"]
