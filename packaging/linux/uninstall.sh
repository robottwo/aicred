#!/bin/bash
set -e

echo "Removing genai-keyfinder from /usr/local/bin..."
sudo rm -f /usr/local/bin/keyfinder

echo "Uninstallation complete!"