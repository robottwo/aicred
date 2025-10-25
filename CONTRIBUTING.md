# Contributing to GenAI Key Finder

Thank you for your interest in contributing!

## Development Setup

```bash
# Clone repository
git clone https://github.com/yourusername/genai-keyfinder
cd genai-keyfinder

# Build all components
cargo build --all

# Run tests
cargo test --all

# Build CLI
cargo build --release -p genai-keyfinder
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Add tests for new features
- Update documentation

## Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Update documentation
6. Submit pull request

## Adding a New Provider

1. Create plugin in `core/src/providers/`
2. Implement `ProviderPlugin` trait
3. Add tests with fixtures
4. Update provider list in documentation
5. Register in `core/src/plugins/mod.rs`

## Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test '*'

# Specific component
cargo test -p genai-keyfinder-core
```

## Documentation

Update relevant documentation in `docs/` when making changes.