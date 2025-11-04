# AICred - Comprehensive Build System
# This Makefile provides logical top-level build targets for the multi-component project

# Platform detection
UNAME_S := $(shell uname -s 2>/dev/null || echo "Windows")
ifeq ($(UNAME_S),Linux)
	PLATFORM := linux
	LIB_EXT := so
endif
ifeq ($(UNAME_S),Darwin)
	PLATFORM := macos
	LIB_EXT := dylib
endif
ifeq ($(OS),Windows_NT)
	PLATFORM := windows
	LIB_EXT := dll
	EXE_EXT := .exe
endif

# Default target
.PHONY: all
all: build-all

# Core build targets
.PHONY: build-all
build-all: build-core build-cli build-ffi build-gui build-python build-go

# Build core components without Python bindings (useful when maturin is not available)
.PHONY: build-core-only
build-core-only: build-core build-cli build-ffi build-gui build-go

.PHONY: build-core
build-core:
	cargo build --release -p aicred-core

.PHONY: build-cli
build-cli: build-core
	cargo build --release -p aicred

.PHONY: build-ffi
build-ffi: build-core
	cargo build --release -p aicred-ffi

.PHONY: build-gui
build-gui: build-core build-gui-frontend
	cargo build --release -p aicred-gui

.PHONY: build-python
build-python: build-core
	@command -v maturin >/dev/null 2>&1 || { echo "Error: maturin not found. Install with: pip install maturin"; exit 1; }
	@echo "Building Python bindings with PyO3 ABI3 forward compatibility..."
	@cd bindings/python && PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin build --release || { \
		echo "Warning: Python bindings build failed. This may be due to Python version incompatibility."; \
		echo "PyO3 currently supports Python 3.7-3.13. You have: $$(python3 --version 2>/dev/null || echo 'unknown')"; \
		echo "PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 is already set to handle Python 3.13.9+ compatibility."; \
		echo "If build continues to fail, try using Python 3.13.8 or earlier, or check PyO3 documentation."; \
		exit 1; \
	}

.PHONY: build-go
build-go: build-ffi
	cd bindings/go && $(MAKE) build

.PHONY: build-gui-frontend
build-gui-frontend:
	cd gui && npm install && npm run build

# Development targets
.PHONY: dev-setup
dev-setup: deps
	@echo "Setting up development environment..."
	@command -v rustup >/dev/null 2>&1 && { echo "Installing Rust components..."; rustup component add clippy rustfmt; } || \
		{ echo "Note: rustup not found. Rust components (clippy, rustfmt) may need manual installation."; }
	cd gui && npm install
	@command -v maturin >/dev/null 2>&1 || { \
		echo "Installing maturin for Python bindings..."; \
		(pip3 install --user maturin 2>/dev/null) || \
		(pip3 install maturin --break-system-packages 2>/dev/null) || \
		(echo "Failed to install maturin automatically. Please install manually:"; \
		 echo "  brew install maturin  # macOS with Homebrew"; \
		 echo "  pip install maturin  # In a virtual environment"; \
		 echo "  cargo install maturin # Using cargo directly"); \
	}
	@command -v pytest >/dev/null 2>&1 || { \
		echo "Installing pytest for Python tests..."; \
		(pip3 install --user pytest 2>/dev/null) || \
		(pip3 install pytest --break-system-packages 2>/dev/null) || \
		(echo "Failed to install pytest automatically. Please install manually:"; \
		 echo "  brew install pytest  # macOS with Homebrew"; \
		 echo "  pip install pytest  # In a virtual environment"); \
	}
	@command -v cargo-watch >/dev/null 2>&1 || { echo "Installing cargo-watch..."; cargo install cargo-watch; }
	@echo "Development environment setup complete!"

# Check for required tools
.PHONY: check-deps
check-deps:
	@echo "Checking dependencies..."
	@command -v rustc >/dev/null 2>&1 || { echo "Error: Rust not found. Install from https://rustup.rs/"; exit 1; }
	@command -v npm >/dev/null 2>&1 || { echo "Error: Node.js/npm not found. Install from https://nodejs.org/"; exit 1; }
	@command -v go >/dev/null 2>&1 || { echo "Error: Go not found. Install from https://golang.org/"; exit 1; }
	@command -v python3 >/dev/null 2>&1 || { echo "Error: Python3 not found. Install Python 3.x"; exit 1; }
	@echo "All basic dependencies found!"

