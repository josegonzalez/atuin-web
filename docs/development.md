# Development

## Prerequisites

- Rust 1.75+
- A running atuin server instance

## Build

```bash
cargo build
```

## Run (dev)

```bash
# With login form
cargo run -- --atuin-server-url http://localhost:8888

# With pre-configured token
cargo run -- --atuin-server-url http://localhost:8888 --token YOUR_TOKEN
```

Templates are loaded from disk in debug mode, so changes to `templates/` are reflected on refresh.

## Hot Reload

Hot-reloading lets you see Rust code changes (handlers, routes, middleware) take effect without manually restarting the server. Templates already hot-reload from disk in debug mode; this extends that to compiled code.

### Prerequisites

```bash
cargo install cargo-watch gaffa
```

### Usage

Start the hot-reload dev environment with [gaffa](https://github.com/oryon-dominik/gaffa):

```bash
gaffa run --procfile Procfile.dev
```

To override the atuin server URL:

```bash
gaffa run --procfile Procfile.dev --env ATUIN_SERVER_URL=http://localhost:9999
```

Or load environment variables from a `.env` file:

```bash
gaffa run --procfile Procfile.dev --env-file .env
```

Alternatively, run two terminals from the workspace root:

**Terminal 1** — rebuilds the dylib on source changes:

```bash
cargo watch -x 'build -p atuin-web-lib'
```

**Terminal 2** — runs the binary with hot-reload enabled:

```bash
cargo run -p atuin-web --features hot-reload -- --atuin-server-url http://localhost:8888
```

When you save a file in `crates/atuin-web-lib/`, cargo-watch rebuilds the dylib. The running binary detects the new library, gracefully shuts down, and rebuilds the router from the reloaded code. Sessions survive across reloads.

### What hot-reloads vs. what requires restart

| Change | Hot-reloads? |
|--------|-------------|
| Handler logic in `crates/atuin-web-lib/` | Yes |
| Route definitions | Yes |
| Auth middleware | Yes |
| Templates (`templates/`) | Yes (already, via disk reload) |
| Config / CLI args | No — restart required |
| Binary code (`crates/atuin-web/src/main.rs`) | No — restart required |

## Test

```bash
cargo test
```

## Release Build

```bash
cargo build --release
```

The release binary embeds all assets and templates — no external files needed.
