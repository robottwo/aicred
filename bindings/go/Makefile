.PHONY: build test clean example

# Build the FFI library first
build-ffi:
	cd ../../ffi && cargo build --release

# Build Go bindings
build: build-ffi
	go build ./genai_keyfinder

# Run tests
test: build-ffi
	go test -v ./genai_keyfinder

# Run example
example: build-ffi
	cd examples/basic_usage && go run main.go

# Clean build artifacts
clean:
	go clean
	rm -f examples/basic_usage/scan_result.json

# Install dependencies
deps:
	go mod download
	go mod tidy

# Run benchmarks
bench: build-ffi
	go test -bench=. ./genai_keyfinder