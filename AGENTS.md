# Repository Guidelines

This is a Rust workspace implementing the Soulseek protocol (Slsk), containing crates for protocol handling, core networking, GUI, and testing utilities.

## Project Structure

```
crates/
├── slsk-proto/      # Protocol message definitions and codec
├── slsk-core/       # Core networking and state management
├── slsk-gui/        # GTK-based GUI application
└── slsk-mock/       # Mock utilities for testing
doc/
└── SLSKPROTOCOL.md  # Protocol specification (updated via `just update-spec`)
```

## Build, Test, and Development Commands

| Command | Description |
|---------|-------------|
| `just test` | Run all workspace tests |
| `just test-proto` | Run protocol crate tests only |
| `just run` | Build and run the GUI application |
| `just lint` | Run clippy with strict warnings (`-D warnings`) |
| `just fmt` | Format all crates with cargo fmt |
| `just update-spec` | Fetch latest protocol spec from nicotine-plus |

## Coding Style

- Rust 2021 edition, standard cargo workspace layout
- Run `just fmt` before committing; linting is enforced in CI
- Use `thiserror` for error handling, `tracing` for logging
- Feature flags: use `#[cfg(feature = "...")]` for optional functionality

## Testing Guidelines

- Tests live alongside source files using `#[cfg(test)]` modules
- Run `just test-proto` for protocol-level unit tests
- Mock utilities in `slsk-mock` support integration testing

## Commit & Pull Request Guidelines

- Prefix commits with scope: `proto: ...`, `core: ...`, `gui: ...`
- PRs should describe the change and any protocol implications
- Link related issues and include test coverage for new functionality
