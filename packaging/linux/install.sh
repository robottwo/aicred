#!/bin/bash
set -e

VERSION="0.1.0"
TARGET="x86_64-unknown-linux-musl"
BINARY_NAME="keyfinder"

echo "Downloading genai-keyfinder v${VERSION}..."
curl -LO "https://github.com/yourusername/genai-keyfinder/releases/download/v${VERSION}/${BINARY_NAME}-linux-${TARGET}.tar.gz"

echo "Extracting..."
tar xzf "${BINARY_NAME}-linux-${TARGET}.tar.gz"

echo "Installing to /usr/local/bin..."
sudo mv "${BINARY_NAME}" /usr/local/bin/

echo "Cleaning up..."
rm "${BINARY_NAME}-linux-${TARGET}.tar.gz"

echo "Installation complete!"
echo "Run 'keyfinder --help' to get started"