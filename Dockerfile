FROM rust:1.94.1-alpine AS chef
RUN apk add --no-cache musl-dev
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=builder /app/target/release/atuin-web /usr/local/bin/
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/atuin-web", "--healthcheck"]
CMD ["atuin-web"]
