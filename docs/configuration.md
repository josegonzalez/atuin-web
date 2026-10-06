# Configuration

All options can be set via CLI args or environment variables. CLI args take precedence.

| Option | Env Var | Default | Description |
|--------|---------|---------|-------------|
| `--bind` | `ATUIN_WEB_BIND` | `127.0.0.1:8080` | Bind address |
| `--atuin-server-url` | `ATUIN_WEB_SERVER_URL` | `http://127.0.0.1:8888` | Upstream atuin server |
| `--token` | `ATUIN_WEB_TOKEN` | (none) | Pre-configured auth token |
| `--session-expiry` | `ATUIN_WEB_SESSION_EXPIRY` | `86400` | Session TTL (seconds) |
| `--log-level` | `ATUIN_WEB_LOG_LEVEL` | `info` | Log level |
| `--log-format` | `ATUIN_WEB_LOG_FORMAT` | `text` | Log format: `text`, `json` or `ecs` (see [Log Formats](#log-formats)) |
| `--secure-cookies` | `ATUIN_WEB_SECURE_COOKIES` | `false` | Set Secure flag on cookies (enable behind HTTPS) |
| `--healthcheck` | — | `false` | Probe `GET /healthz` and exit 0/1; used by Docker HEALTHCHECK |

## Example .env

```env
ATUIN_WEB_BIND=0.0.0.0:8080
ATUIN_WEB_SERVER_URL=http://localhost:8888
ATUIN_WEB_TOKEN=your-session-token
ATUIN_WEB_LOG_LEVEL=info
```

## Log Formats

- `text` — human-readable lines. Colored only when stdout is a terminal, so container logs carry no ANSI escape codes.
- `json` — one JSON object per line: `timestamp`, `level`, `target`, `message` and any event fields.
- `ecs` — one [Elastic Common Schema](https://www.elastic.co/guide/en/ecs/current/index.html) object per line: `@timestamp`, `log.level`, `log.logger`, `log.origin.file.*`, `message`, `ecs.version`, `service.name` and `service.version`. Other event fields go into `labels` as strings.

`RUST_LOG` overrides `--log-level` in every format.

## Upstream Request Timeouts

All HTTP requests to the upstream atuin server have a 30-second request timeout and a 10-second connection timeout. These are not currently configurable.

## Getting Your Auth Token

```bash
sqlite3 ~/.local/share/atuin/meta.db "SELECT value FROM meta WHERE key = 'session';"
```
