FROM rust:1.85-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /build

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM alpine:3.21
WORKDIR /app
RUN apk add --no-cache ca-certificates tzdata

COPY --from=builder /build/target/release/tactical-arena-server /usr/local/bin/tactical-arena-server

EXPOSE 8080
ENV PORT=8080

CMD ["tactical-arena-server"]