# Check for optional tools
.PHONY: check-optional
check-optional:
	@echo "Checking optional tools..."
	@command -v maturin >/dev/null 2>&1 && echo "✓ maturin found" || echo "✗ maturin missing (needed for Python bindings)"
	@(command -v cargo-watch >/dev/null 2>&1 || [ -f ~/.cargo/bin/cargo-watch ]) && echo "✓ cargo-watch found" || echo "✗ cargo-watch missing (needed for watch target)"
	@command -v pytest >/dev/null 2>&1 && echo "✓ pytest found" || echo "✗ pytest missing (needed for Python tests)"
	@echo "Run 'make dev-setup' to install missing optional tools"

.PHONY: deps
deps:
	cargo fetch
	cd gui && npm install

.PHONY: watch
watch:
	cargo watch -x check -x test -x clippy

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: fmt-check
fmt-check:
	cargo fmt --all --check

.PHONY: clippy
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: check
check:
	cargo check --all

# Testing targets
# Note: On macOS, FFI tests are excluded from test-integration due to SIGTRAP issue
.PHONY: test
test: test-unit test-integration test-python test-go

.PHONY: test-unit
test-unit:
	cargo test --lib

.PHONY: test-integration
test-integration:
ifeq ($(PLATFORM),macos)
	@echo "Running integration tests (excluding FFI on macOS due to SIGTRAP issue)..."
	cargo test --workspace --exclude aicred-ffi --test '*'
else
	cargo test --test '*'
endif

.PHONY: test-python
test-python: build-python
	@command -v pytest >/dev/null 2>&1 || { echo "Error: pytest not found. Run 'make dev-setup' to install it."; exit 1; }
	@echo "Installing Python wheel..."
	@WHEEL=$$(ls target/wheels/aicred-*.whl 2>/dev/null | head -1); \
	if [ -z "$$WHEEL" ]; then echo "Error: Python wheel not found. Run 'make build-python' first."; exit 1; fi; \
	echo "Installing $$WHEEL..."; \
	python3 -m pip install --break-system-packages --user --force-reinstall "$$WHEEL" || { \
		echo "Failed to install Python wheel. Make sure python3 and pip are available and the wheel is built."; \
		exit 1; \
	}
	cd bindings/python && python3 -m pytest tests/

.PHONY: test-go
test-go: build-go
	cd bindings/go && $(MAKE) test

.PHONY: test-all
test-all: test test-bench

.PHONY: test-bench
test-bench:
	cargo bench

# Cleaning targets
.PHONY: clean
clean:
	cargo clean
	rm -rf gui/node_modules
	rm -rf bindings/python/target
	cd bindings/go && $(MAKE) clean

