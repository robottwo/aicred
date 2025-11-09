#!/usr/bin/env python3
"""
Demonstration of the fixed PyModel.metadata getter functionality.
This shows that metadata is now properly returned instead of always being None.
"""

import aicred


def main():
    print("=== PyModel Metadata Getter Fix Demo ===\n")

    # Create a model
    model = aicred.Model(
        model_id="gpt-4-demo",
        provider_instance_id="openai-demo",
        name="GPT-4 Demo Model",
    )

    print("1. Initial state:")
    print(f"   model.metadata = {model.metadata}")

    # Set some metadata
    print("\n2. Setting metadata...")
    test_metadata = {
        "description": "A powerful language model for text generation",
        "version": "1.0.0",
        "capabilities": ["text-generation", "code-completion", "translation"],
        "max_tokens": 8192,
        "supports_streaming": True,
    }

    model.metadata = test_metadata
    print("   Metadata set successfully!")

    # Retrieve and verify metadata
    print("\n3. Retrieving metadata...")
    retrieved_metadata = model.metadata

    if retrieved_metadata is None:
        print("   ERROR: metadata is still None!")
        return

    print(f"   Retrieved {len(retrieved_metadata)} metadata entries:")
    for key, value in retrieved_metadata.items():
        print(f"     {key}: {value}")

    # Verify data integrity
    print("\n4. Verifying data integrity...")
    all_match = True
    for key, expected_value in test_metadata.items():
        if key not in retrieved_metadata:
            print(f"   ERROR: Missing key '{key}'")
            all_match = False
        elif retrieved_metadata[key] != expected_value:
            print(
                f"   ERROR: Value mismatch for '{key}': expected {expected_value}, got {retrieved_metadata[key]}"
            )
            all_match = False

    if all_match:
        print("   ✓ All metadata values match perfectly!")

    # Test setting to None
    print("\n5. Testing metadata = None...")
    model.metadata = None
    print(f"   model.metadata = {model.metadata}")

    print("\n=== Demo completed successfully! ===")


if __name__ == "__main__":
    main()
