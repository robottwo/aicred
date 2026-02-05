#!/bin/bash
# Fix ProviderInstance::new_without_models calls in provider test files

for file in core/src/providers/groq.rs core/src/providers/huggingface.rs core/src/providers/litellm.rs core/src/providers/ollama.rs; do
    if [ -f "$file" ]; then
        echo "Fixing $file..."
        # Create backup
        cp "$file" "$file.bak"

        # Use sed to fix the pattern
        # Pattern: ProviderInstance::new_without_models( "test-xxx", "Test XXX", "xxx", "url" )
        # Fix to: ProviderInstance::new_without_models( "test-xxx", "xxx", "url", String::new() )
        sed -i '' \
            -e 's/ProviderInstance::new_without_models(\s*"\([^"]*\)"\.to_string(),\s*"Test [^"]*"\.to_string(),\s*"\([^"]*\)"\.to_string(),\s*"\([^"]*\)"\.to_string(),\s*)/ProviderInstance::new_without_models("\1".to_string(), "\2".to_string(), "\3".to_string(), String::new())/g' \
            "$file"
    fi
done

echo "Done!"
