import pytest
import genai_keyfinder
import tempfile
import os
import json

def test_version():
    """Test version function"""
    version = genai_keyfinder.version()
    assert isinstance(version, str)
    assert len(version) > 0

def test_list_providers():
    """Test list_providers function"""
    providers = genai_keyfinder.list_providers()
    assert isinstance(providers, list)
    assert "openai" in providers
    assert "anthropic" in providers

def test_list_scanners():
    """Test list_scanners function"""
    scanners = genai_keyfinder.list_scanners()
    assert isinstance(scanners, list)
    assert "roo-code" in scanners
    assert "claude-desktop" in scanners

def test_scan_basic():
    """Test basic scan functionality"""
    with tempfile.TemporaryDirectory() as tmpdir:
        result = genai_keyfinder.scan(home_dir=tmpdir)
        assert isinstance(result, dict)
        assert "keys" in result
        assert "config_instances" in result
        assert "home_directory" in result
        assert "scan_started_at" in result
        assert "providers_scanned" in result

def test_scan_with_options():
    """Test scan with various options"""
    with tempfile.TemporaryDirectory() as tmpdir:
        result = genai_keyfinder.scan(
            home_dir=tmpdir,
            include_full_values=False,
            max_file_size=512000,
            only_providers=["openai", "anthropic"]
        )
        assert isinstance(result, dict)
        assert isinstance(result["keys"], list)

def test_scan_with_exclude():
    """Test scan with exclude_providers"""
    with tempfile.TemporaryDirectory() as tmpdir:
        result = genai_keyfinder.scan(
            home_dir=tmpdir,
            exclude_providers=["ollama"]
        )
        assert isinstance(result, dict)

def test_scan_invalid_home():
    """Test scan with invalid home directory"""
    with pytest.raises(Exception):
        genai_keyfinder.scan(home_dir="/nonexistent/path/that/does/not/exist")

def test_result_structure():
    """Test that result has expected structure"""
    with tempfile.TemporaryDirectory() as tmpdir:
        result = genai_keyfinder.scan(home_dir=tmpdir)
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
        result = genai_keyfinder.scan(home_dir=tmpdir, include_full_values=True)
        assert isinstance(result, dict)

def test_scan_with_max_file_size():
    """Test scanning with custom max file size."""
    with tempfile.TemporaryDirectory() as tmpdir:
        result = genai_keyfinder.scan(home_dir=tmpdir, max_file_size=512000)
        assert isinstance(result, dict)

def test_scan_with_provider_filter():
    """Test scanning with provider filtering."""
    with tempfile.TemporaryDirectory() as tmpdir:
        result = genai_keyfinder.scan(home_dir=tmpdir, only_providers=["openai", "anthropic"])
        assert isinstance(result, dict)

def test_scan_with_exclude_filter():
    """Test scanning with provider exclusion."""
    with tempfile.TemporaryDirectory() as tmpdir:
        result = genai_keyfinder.scan(home_dir=tmpdir, exclude_providers=["ollama"])
        assert isinstance(result, dict)