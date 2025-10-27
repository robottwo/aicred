# Packaging and Distribution

This directory contains packaging configurations for various platforms.

## Available Packages

- **Homebrew**: macOS/Linux formula in `homebrew/`
- **Scoop**: Windows manifest in `scoop/`
- **Chocolatey**: Windows package in `chocolatey/`
- **Linux**: .deb and .rpm packages in `linux/`
- **macOS**: Universal binary installer
- **Windows**: MSI installer

## Building Packages

### Homebrew
```bash
brew install --build-from-source robottwo/aicred/genai-keyfinder
```

### Scoop
```powershell
scoop install genai-keyfinder
```

### Chocolatey
```powershell
choco pack
choco install genai-keyfinder --source="'PATH_TO_NUPKG'"
```

### Linux .deb
```bash
dpkg-deb --build deb/
sudo dpkg -i genai-keyfinder.deb
```

### Windows Installer
Build with Inno Setup:
```bash
iscc packaging/windows/installer.iss
```

## Release Process

1. Update version numbers in all `Cargo.toml` files
2. Update `CHANGELOG.md`
3. Run `scripts/release.sh <version>`
4. CI/CD will automatically build and publish packages
5. Manually approve PyPI and Chocolatey releases if needed