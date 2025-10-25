#!/bin/bash
set -e

VERSION="0.1.0"
BINARY_NAME="keyfinder"

echo "Downloading genai-keyfinder v${VERSION}..."
curl -LO "https://github.com/yourusername/genai-keyfinder/releases/download/v${VERSION}/${BINARY_NAME}-macos-universal.tar.gz"

echo "Extracting..."
tar xzf "${BINARY_NAME}-macos-universal.tar.gz"

echo "Installing to /usr/local/bin..."
sudo mv "${BINARY_NAME}" /usr/local/bin/

echo "Cleaning up..."
rm "${BINARY_NAME}-macos-universal.tar.gz"

echo "Installation complete!"
echo "Run 'keyfinder --help' to get started"