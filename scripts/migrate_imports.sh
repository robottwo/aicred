#!/bin/bash
# Migration script for aicred v0.2.0 internal code
set -e

echo "🔄 Starting internal code migration to new types..."

# Find all Rust files in core/src (excluding models/)
RUST_FILES=$(find core/src -name "*.rs" -not -path "*/models/*" -not -path "*/target/*")

echo "📝 Found $(echo "$RUST_FILES" | wc -l) files to update"

# Backup
echo "💾 Creating backup..."
tar -czf migration-backup-$(date +%Y%m%d-%H%M%S).tar.gz core/src

# Replace DiscoveredKey with DiscoveredCredential
echo "🔧 Updating DiscoveredKey → DiscoveredCredential..."
for file in $RUST_FILES; do
    sed -i '' 's/DiscoveredKey/DiscoveredCredential/g' "$file"
done

# Replace old ValueType with ValueTypeNew temporarily
echo "🔧 Updating ValueType references..."
for file in $RUST_FILES; do
    # Only in type contexts, not as variable names
    sed -i '' 's/ValueType::/ValueTypeNew::/g' "$file"
done

echo "✅ Import migration phase 1 complete"
echo "⚠️  Manual fixes still needed for:"
echo "   - Type constructor changes"
echo "   - Field name changes"
echo "   - Method signature changes"
