import pytest
import aicred
import tempfile


def test_version():
    """Test version function"""
    version = aicred.version()
    assert isinstance(version, str)
    assert len(version) > 0


def test_list_providers():
    """Test list_providers function"""
    providers = aicred.list_providers()
    assert isinstance(providers, list)
    assert "openai" in providers
    assert "anthropic" in providers


def test_list_scanners():
    """Test list_scanners function"""
    scanners = aicred.list_scanners()
    assert isinstance(scanners, list)
    assert "roo-code" in scanners
    assert "claude-desktop" in scanners


def test_scan_basic():
    """Test basic scan functionality"""
    with tempfile.TemporaryDirectory() as tmpdir:
        result = aicred.scan(home_dir=tmpdir)
        assert isinstance(result, dict)
        assert "keys" in result
        assert "config_instances" in result
        assert "home_directory" in result
        assert "scan_started_at" in result
        assert "providers_scanned" in result


def test_scan_with_options():
    """Test scan with various options"""
    with tempfile.TemporaryDirectory() as tmpdir:
        result = aicred.scan(
            home_dir=tmpdir,
            include_full_values=False,
            max_file_size=512000,
            only_providers=["openai", "anthropic"],
        )
        assert isinstance(result, dict)
        assert isinstance(result["keys"], list)


def test_scan_with_exclude():
    """Test scan with exclude_providers"""
    with tempfile.TemporaryDirectory() as tmpdir:
        result = aicred.scan(home_dir=tmpdir, exclude_providers=["ollama"])
        assert isinstance(result, dict)


def test_scan_invalid_home():
    """Test scan with invalid home directory"""
    with pytest.raises(Exception):
        aicred.scan(home_dir="/nonexistent/path/that/does/not/exist")


def test_result_structure():
    """Test that result has expected structure"""
    with tempfile.TemporaryDirectory() as tmpdir:
        result = aicred.scan(home_dir=tmpdir)
        # Check keys structure
        assert isinstance(result["keys"], list)
        # Check config_instances structure
        assert isinstance(result["config_instances"], list)
        # Check metadata fields available from core
        assert isinstance(result["home_directory"], str)
        assert isinstance(result["scan_started_at"], str)
        assert isinstance(result["providers_scanned"], list)


def test_scan_with_full_values():
    """Test scanning with full values enabled."""
    with tempfile.TemporaryDirectory() as tmpdir:
        result = aicred.scan(home_dir=tmpdir, include_full_values=True)
        assert isinstance(result, dict)


def test_scan_with_max_file_size():
    """Test scanning with custom max file size."""
    with tempfile.TemporaryDirectory() as tmpdir:
        result = aicred.scan(home_dir=tmpdir, max_file_size=512000)
        assert isinstance(result, dict)


def test_scan_with_provider_filter():
    """Test scanning with provider filtering."""
    with tempfile.TemporaryDirectory() as tmpdir:
        result = aicred.scan(home_dir=tmpdir, only_providers=["openai", "anthropic"])
        assert isinstance(result, dict)


def test_scan_with_exclude_filter():
    """Test scanning with provider exclusion."""
    with tempfile.TemporaryDirectory() as tmpdir:
        result = aicred.scan(home_dir=tmpdir, exclude_providers=["ollama"])
        assert isinstance(result, dict)


# New tests for enhanced model and provider instance structures


def test_token_cost_creation():
    """Test TokenCost class creation"""
    cost = aicred.TokenCost(
        input_cost_per_million=0.001,
        output_cost_per_million=0.002,
        cached_input_cost_modifier=0.1,
    )
    assert cost.input_cost_per_million == 0.001
    assert cost.output_cost_per_million == 0.002
    assert cost.cached_input_cost_modifier == 0.1


def test_capabilities_creation():
    """Test Capabilities class creation"""
    caps = aicred.Capabilities(
        text_generation=True, code_generation=True, streaming=True
    )
    assert caps.text_generation is True
    assert caps.code_generation is True
    assert caps.streaming is True
    assert caps.image_generation is False  # Default value


def test_model_creation():
    """Test Model class creation"""
    model = aicred.Model(
        model_id="gpt-4", provider_instance_id="openai-prod", name="GPT-4"
    )
    assert model.model_id == "gpt-4"
    assert model.provider_instance_id == "openai-prod"
    assert model.name == "GPT-4"
    assert model.temperature is None
    assert model.tags is None


def test_model_with_enhanced_features():
    """Test Model with temperature, tags, and cost"""
    cost = aicred.TokenCost(input_cost_per_million=0.001, output_cost_per_million=0.002)

    model = aicred.Model(
        model_id="claude-3", provider_instance_id="anthropic-prod", name="Claude 3"
    )
    model.set_temperature(0.7)
    model.set_tags(["text-generation", "code"])
    model.set_cost(cost)

    assert abs(model.temperature - 0.7) < 0.001
    assert model.tags == ["text-generation", "code"]
    assert model.cost.input_cost_per_million == 0.001


def test_model_validation():
    """Test Model validation"""
    # Valid model
    valid_model = aicred.Model(
        model_id="valid-model",
        provider_instance_id="provider-instance",
        name="Valid Model",
    )
    valid_model.validate()  # Should not raise

    # Invalid model
    invalid_model = aicred.Model(
        model_id="", provider_instance_id="provider-instance", name="Invalid Model"
    )
    with pytest.raises(Exception):
        invalid_model.validate()


