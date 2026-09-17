# Deployment

## Single Binary

The release binary is self-contained — all assets and templates are embedded.

```bash
cargo build --release
./target/release/atuin-web --atuin-server-url http://your-atuin-server:8888
```

## Docker

Multi-arch images (`linux/amd64`, `linux/arm64`) are published to GHCR on each release.

```bash
docker run -p 8080:8080 \
  -e ATUIN_WEB_BIND=0.0.0.0:8080 \
  -e ATUIN_WEB_SERVER_URL=http://your-atuin-server:8888 \
  ghcr.io/josegonzalez/atuin-web:v0.2.1
```

Replace `v0.2.0` with the desired release tag. Available tags are listed on the
[packages page](https://github.com/josegonzalez/atuin-web/pkgs/container/atuin-web).

## Docker Compose

```yaml
services:
  atuin-web:
    image: ghcr.io/josegonzalez/atuin-web:v0.2.1
    ports:
      - "8080:8080"
    environment:
      ATUIN_WEB_BIND: "0.0.0.0:8080"
      ATUIN_WEB_SERVER_URL: "http://atuin-server:8888"
      ATUIN_WEB_TOKEN: "your-token"
    restart: unless-stopped
```

If your atuin server is also running in Docker Compose, place both services in the
same Compose file (or use an external network) so `atuin-web` can reach it by
service name:

```yaml
services:
  atuin-server:
    image: ghcr.io/atuinsh/atuin:latest
    # ... your atuin server config ...

  atuin-web:
    image: ghcr.io/josegonzalez/atuin-web:v0.2.1
    ports:
      - "8080:8080"
    environment:
      ATUIN_WEB_BIND: "0.0.0.0:8080"
      ATUIN_WEB_SERVER_URL: "http://atuin-server:8888"
      ATUIN_WEB_TOKEN: "your-token"
    restart: unless-stopped
    depends_on:
      - atuin-server
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

        location /proxy/login {
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

## NixOS

The repository is a flake, exposing a package, an overlay and a NixOS module.

Try it without installing anything:

```bash
nix run github:josegonzalez/atuin-web -- --atuin-server-url http://127.0.0.1:8888
```

Add it to a system configuration:

```nix
{
  inputs.atuin-web.url = "github:josegonzalez/atuin-web";

  outputs = { nixpkgs, atuin-web, ... }: {
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      modules = [
        atuin-web.nixosModules.default
        {
          nixpkgs.overlays = [ atuin-web.overlays.default ];

          services.atuin-web = {
            enable = true;
            port = 8080;
            atuinServerUrl = "http://127.0.0.1:8888";
            secureCookies = true;
          };
        }
      ];
    };
  };
}
```

The module runs atuin-web under `DynamicUser` with a restrictive sandbox, and
listens on loopback by default so a reverse proxy can sit in front. Settings that
should not land in the world-readable Nix store go in an environment file:

```nix
services.atuin-web.environmentFile = "/run/secrets/atuin-web";
```

```env
ATUIN_WEB_TOKEN=your-token
```

Options: `host`, `port`, `atuinServerUrl`, `sessionExpiry`, `logLevel`,
`secureCookies`, `openFirewall`, `environmentFile` and `package`.

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
