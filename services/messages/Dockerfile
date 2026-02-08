FROM rust:1.83 as builder

WORKDIR /app

ENV CARGO_TARGET_DIR=/app/target

COPY shared shared
COPY services/messages services/messages

RUN cargo build --release --manifest-path services/messages/Cargo.toml

FROM debian:bookworm
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/messages /usr/local/bin/messages

EXPOSE 8080
CMD ["messages"]
