# Deployment

## Single Binary

The release binary is self-contained — all assets and templates are embedded.

```bash
cargo build --release
./target/release/atuin-web --atuin-server-url http://your-atuin-server:8888
```

## Docker

```dockerfile
FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/atuin-web /usr/local/bin/
EXPOSE 8080
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
