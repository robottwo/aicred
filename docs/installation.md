# Installation Guide

## Prerequisites

- Rust 1.70 or later (for building from source)
- C compiler (gcc, clang, or MSVC)
- Python 3.8+ (for Python bindings)
- Go 1.21+ (for Go bindings)
- Node.js 18+ (for GUI)

## CLI Installation

### From Crates.io

```bash
cargo install genai-keyfinder
```

### From Source

```bash
git clone https://github.com/robottwo/aicred
cd aicred
cargo build --release -p genai-keyfinder
```

### Platform-Specific

#### macOS (Homebrew)

```bash
brew tap robottwo/aicred
brew install genai-keyfinder
```

#### Linux

```bash
# Download latest release
curl -LO https://github.com/robottwo/aicred/releases/latest/download/keyfinder-linux-x86_64.tar.gz
tar xzf keyfinder-linux-x86_64.tar.gz
sudo mv keyfinder /usr/local/bin/
```

#### Windows (Scoop)

```powershell
scoop bucket add genai-keyfinder https://github.com/robottwo/scoop-aicred
scoop install genai-keyfinder
```

## Python Bindings

### From PyPI

```bash
pip install genai-keyfinder
```

### From Source

```bash
cd bindings/python
pip install maturin
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop
```

**Note:** The `PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1` environment variable is required for Python 3.13.9+ compatibility.

## Go Bindings

```bash
go get github.com/robottwo/aicred/bindings/go/genai_keyfinder
```

## GUI Application

### Download Installers

Download platform-specific installers from [releases](https://github.com/robottwo/aicred/releases):

- **macOS**: `.dmg` file
- **Windows**: `.msi` installer
- **Linux**: `.AppImage` or `.deb` package

### Build from Source

```bash
cd gui
npm install
npm run tauri build
```

## Rust Library

Add to your `Cargo.toml`:

```toml
[dependencies]
genai-keyfinder-core = "0.1.0"
```

## Verification

Verify installation:

```bash
# CLI
keyfinder version

# Python
python -c "import genai_keyfinder; print(genai_keyfinder.version())"

# Go (run the provided example)
cd bindings/go/examples/basic_usage && go run .