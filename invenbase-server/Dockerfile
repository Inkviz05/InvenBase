FROM rust:1-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY invenbase-server/Cargo.toml invenbase-server/Cargo.lock ./
COPY invenbase-server/build.rs ./build.rs
COPY invenbase-server/migrations ./migrations
COPY invenbase-server/static ./static
COPY invenbase-server/src ./src

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 wget \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/kvantoriym-server /usr/local/bin/kvantoriym-server
COPY --from=builder /app/static ./static

ENV HOST=0.0.0.0
ENV PORT=8080

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD wget -qO- http://127.0.0.1:8080/health || exit 1

CMD ["kvantoriym-server"]
