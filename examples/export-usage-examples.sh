#!/bin/bash
# aicred Export Usage Examples
# This script demonstrates various ways to use the aicred export command

set -e

echo "=========================================="
echo "aicred Export Usage Examples"
echo "=========================================="
echo

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if aicred is available
if ! command -v aicred &> /dev/null; then
    echo "❌ aicred is not installed or not in PATH"
    exit 1
fi

echo -e "${GREEN}✓${NC} aicred found at: $(which aicred)"
echo

# Example 1: Basic export
echo -e "${BLUE}Example 1: Basic export (bash format)${NC}"
echo "Command: aicred export"
echo "Output:"
aicred export | head -10
echo "..."
echo

# Example 2: Export to fish format
echo -e "${BLUE}Example 2: Export to fish shell format${NC}"
echo "Command: aicred export --format fish"
echo "Output:"
aicred export --format fish | head -5
echo "..."
echo

# Example 3: Export to PowerShell format
echo -e "${BLUE}Example 3: Export to PowerShell format${NC}"
echo "Command: aicred export --format powershell"
echo "Output:"
aicred export --format powershell | head -5
echo "..."
echo

# Example 4: Dry run mode
echo -e "${BLUE}Example 4: Dry run mode (preview without writing)${NC}"
echo "Command: aicred export --dry-run"
aicred export --dry-run | head -5
echo "..."
echo

# Example 5: Using custom template
if [ -f "custom-export-template.yaml" ]; then
    echo -e "${BLUE}Example 5: Using custom template${NC}"
    echo "Command: aicred export --template custom-export-template.yaml --vars default_provider=openai --vars model_version=v1"
    echo "Output:"
    aicred export --template custom-export-template.yaml --vars default_provider=openai --vars model_version=v1 | head -10
    echo "..."
    echo
else
    echo -e "${YELLOW}⚠${NC} Skipping template example (custom-export-template.yaml not found)"
    echo
fi

# Example 6: Adding a prefix
echo -e "${BLUE}Example 6: Adding a prefix to all variables${NC}"
echo "Command: aicred export --prefix AI"
echo "Output:"
aicred export --prefix AI | head -5
echo "..."
echo

# Example 7: Export to file
TEMP_FILE=$(mktemp)
echo -e "${BLUE}Example 7: Export to file${NC}"
echo "Command: aicred export --output $TEMP_FILE"
aicred export --output "$TEMP_FILE"
echo -e "${GREEN}✓${NC} Exported to: $TEMP_FILE"
echo "Content:"
head -10 "$TEMP_FILE"
echo "..."
rm "$TEMP_FILE"
echo

# Example 8: Integration with source command
echo -e "${BLUE}Example 8: Integration with shell source${NC}"
echo "Add this to your ~/.bashrc or ~/.zshrc:"
echo "  eval \"\$(aicred export)\""
echo
echo "For fish shell, add to ~/.config/fish/config.fish:"
echo "  aicred export --format fish | source"
echo
echo "For PowerShell, add to \$PROFILE:"
echo "  aicred export --format powershell | Invoke-Expression"
echo

# Example 9: Using in Docker
echo -e "${BLUE}Example 9: Using with Docker${NC}"
echo "Create an environment file:"
echo "  aicred export --output .env"
echo
echo "Use with docker run:"
echo "  docker run --env-file <(aicred export) my-image"
echo

# Example 10: Creating a development environment
echo -e "${BLUE}Example 10: Creating a development environment${NC}"
echo "Create a dev template:"
cat << 'EOF' > /tmp/dev-template.yaml
header: |
  Development Environment
  ========================
  DO NOT USE IN PRODUCTION!

variables:
  - name: DEV_OPENAI_KEY
    value: "{{openai.api_key}}"
    description: "OpenAI key for development only"

  - name: DEV_ENV
    value: "development"
    description: "Environment setting"
    required: true
EOF
echo
echo "Export with dev template:"
echo "  aicred export --template /tmp/dev-template.yaml"
echo
echo "Preview output:"
aicred export --template /tmp/dev-template.yaml | head -5
echo "..."
rm /tmp/dev-template.yaml
echo

# Security reminder
echo -e "${YELLOW}⚠️  Security Reminders${NC}"
echo "  • Never commit export files with actual secrets"
echo "  • Add *.env.sh, ai-config.sh to .gitignore"
echo "  • Set restrictive permissions: chmod 600 <file>"
echo "  • Avoid --include-secrets flag unless absolutely necessary"
echo

echo -e "${GREEN}=========================================="
echo "Examples completed!"
echo "==========================================${NC}"
