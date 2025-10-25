#!/bin/bash
set -e

VERSION=$1
if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>"
  exit 1
fi

echo "Creating release v$VERSION..."

# Update version in Cargo.toml files
find . -name "Cargo.toml" -exec sed -i "" "s/version = \".*\"/version = \"$VERSION\"/g" {} +

# Update changelog
sed -i "" "s/## \[Unreleased\]/## \[Unreleased\]\n\n## \[$VERSION\] - $(date +%Y-%m-%d)/" CHANGELOG.md

# Commit changes
git add .
git commit -m "Release v$VERSION"
git tag "v$VERSION"
git push origin "v$VERSION"

echo "Release process started. CI/CD will handle the rest."