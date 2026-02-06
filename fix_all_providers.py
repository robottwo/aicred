#!/usr/bin/env python3
"""
Fix all ProviderInstance::new_without_models calls in provider test files.
The correct parameter order is:
  ProviderInstance::new_without_models(id, provider_type, base_url, api_key)
"""

import os

def fix_file(filename):
    """Fix ProviderInstance::new_without_models calls in a file."""
    with open(filename, 'r') as f:
        lines = f.readlines()

    modified = False
    new_lines = []
    i = 0

    while i < len(lines):
        line = lines[i]

        # Check if this line contains ProviderInstance::new_without_models
        if 'ProviderInstance::new_without_models(' in line:
            # Save this line
            new_lines.append(line)

            # Read next 4 lines
            if i + 1 < len(lines):
                new_lines.append(lines[i + 1])  # "test-xxx".to_string()
            else:
                i += 1
                continue

            if i + 2 < len(lines):
                line3 = lines[i + 2]  # provider_type or "Test XXX"
            else:
                i += 1
                continue

            if i + 3 < len(lines):
                line4 = lines[i + 3]  # base_url or provider_type
            else:
                i += 1
                continue

            if i + 4 < len(lines):
                line5 = lines[i + 4]  # api_key or base_url
            else:
                i += 1
                continue

            if i + 5 < len(lines):
                line6 = lines[i + 5]  # closing )
            else:
                i += 1
                continue

            # Detect the pattern and fix it
            # Pattern 1: "Test XXX" followed by lowercase provider followed by URL
            if '"Test ' in line3:
                provider_match = None
                if '"groq"' in line4:
                    provider_match = ('"groq".to_string()', "https://api.groq.com")
                elif '"huggingface"' in line4:
                    provider_match = ('"huggingface".to_string()', "https://huggingface.co")
                elif '"litellm"' in line4:
                    provider_match = ('"litellm".to_string()', "https://api.litellm.com")
                elif '"ollama"' in line4:
                    provider_match = ('"ollama".to_string()', "http://localhost:11434")

                if provider_match and ('https://' in line5 or 'http://' in line5):
                    # Fix it
                    new_lines.append(f'            {provider_match[0]},\n')
                    new_lines.append(f'            "{line5.strip().strip("\'\\",")}".to_string(),\n')
                    new_lines.append('            String::new(),\n')
                    new_lines.append('        )\n')
                    modified = True
                    i += 6
                    continue

            # Pattern 2: Already have lowercase provider but wrong order
            # Line 3: "groq".to_string(),
            # Line 4: "groq".to_string(),  <- This should be URL
            # Line 5: "https://xxx".to_string(),  <- This should be String::new()
            if any(f'"{p}"' in line4 for p in ['groq', 'huggingface', 'litellm', 'ollama']):
                if 'https://' in line5 or 'http://' in line5:
                    # Fix it: swap line 4 and 5, make line 5 String::new()
                    new_lines.append(line5)  # URL becomes base_url
                    new_lines.append('            String::new(),\n')
                    new_lines.append('        )\n')
                    modified = True
                    i += 5
                    continue

            # If no pattern matched, keep original lines
            new_lines.append(line3)
            new_lines.append(line4)
            new_lines.append(line5)
            new_lines.append(line6)
            i += 6
        else:
            new_lines.append(line)
            i += 1

    if modified:
        with open(filename, 'w') as f:
            f.writelines(new_lines)
        return True
    return False

# Files to fix
files = [
    'core/src/providers/groq.rs',
    'core/src/providers/huggingface.rs',
    'core/src/providers/litellm.rs',
    'core/src/providers/ollama.rs',
]

for f in files:
    if os.path.exists(f):
        if fix_file(f):
            print(f"✓ Fixed {f}")
        else:
            print(f"- No changes in {f}")
    else:
        print(f"✗ File not found: {f}")
