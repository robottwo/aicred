# GenAI KeyFinder FFI (Foreign Function Interface)

This crate provides a C-compatible API for the genai-keyfinder core library, enabling bindings for Python, Go, and other languages through a stable C ABI.

## Overview

The FFI layer exposes the core functionality of genai-keyfinder through a stable C API that can be used to create language bindings. All functions are designed to be safe to call from C code and handle memory management correctly.

## API Functions

### Core Functions

#### `keyfinder_scan`
```c
char* keyfinder_scan(const char* home_path, const char* options_json);
```

Scan for GenAI credentials and configurations in the specified home directory.

**Parameters:**
- `home_path`: UTF-8 encoded home directory path (null-terminated C string)
- `options_json`: UTF-8 encoded JSON options (null-terminated C string)

**Returns:**
- UTF-8 encoded JSON string containing scan results. Caller must free with `keyfinder_free()`.
- Returns `NULL` on error.

**Example options JSON:**
```json
{
  "include_full_values": false,
  "max_file_size": 1048576,
  "only_providers": ["openai", "anthropic"],
  "exclude_providers": []
}
```

#### `keyfinder_free`
```c
void keyfinder_free(char* ptr);
```

Free a string returned by `keyfinder_scan`.

**Parameters:**
- `ptr`: Pointer to string allocated by the library (can be `NULL`)

#### `keyfinder_version`
```c
const char* keyfinder_version();
```

Get the library version string.

**Returns:**
- Static version string that does not need to be freed.

#### `keyfinder_last_error`
```c
const char* keyfinder_last_error();
```

Get the last error message (thread-local).

**Returns:**
- Pointer to the last error message, or `NULL` if no error occurred.
- The returned pointer is valid until the next call to any keyfinder function.

## Usage Example

### C Example

```c
#include "genai_keyfinder.h"
#include <stdio.h>
#include <stdlib.h>

int main() {
    const char* home = "/Users/username";
    const char* options = "{\"include_full_values\": false}";
    
    char* result = keyfinder_scan(home, options);
    if (result == NULL) {
        const char* error = keyfinder_last_error();
        fprintf(stderr, "Error: %s\n", error ? error : "Unknown error");
        return 1;
    }
    
    printf("Result: %s\n", result);
    keyfinder_free(result);
    
    return 0;
}
```

### Compilation

#### Linux/macOS
```bash
gcc -o myapp myapp.c -L/path/to/lib -lgenai_keyfinder_ffi -lpthread -ldl
```

#### Windows
```cmd
cl /I"path\to\include" myapp.c /link path\to\genai_keyfinder_ffi.dll.lib
```

## Memory Management

- **Caller Responsibility**: Strings returned by `keyfinder_scan` must be freed by the caller using `keyfinder_free`.
- **Thread Safety**: Error messages are stored in thread-local storage.
- **Null Safety**: All functions handle null pointers gracefully.

## Error Handling

The library uses thread-local storage for error messages. If a function fails:

1. The function returns `NULL` or an appropriate error value
2. The error message can be retrieved using `keyfinder_last_error()`
3. The error message remains available until the next API call

## Platform Support

The FFI layer supports the following platforms:

- **Linux**: Builds `.so` shared library
- **macOS**: Builds `.dylib` dynamic library  
- **Windows**: Builds `.dll` with `.lib` import library

## Building

### Prerequisites

- Rust toolchain (1.70+)
- C compiler (gcc, clang, or MSVC)
- cbindgen (for header generation)

### Build Commands

```bash
# Build the FFI library
cargo build --release

# Generate C header (automatically done during build)
# Header will be in ffi/include/genai_keyfinder.h

# Run tests
cargo test
```

### Cross-compilation

For cross-compilation, you may need to specify the target:

```bash
# For Linux x86_64
cargo build --target x86_64-unknown-linux-gnu --release

# For macOS x86_64
cargo build --target x86_64-apple-darwin --release

# For Windows x86_64
cargo build --target x86_64-pc-windows-msvc --release
```

## JSON Output Format

The `keyfinder_scan` function returns a JSON string with the following structure:

```json
{
  "keys": [
    {
      "provider": "openai",
      "key_type": "api_key",
      "confidence": "high",
      "value": "sk-...",
      "source_file": "/home/user/.env",
      "line_number": 5
    }
  ],
  "config_instances": [
    {
      "name": "my-app",
      "type": "application",
      "path": "/home/user/.config/my-app",
      "keys": []
    }
  ],
  "scan_summary": {
    "total_keys": 1,
    "total_config_instances": 1,
    "files_scanned": 10,
    "directories_scanned": 3,
    "scan_duration": 0.123
  }
}
```

## Safety

All FFI functions are designed to be safe:

- **Null Pointer Checks**: All pointers are checked for null before dereferencing
- **Panic Safety**: Uses `std::panic::catch_unwind` to prevent panics from propagating to C
- **UTF-8 Validation**: All string inputs are validated for UTF-8 encoding
- **Memory Safety**: Proper memory management with no leaks

## Thread Safety

The library is thread-safe:

- Error messages are stored in thread-local storage
- No global mutable state
- All functions can be called concurrently from multiple threads

## License

This project is licensed under the same terms as the main genai-keyfinder project (MIT OR Apache-2.0).