FROM rust:1-alpine AS builder

WORKDIR /app

RUN apk add --no-cache musl-dev

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked

FROM alpine:latest

RUN apk add --no-cache ca-certificates \
    && adduser -D -H -u 10001 app

COPY --from=builder /app/target/release/acg-calendar /usr/local/bin/acg-calendar

ENV BIND=0.0.0.0:3000
EXPOSE 3000

USER app
ENTRYPOINT ["/usr/local/bin/acg-calendar"]
