# Architecture

## System Diagram

```
Browser <--HTTP--> [atuin-web (Axum 0.8)] <--HTTP proxy--> [atuin server]
                   |
                   +-- MiniJinja templates (disk in dev, embedded in release)
                   +-- Bootstrap 5.3 CSS/JS (embedded via rust-embed)
                   +-- htmx 2.0 JS (embedded via rust-embed)
                   +-- tower-sessions (cookie + in-memory store)
```

## Data Flow

1. Browser requests a page (e.g., `/records`)
2. Auth middleware checks for valid session or config token
3. Route handler calls atuin server API via `AtuinClient`
4. Response data is rendered into MiniJinja templates
5. HTML is returned to the browser
6. htmx handles partial page updates for filters and polling

## Security Model

- **Read-only**: Only proxies GET endpoints + POST `/login` for auth
- **End-to-end encryption**: History and record data is encrypted server-side
- **Client-side decryption**: Optional — encryption key is stored only in browser `sessionStorage`
- **Key never sent to server**: All decryption happens in JavaScript
- **Session management**: `HttpOnly` + `SameSite=Lax` cookies with server-side memory store

## Project Structure

The project uses a Cargo workspace with two crates:

- `crates/atuin-web/` — Binary crate (server startup, session layer)
- `crates/atuin-web-lib/` — Library crate (routes, templates, client, auth)

Static assets and templates are embedded into the release binary via `rust-embed`.
