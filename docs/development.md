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

## Auto-Restart Dev Server

Use `watchexec` to automatically rebuild and restart the server on code changes. Templates already hot-reload from disk in debug mode without a restart.

### Prerequisites

```bash
cargo install watchexec-cli
```

### Usage

```bash
./bin/dev
```

This loads `.env`, then `exec`s `watchexec` so it is the direct foreground process. Ctrl+C cleanly terminates watchexec and all child processes. Pass CLI args to override the default bind address:

```bash
./bin/dev --bind 127.0.0.1:3000
```

When you save a `.rs` or `.toml` file, `watchexec` kills the running process and rebuilds. Template changes (`templates/`) are picked up on the next request without a restart.

> **Note:** `Procfile.dev` is kept for Procfile-compatible runners, but gaffa does not forward signals to child processes on Ctrl+C ([gaffa#signal-handling](https://github.com/oryon-dominik/gaffa)), leaving orphaned listeners on the port. Prefer `bin/dev` instead.

## Test

```bash
cargo test
```

## Release Build

```bash
cargo build --release
```

The release binary embeds all assets and templates — no external files needed.
