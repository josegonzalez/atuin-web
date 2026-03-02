FROM rust:1.93.1 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/atuin-web /usr/local/bin/
EXPOSE 8080
CMD ["atuin-web"]
