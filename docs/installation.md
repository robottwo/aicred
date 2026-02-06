# Installation Guide

## Prerequisites

- **Rust 1.70 or later** (for building from source)
- **C compiler** (gcc, clang, or MSVC)
- **Python 3.8+** (optional, for Python bindings)
- **Go 1.21+** (optional, for Go bindings)
- **Node.js 18+** (optional, for GUI)

## CLI Installation

### From Source (Recommended for v0.2.0 Beta)

```bash
# Clone the repository
git clone https://github.com/robottwo/aicred
cd aicred

# Build the CLI
cargo build --release

# The binary will be at target/release/aicred
# On Unix-like systems, you can install to /usr/local/bin:
sudo cp target/release/aicred /usr/local/bin/

# Verify installation
aicred --version
```

### From Crates.io (Coming Soon)

Official crates.io publication will be available with the v0.2.0 stable release:

```bash
# Coming soon:
cargo install aicred
```

### Platform-Specific Packages (Coming Soon)

#### macOS (Homebrew)

```bash
# Coming soon:
brew tap robottwo/aicred
brew install aicred
```

#### Linux

```bash
# Coming soon - download from releases:
curl -LO https://github.com/robottwo/aicred/releases/latest/download/aicred-linux-x86_64.tar.gz
tar xzf aicred-linux-x86_64.tar.gz
sudo mv aicred /usr/local/bin/
```

#### Windows (Scoop)

```powershell
# Coming soon:
scoop bucket add aicred https://github.com/robottwo/scoop-aicred
scoop install aicred
```

## Python Bindings

### From Source (Current)

```bash
# From the aicred repository root:
cd bindings/python

# Install maturin (build tool for Python+Rust)
pip install maturin

# Build and install in development mode
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop

# Or build a wheel for distribution
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin build --release
```

**Note:** The `PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1` environment variable is required for Python 3.13+ compatibility.

### From PyPI (Coming Soon)

Official PyPI publication will be available with the v0.2.0 stable release:

```bash
# Coming soon:
pip install aicred
```

## Go Bindings

### From Source (Current)

```bash
# Clone the repository if you haven't already
git clone https://github.com/robottwo/aicred
cd aicred

# The Go bindings use CGO to link against the Rust core
# First, build the C library:
cargo build --release -p aicred-ffi

# Then you can use the Go bindings:
cd bindings/go
go build ./...

# Run the example:
cd examples/basic_usage
go run .
```

### Go Module (Coming Soon)

Official Go module publication will be available with the v0.2.0 stable release:

```bash
# Coming soon:
go get github.com/robottwo/aicred/bindings/go
```

## GUI Application

### Build from Source (Current)

```bash
cd gui
npm install
npm run tauri build

# The built application will be in src-tauri/target/release/bundle/
```

### Download Installers (Coming Soon)

Platform-specific installers will be available from [releases](https://github.com/robottwo/aicred/releases) with the v0.2.0 stable release:

- **macOS**: `.dmg` file
- **Windows**: `.msi` installer
- **Linux**: `.AppImage` or `.deb` package

## Rust Library

### From Source (Current)

Add to your `Cargo.toml`:

```toml
[dependencies]
aicred-core = { git = "https://github.com/robottwo/aicred", branch = "main" }
```

### From Crates.io (Coming Soon)

```toml
[dependencies]
aicred-core = "0.2.0"
```

## Verification

Verify your installation:

```bash
# CLI
aicred --version

# Python
python -c "import aicred; print(aicred.__version__)"

# Go (run the provided example)
cd bindings/go/examples/basic_usage && go run .
```