# Rust: 1.92 | Node: 20 | pnpm: latest
# Multi-stage build for optimized image size

# Stage 1: Build frontend
FROM node:20-alpine AS frontend
LABEL stage=frontend

WORKDIR /app/web
RUN npm install -g pnpm
COPY web/package.json web/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile --prefer-offline
COPY web/ ./
RUN pnpm run build

# Stage 2: Build dependencies (for layer caching)
FROM rust:1.92-slim AS dependencies
LABEL stage=dependencies
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock build.rs ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Stage 3: Build backend
FROM rust:1.92-slim AS backend
LABEL stage=backend
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=dependencies /app/target /app/target
COPY Cargo.toml Cargo.lock build.rs package.json ./
COPY src/ ./src/
COPY --from=frontend /app/web/dist ./web/dist
RUN cargo build --release

# Stage 4: Runtime
FROM debian:bookworm-slim
LABEL maintainer="lynn1286"
LABEL description="Feishu Bot - Sentry alert forwarding service"
LABEL version="1.0"

WORKDIR /app
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates wget && \
    rm -rf /var/lib/apt/lists/* && \
    useradd -m -u 1000 -s /bin/bash feishu

COPY --from=backend /app/target/release/feishu-bot /usr/local/bin/
RUN chown -R feishu:feishu /app

ENV FEISHU_BOT_DATA_DIR=/app/data
ENV DATABASE_URL=sqlite:/app/data/data.db?mode=rwc

EXPOSE 3000
VOLUME ["/app/data"]

USER feishu
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1

CMD ["feishu-bot", "serve", "--host", "0.0.0.0"]