def test_provider_instance_creation():
    """Test ProviderInstance creation"""
    instance = aicred.ProviderInstance(
        id="openai-prod",
        display_name="OpenAI Production",
        provider_type="openai",
        base_url="https://api.openai.com",
    )
    assert instance.id == "openai-prod"
    assert instance.display_name == "OpenAI Production"
    assert instance.provider_type == "openai"
    assert instance.base_url == "https://api.openai.com"
    assert instance.active is True
    assert len(instance.models) == 0
    assert len(instance.keys) == 0


def test_provider_instance_with_models():
    """Test ProviderInstance with models"""
    instance = aicred.ProviderInstance(
        id="anthropic-dev",
        display_name="Anthropic Development",
        provider_type="anthropic",
        base_url="https://api.anthropic.com",
    )

    model1 = aicred.Model(
        model_id="claude-3-sonnet",
        provider_instance_id="anthropic-dev",
        name="Claude 3 Sonnet",
    )

    model2 = aicred.Model(
        model_id="claude-3-opus",
        provider_instance_id="anthropic-dev",
        name="Claude 3 Opus",
    )

    instance.add_model(model1)
    instance.add_model(model2)

    assert instance.model_count() == 2
    assert len(instance.models) == 2


def test_provider_instances_collection():
    """Test ProviderInstances collection"""
    instances = aicred.ProviderInstances()
    assert instances.is_empty() is True
    assert instances.len() == 0

    instance1 = aicred.ProviderInstance(
        id="openai-1",
        display_name="OpenAI Instance 1",
        provider_type="openai",
        base_url="https://api.openai.com",
    )

    instance2 = aicred.ProviderInstance(
        id="anthropic-1",
        display_name="Anthropic Instance 1",
        provider_type="anthropic",
        base_url="https://api.anthropic.com",
    )

    instances.add_instance(instance1)
    instances.add_instance(instance2)

    assert instances.len() == 2
    assert instances.is_empty() is False
    assert "openai-1" in instances.instance_ids()
    assert "anthropic-1" in instances.instance_ids()


def test_provider_instances_filtering():
    """Test ProviderInstances filtering methods"""
    instances = aicred.ProviderInstances()

    active_instance = aicred.ProviderInstance(
        id="openai-active",
        display_name="OpenAI Active",
        provider_type="openai",
        base_url="https://api.openai.com",
    )
    active_instance.set_active(True)

    inactive_instance = aicred.ProviderInstance(
        id="openai-inactive",
        display_name="OpenAI Inactive",
        provider_type="openai",
        base_url="https://api.openai.com",
    )
    inactive_instance.set_active(False)

    anthropic_instance = aicred.ProviderInstance(
        id="anthropic-1",
        display_name="Anthropic Instance",
        provider_type="anthropic",
        base_url="https://api.anthropic.com",
    )

    instances.add_instance(active_instance)
    instances.add_instance(inactive_instance)
    instances.add_instance(anthropic_instance)

    # Test filtering by type
    openai_instances = instances.instances_by_type("openai")
    assert len(openai_instances) == 2

    # Test active instances
    active_instances = instances.active_instances()
    assert (
        len(active_instances) == 2
    )  # active_instance and anthropic_instance (default active)

    # Test active instances by type
    active_openai = instances.active_instances_by_type("openai")
    assert len(active_openai) == 1


def test_model_metadata_getter():
    """Test Model metadata getter functionality"""
    model = aicred.Model(
        model_id="test-model", provider_instance_id="test-provider", name="Test Model"
    )

    # Initially metadata should be None
    assert model.metadata is None

    # Set metadata with some test data using property syntax
    test_metadata = {
        "description": "A test model",
        "version": "1.0.0",
        "capabilities": ["text", "code"],
        "custom_field": "custom_value",
    }

    model.metadata = test_metadata

    # Now metadata should return the same data
    retrieved_metadata = model.metadata
    assert retrieved_metadata is not None
    assert len(retrieved_metadata) == len(test_metadata)

    # Check that all keys and values match
    for key, expected_value in test_metadata.items():
        assert key in retrieved_metadata
        assert retrieved_metadata[key] == expected_value


def test_model_metadata_empty():
    """Test Model metadata with empty dict"""
    model = aicred.Model(
        model_id="test-model", provider_instance_id="test-provider", name="Test Model"
    )

    # Set empty metadata using property syntax
    model.metadata = {}
    retrieved_metadata = model.metadata
    assert retrieved_metadata is not None
    assert len(retrieved_metadata) == 0


def test_model_metadata_none():
    """Test Model metadata when set to None"""
    model = aicred.Model(
        model_id="test-model", provider_instance_id="test-provider", name="Test Model"
    )

    # Set metadata first
    test_metadata = {"test": "value"}
    model.metadata = test_metadata
    assert model.metadata is not None

    # Then set to None
    model.metadata = None
    assert model.metadata is None


def test_backward_compatibility():
    """Test that existing functionality still works"""
    # Ensure the main scan function still returns expected structure
    with tempfile.TemporaryDirectory() as tmpdir:
        result = aicred.scan(home_dir=tmpdir)

        # Check that all expected keys are present
        expected_keys = [
            "keys",
            "config_instances",
            "home_directory",
            "scan_started_at",
            "providers_scanned",
        ]
        for key in expected_keys:
            assert key in result

        # Check types
        assert isinstance(result["keys"], list)
        assert isinstance(result["config_instances"], list)
        assert isinstance(result["home_directory"], str)
        assert isinstance(result["scan_started_at"], str)
        assert isinstance(result["providers_scanned"], list)


if __name__ == "__main__":
    # Run all tests
    pytest.main([__file__, "-v"])
