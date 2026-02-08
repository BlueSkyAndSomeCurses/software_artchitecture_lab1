FROM rust:1.83 as builder

WORKDIR /app

ENV CARGO_TARGET_DIR=/app/target

COPY shared shared
COPY services/facade services/facade

RUN cargo build --release --manifest-path services/facade/Cargo.toml

FROM debian:bookworm
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/facade /usr/local/bin/facade

EXPOSE 8080
CMD ["facade"]
