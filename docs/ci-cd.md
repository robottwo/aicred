# CI/CD Pipeline

## Overview

The project uses GitHub Actions for continuous integration and deployment across multiple platforms.

## Workflows

### Main CI (`ci.yml`)
- Runs on: Push to main/develop, Pull Requests
- Tests on: Linux, macOS, Windows
- Rust versions: stable, beta
- Includes: tests, rustfmt, clippy

### Python Bindings (`python.yml`)
- Tests Python 3.8-3.12
- Builds wheels for all platforms
- Uses maturin for building

### Go Bindings (`go.yml`)
- Tests Go 1.21, 1.22
- Builds FFI library
- Runs Go tests and examples

### GUI Application (`gui.yml`)
- Builds Tauri app for all platforms
- Creates installers (.deb, .dmg, .msi)

### Release (`release.yml`)
- Triggered by version tags (v*)
- Creates GitHub releases
- Uploads platform-specific binaries
- Publishes to PyPI

### Security (`security.yml`)
- Weekly security audits
- Dependency review on PRs

### Coverage (`coverage.yml`)
- Generates code coverage reports
- Uploads to codecov.io

### Documentation (`docs.yml`)
- Builds and deploys Rust docs
- Publishes to GitHub Pages

## Local Testing

Run CI checks locally:

```bash
# Format check
cargo fmt --all -- --check

# Clippy
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test --all

# Python tests
cd bindings/python && maturin develop && pytest

# Go tests
cd bindings/go && go test ./...
```

## Release Process

1. Update version in all `Cargo.toml` files
2. Update `CHANGELOG.md`
3. Commit changes
4. Create and push tag: `git tag v0.1.0 && git push origin v0.1.0`
5. GitHub Actions will automatically create release and publish packages