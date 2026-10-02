# Multi-stage production build for Rust Game Server
FROM rust:1.85-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /build

COPY server/Cargo.toml server/Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY server/src ./src
RUN touch src/main.rs && cargo build --release

FROM alpine:3.21
WORKDIR /app
RUN apk add --no-cache ca-certificates tzdata

COPY --from=builder /build/target/release/tactical-arena-server /usr/local/bin/tactical-arena-server

EXPOSE 8080
ENV PORT=8080
ENV RUST_LOG=info

CMD ["tactical-arena-server"]
