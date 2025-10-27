# Security

## Default Behavior

- All secrets are **redacted by default**
- Only last 4 characters shown
- SHA-256 hashes for deduplication
- No network connections
- No telemetry

## Opt-in Full Values

Use `include_full_values` only in secure environments:

```bash
# CLI
keyfinder scan --include-values

# Python
python - <<'PY'
import genai_keyfinder
result = genai_keyfinder.scan(include_full_values=True)
print(len(result["keys"]))
PY

# Go
package main

import (
  "fmt"
  "github.com/robottwo/aicred/bindings/go/genai_keyfinder"
)

func main() {
  res, _ := genai_keyfinder.Scan(genai_keyfinder.ScanOptions{
    IncludeFullValues: true,
  })
  fmt.Println(len(res.Keys))
}
```

## File Size Limits

Default: 1MB per file. Configurable:

```bash
keyfinder scan --max-bytes-per-file 512000
```

## Audit Logging

Enable audit logging:

```bash
keyfinder scan --audit-log scan-audit.log
```

## Best Practices

1. Never commit scan results with full values
2. Use dry-run mode for testing
3. Limit scans to specific providers when possible
4. Review confidence scores
5. Rotate discovered keys immediately

## Reporting Security Issues

Please report security vulnerabilities to security@example.com