FROM rust:1.88 as builder

WORKDIR /app

ENV CARGO_TARGET_DIR=/app/target

COPY shared shared
COPY services/config-server services/config-server

RUN cargo build --release --manifest-path services/config-server/Cargo.toml

FROM debian:bookworm
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/config-server /usr/local/bin/config-server

EXPOSE 8083
CMD ["config-server"]