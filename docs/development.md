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

## Test

```bash
cargo test
```

## Release Build

```bash
cargo build --release
```

The release binary embeds all assets and templates — no external files needed.
