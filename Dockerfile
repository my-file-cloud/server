# ── Stage 1: builder ────────────────────────────────────────
FROM rust:1-slim AS builder

# Native dependencies required by sqlx (openssl/tls)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

COPY . .

ENV SQLX_OFFLINE=true

RUN cargo build --release --no-default-features --features db

# ── Stage 2: runtime ────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /build/target/release/server ./server

RUN mkdir -p /data/cloud /data/logs \
    && useradd -r -s /bin/false appuser \
    && chown -R appuser:appuser /app /data/cloud /data/logs

USER appuser

CMD ["./server"]
