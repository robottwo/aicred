# AICred - Go Bindings

Go bindings for the aicred library using CGo.

## Prerequisites

- Go 1.21 or later
- Rust toolchain (for building the FFI library)
- C compiler (gcc, clang, or MSVC)

## Installation

```bash
go get github.com/robottwo/aicred/bindings/go/aicred
```

## Building from Source

```bash
# Build the FFI library
cd ../../ffi
cargo build --release

# Build Go bindings
cd ../bindings/go
go build ./aicred
```

## Usage

```go
package main

import (
    "fmt"
    "log"
    
    aicred "github.com/robottwo/aicred/bindings/go/aicred"
)

func main() {
    // Scan for credentials
    result, err := aicred.Scan(aicred.ScanOptions{})
    if err != nil {
        log.Fatal(err)
    }
    
    // Print results
    fmt.Printf("Found %d keys\n", len(result.Keys))
    for _, key := range result.Keys {
        fmt.Printf("%s: %s\n", key.Provider, key.Redacted)
    }
}
```

## API Reference

### Types

#### `ScanOptions`
Configuration options for scanning.

**Fields:**
- `HomeDir` (string): Home directory to scan
- `IncludeFullValues` (bool): Include full secret values (DANGEROUS)
- `MaxFileSize` (int): Maximum file size in bytes
- `OnlyProviders` ([]string): Only scan these providers
- `ExcludeProviders` ([]string): Exclude these providers

#### `ScanResult`
Results of a scan operation.

**Fields:**
- `Keys` ([]DiscoveredKey): Discovered API keys
- `ConfigInstances` ([]ConfigInstance): Application config instances
- `HomeDir` (string): Scanned home directory
- `ScannedAt` (string): Timestamp of scan
- `ProvidersScanned` ([]string): List of providers scanned

### Functions

#### `Scan(options ScanOptions) (*ScanResult, error)`
Scan for GenAI credentials and configurations.

#### `Version() string`
Get library version.

#### `ListProviders() []string`
List available provider plugins.

#### `ListScanners() []string`
List available application scanners.

## Testing

```bash
make test
```

## Examples

```bash
make example
```

## Security

By default, all secrets are redacted. Only use `IncludeFullValues: true` in secure environments.

## Platform Support

- Linux (x86_64, aarch64)
- macOS (x86_64, aarch64/Apple Silicon)
- Windows (x86_64)