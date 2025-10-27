# Distribution Guide

## Overview

This document covers how to package and distribute genai-keyfinder across different platforms.

## Homebrew (macOS/Linux)

### Tap Setup

```bash
brew tap robottwo/aicred
brew install genai-keyfinder
```

### Formula Maintenance

Update the formula when releasing new versions:

```bash
brew bump-formula-pr --version=X.X.X genai-keyfinder
```

## Scoop (Windows)

### Bucket Setup

```powershell
scoop bucket add genai-keyfinder https://github.com/robottwo/scoop-aicred
scoop install genai-keyfinder
```

### Update Process

```powershell
scoop update genai-keyfinder
```

## Chocolatey (Windows)

### Publishing

```powershell
choco pack packaging/chocolatey/genai-keyfinder.nuspec
choco push genai-keyfinder.0.1.0.nupkg --source https://push.chocolatey.org/
```

## Linux Packages

### .deb Package (Ubuntu/Debian)

Create `packaging/linux/deb/control`:

```
Package: genai-keyfinder
Version: 0.1.0
Section: utils
Priority: optional
Architecture: amd64
Depends: libc6 (>= 2.14)
Maintainer: Your Name <your.email@example.com>
Description: Cross-platform GenAI key discovery tool
 A tool for discovering GenAI API keys and configurations across various providers.
```

### .rpm Package (Fedora/RHEL)

Create `packaging/linux/rpm/spec`:

```spec
Name:           genai-keyfinder
Version:        0.1.0
Release:        1%{?dist}
Summary:        Cross-platform GenAI key discovery tool

License:        MIT
URL:            https://github.com/robottwo/aicred
Source0:        https://github.com/robottwo/aicred/releases/download/v%{version}/keyfinder-linux-x86_64.tar.gz

BuildArch:      x86_64
BuildRequires:  gcc

%description
Cross-platform tool for discovering GenAI API keys and configurations

%prep
%setup -q -T -D -a 0

%build
# No build needed - precompiled binary

%install
mkdir -p %{buildroot}%{_bindir}
install -m 755 keyfinder %{buildroot}%{_bindir}/

%files
%{_bindir}/keyfinder

%changelog
* Mon Oct 20 2024 Your Name <your.email@example.com> - 0.1.0-1
- Initial package