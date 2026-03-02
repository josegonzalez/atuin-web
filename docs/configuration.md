# Configuration

All options can be set via CLI args or environment variables. CLI args take precedence.

| Option | Env Var | Default | Description |
|--------|---------|---------|-------------|
| `--bind` | `ATUIN_WEB_BIND` | `127.0.0.1:8080` | Bind address |
| `--atuin-server-url` | `ATUIN_WEB_SERVER_URL` | `http://127.0.0.1:8888` | Upstream atuin server |
| `--token` | `ATUIN_WEB_TOKEN` | (none) | Pre-configured auth token |
| `--session-expiry` | `ATUIN_WEB_SESSION_EXPIRY` | `86400` | Session TTL (seconds) |
| `--log-level` | `ATUIN_WEB_LOG_LEVEL` | `info` | Log level |
| `--secure-cookies` | `ATUIN_WEB_SECURE_COOKIES` | `false` | Set Secure flag on cookies (enable behind HTTPS) |

## Example .env

```env
ATUIN_WEB_BIND=0.0.0.0:8080
ATUIN_WEB_SERVER_URL=http://localhost:8888
ATUIN_WEB_TOKEN=your-session-token
ATUIN_WEB_LOG_LEVEL=info
```

## Upstream Request Timeouts

All HTTP requests to the upstream atuin server have a 30-second request timeout and a 10-second connection timeout. These are not currently configurable.

## Getting Your Auth Token

```bash
sqlite3 ~/.local/share/atuin/meta.db "SELECT value FROM meta WHERE key = 'session';"
```
