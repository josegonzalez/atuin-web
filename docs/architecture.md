# Architecture

## System Diagram

```
Browser <--HTTP--> [atuin-web (Axum 0.8)] <--API client--> [atuin server]
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
6. htmx handles partial page updates (e.g., records pagination via `hx-get` / `hx-target` / `hx-push-url`); `decrypt.js` listens on `htmx:afterSwap` to re-process encrypted elements

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

## Record Types

The web UI supports five record types, selected via the `?tag=` query parameter on `/records`:

| Tag | Label | Description |
|-----|-------|-------------|
| `history` | History | Shell command history |
| `kv` | Key-Value | Key-value store entries |
| `config-shell-alias` | Aliases | Shell alias definitions |
| `dotfiles-var` | Variables | Environment variables |
| `script` | Scripts | Saved scripts |

Each type has a tag-specific MessagePack decoder in `decrypt.js` that formats the decrypted data for display.

## Client-Side JavaScript

- `paseto-v4.js` — PASETO V4 local decryption + PASERK PIE key unwrapping
- `blake2b.min.js` — BLAKE2b hashing (used by PASETO key derivation)
- `msgpack.min.js` — MessagePack decoding for decrypted record payloads
- `bip39.min.js` — BIP39 mnemonic word list for key input
- `decrypt.js` — Key management (sessionStorage), decryption orchestration, click-to-copy, tag-specific MessagePack decoders for history/kv/alias/var/script records
- `theme.js` — Dark/light theme toggle
- `htmx.min.js` — Partial page updates; used for records pagination via `hx-*` attributes
