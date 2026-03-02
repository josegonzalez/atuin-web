FROM rust:1.93.1-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:3.21
RUN apk add --no-cache ca-certificates curl \
    && adduser -D -u 1000 -s /sbin/nologin atuin-web
COPY --from=builder /app/target/release/atuin-web /usr/local/bin/
USER atuin-web
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://127.0.0.1:8080/login || exit 1
CMD ["atuin-web"]
