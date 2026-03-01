# atuin-web

A web UI for [atuin](https://github.com/atuinsh/atuin), providing a browser-based interface to browse and search your shell history.

## Quick Start

```bash
cargo build
cargo run -- --atuin-server-url http://localhost:8888
```

## Documentation

See [docs/](docs/) for full documentation:

- [Architecture](docs/architecture.md) - System diagram, data flow, security model
- [Configuration](docs/configuration.md) - All config options, example .env
- [Development](docs/development.md) - Build, test, dev server setup
- [Deployment](docs/deployment.md) - Docker, systemd, reverse proxy
- [API Mapping](docs/api-mapping.md) - Web UI route to atuin API endpoint mapping

## License

MIT - see [LICENSE](LICENSE) for details.
