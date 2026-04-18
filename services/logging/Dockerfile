FROM rust:1.88 as builder

WORKDIR /app

ENV CARGO_TARGET_DIR=/app/target

COPY shared shared
COPY services/logging services/logging

RUN cargo build --release --manifest-path services/logging/Cargo.toml

FROM debian:bookworm
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/logging /usr/local/bin/logging

EXPOSE 8082
CMD ["logging"]
