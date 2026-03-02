# API Mapping

Web UI routes and their corresponding atuin API endpoints.

| Web UI Route | Atuin API Endpoint(s) | Auth | Notes |
|---|---|---|---|
| `GET /healthz` | — | No | Health check endpoint; returns `{"status": "ok"}` |
| `GET /login` | — | No | Login form |
| `POST /login` | `POST /login` | No | Get session token |
| `POST /logout` | — | Yes | Clear session |
| `GET /` | `GET /healthz`, `GET /api/v0/me`, `GET /api/v0/record` | Yes | Dashboard: health, user info, record status (per-tag record counts summed across all hosts) |
| `GET /records` | — | Yes | Landing page when no `?tag=` is provided; shows links to each record type |
| `GET /records?tag=T` | `GET /api/v0/record`, `GET /api/v0/record/next?host=X&tag=T&start=S&count=N` | Yes | Paginated records for a specific tag. Accepts `?page=N&page_size=S` query params (page_size: 25, 50, or 100; default: page=1, page_size=25). Allowed tags: see [Record Types](architecture.md#record-types) |
| `GET /assets/{*path}` | — | No | Static assets (embedded) |
