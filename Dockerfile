FROM rust:1.93.1 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/* \
    && useradd -r -u 1000 -s /usr/sbin/nologin atuin-web
COPY --from=builder /app/target/release/atuin-web /usr/local/bin/
USER atuin-web
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://127.0.0.1:8080/login || exit 1
CMD ["atuin-web"]
