# Deployment

## Single Binary

The release binary is self-contained — all assets and templates are embedded.

```bash
cargo build --release
./target/release/atuin-web --atuin-server-url http://your-atuin-server:8888
```

## Docker

```dockerfile
FROM rust:1.93.1-alpine AS chef
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
```

## Reverse Proxy (nginx)

```nginx
server {
    listen 443 ssl;
    server_name atuin-web.example.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Rate Limiting (nginx)

Add `limit_req` to protect the login endpoint from brute-force attacks:

```nginx
http {
    limit_req_zone $binary_remote_addr zone=login:10m rate=5r/m;

    server {
        # ... existing config ...

        location /login {
            limit_req zone=login burst=3 nodelay;
            proxy_pass http://127.0.0.1:8080;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
    }
}
```

## Session Storage

The default in-memory session store is suitable for single-instance deployments with moderate traffic. Sessions expire after the configured TTL (default 24 hours). For high-traffic deployments, consider using a persistent session store or placing atuin-web behind a load balancer with sticky sessions.

## systemd

```ini
[Unit]
Description=Atuin Web UI
After=network.target

[Service]
Type=simple
User=atuin
Environment=ATUIN_WEB_SERVER_URL=http://127.0.0.1:8888
Environment=ATUIN_WEB_TOKEN=your-token
ExecStart=/usr/local/bin/atuin-web
Restart=on-failure

[Install]
WantedBy=multi-user.target
```
