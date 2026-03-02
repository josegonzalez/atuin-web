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

Use `cargo-watch` to automatically rebuild and restart the server on code changes. Templates already hot-reload from disk in debug mode without a restart.

### Prerequisites

```bash
cargo install cargo-watch gaffa
```

### Usage

Start the dev environment with [gaffa](https://github.com/oryon-dominik/gaffa):

```bash
gaffa run --procfile Procfile.dev --env-file .env
```

Or run directly:

```bash
cargo watch -x 'run -p atuin-web'
```

When you save a Rust file, `cargo-watch` rebuilds and restarts the binary. Template changes (`templates/`) are picked up on the next request without a restart.

## Test

```bash
cargo test
```

## Release Build

```bash
cargo build --release
```

The release binary embeds all assets and templates — no external files needed.
