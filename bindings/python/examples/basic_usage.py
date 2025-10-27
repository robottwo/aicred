#!/usr/bin/env python3
"""
Basic usage example for genai-keyfinder Python bindings
"""

import genai_keyfinder
import json
import os

def main():
    print("GenAI Key Finder - Python Example")
    print(f"Version: {genai_keyfinder.version()}\n")
    
    # List available providers
    print("Available Providers:")
    for provider in genai_keyfinder.list_providers():
        print(f"  - {provider}")
    
    print("\nAvailable Scanners:")
    for scanner in genai_keyfinder.list_scanners():
        print(f"  - {scanner}")
    
    # Perform scan
    print("\nScanning for credentials...")
    result = genai_keyfinder.scan(
        include_full_values=False,  # Keep secrets redacted
        only_providers=["openai", "anthropic"]  # Only scan these
    )
    
    # Display results
    print(f"\nFound {len(result['keys'])} keys")
    print(f"Found {len(result['config_instances'])} config instances")
    
    if result['keys']:
        print("\nDiscovered Keys:")
        for key in result['keys']:
            print(f"  {key['provider']}: {key['redacted']} (confidence: {key['confidence']})")
    
    if result['config_instances']:
        print("\nConfig Instances:")
        for instance in result['config_instances']:
            print(f"  {instance['app_name']}: {instance['config_path']}")
    
    # Save to JSON
    with open('scan_result.json', 'w') as f:
        json.dump(result, f, indent=2)
    # Set restrictive permissions (owner read/write only)
    os.chmod('scan_result.json', 0o600)
    print("\nResults saved to scan_result.json")

if __name__ == "__main__":
    main()