.PHONY: clean-all
clean-all: clean
	rm -rf dist/
	rm -rf gui/dist/
	rm -rf bindings/python/*.egg-info

# Packaging targets
.PHONY: package-all
package-all: package-linux package-macos package-windows

.PHONY: package-linux
package-linux: build-all
	mkdir -p dist/linux
	cp target/release/aicred dist/linux/
	cp target/release/libaicred_ffi.so dist/linux/
	cp -r conf dist/linux/

.PHONY: package-macos
package-macos: build-all
	mkdir -p dist/macos
	cp target/release/aicred dist/macos/
	cp target/release/libaicred_ffi.dylib dist/macos/
	cp -r conf dist/macos/

.PHONY: package-windows
package-windows: build-all
	mkdir -p dist/windows
	cp target/release/aicred.exe dist/windows/
	cp target/release/aicred_ffi.dll dist/windows/
	xcopy /E /I conf dist\windows\conf
# Platform-specific targets
.PHONY: build-platform

# Map PLATFORM values to existing build targets
ifeq ($(PLATFORM),linux)
BUILD_TARGET := build-all
else ifeq ($(PLATFORM),macos)
BUILD_TARGET := build-all
else ifeq ($(PLATFORM),windows)
BUILD_TARGET := build-all
else
BUILD_TARGET := build-all
endif

build-platform: $(BUILD_TARGET)

.PHONY: package-platform
package-platform: package-$(PLATFORM)

.PHONY: install-platform
install-platform: package-platform
ifeq ($(PLATFORM),windows)
	@echo "Manual installation required on Windows"
else
	cp dist/$(PLATFORM)/aicred /usr/local/bin/
endif

# Package manager targets
.PHONY: package-homebrew
package-homebrew:
	cd packaging/homebrew && $(MAKE)

.PHONY: package-scoop
package-scoop:
	cd packaging/scoop && $(MAKE)

.PHONY: package-chocolatey
package-chocolatey:
	cd packaging/chocolatey && $(MAKE)

# Documentation targets
.PHONY: docs
docs:
	cargo doc --no-deps --open

.PHONY: docs-all
docs-all:
	cargo doc --all-features --no-deps

# Release targets
.PHONY: release
release: clean-all test-all package-all

.PHONY: release-patch
release-patch:
	cargo release patch

.PHONY: release-minor
release-minor:
	cargo release minor

.PHONY: release-major
release-major:
	cargo release major

# Utility targets
.PHONY: version
version:
	@echo "AICred Build System"
	@echo "Platform: $(PLATFORM)"
	@echo "Library Extension: $(LIB_EXT)"
	@echo "Executable Extension: $(EXE_EXT)"

.PHONY: info
info:
	@echo "Rust toolchain:"
	@rustc --version
	@echo "Node.js version:"
	@node --version
	@echo "Go version:"
	@go version
	@echo "Python version:"
	@python3 --version

# Help system
.PHONY: help
help:
	@echo "AICred Build System"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Core build targets:"
	@echo "  build-all      - Build all components (core, cli, ffi, gui, python, go)"
	@echo "  build-core-only - Build core components without Python bindings"
	@echo "  build-core     - Build core library only"
	@echo "  build-cli      - Build CLI tool"
	@echo "  build-ffi      - Build FFI library"
	@echo "  build-gui      - Build GUI application"
	@echo "  build-python   - Build Python bindings (requires maturin)"
	@echo "  build-go       - Build Go bindings"
	@echo ""
	@echo "Development targets:"
	@echo "  dev-setup      - Setup development environment"
	@echo "  check-deps     - Check required dependencies"
	@echo "  check-optional - Check optional tools"
	@echo "  watch          - Watch for changes and rebuild"
	@echo "  fmt            - Format all code"
	@echo "  fmt-check      - Check code formatting without making changes"
	@echo "  clippy         - Run clippy linter"
	@echo "  check          - Check code without building"
	@echo ""
	@echo "Testing targets:"
	@echo "  test           - Run unit and integration tests"
	@echo "  test-unit      - Run unit tests only"
	@echo "  test-integration - Run integration tests only"
	@echo "  test-python    - Test Python bindings"
	@echo "  test-go        - Test Go bindings"
	@echo "  test-all       - Run all tests including benchmarks"
	@echo ""
	@echo "Cleaning targets:"
	@echo "  clean          - Clean build artifacts"
	@echo "  clean-all      - Clean everything including dist"
	@echo ""
	@echo "Packaging targets:"
	@echo "  package-all    - Create packages for all platforms"
	@echo "  package-linux  - Create Linux package"
	@echo "  package-macos  - Create macOS package"
	@echo "  package-windows - Create Windows package"
	@echo "  package-platform - Create package for current platform"
	@echo ""
	@echo "Platform targets:"
	@echo "  build-platform - Build for current platform"
	@echo "  install-platform - Install to system (Linux/macOS)"
	@echo ""
	@echo "Package manager targets:"
	@echo "  package-homebrew - Create Homebrew package"
	@echo "  package-scoop   - Create Scoop package"
	@echo "  package-chocolatey - Create Chocolatey package"
	@echo ""
	@echo "Documentation targets:"
	@echo "  docs           - Generate and open documentation"
	@echo "  docs-all       - Generate all documentation"
	@echo ""
	@echo "Release targets:"
	@echo "  release        - Full release build (clean, test, package)"
	@echo "  release-patch  - Release patch version"
	@echo "  release-minor  - Release minor version"
	@echo "  release-major  - Release major version"
	@echo ""
	@echo "Utility targets:"
	@echo "  version        - Show version information"
	@echo "  info           - Show toolchain information"
	@echo "  help           - Show this help message"
	@echo ""
	@echo "Default target: help"

# Default help when no target specified
.DEFAULT_GOAL := help