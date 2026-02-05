package aicred

import (
	"os"
	"path/filepath"
	"testing"
)

// TestLoadEmptyInstances tests loading instances when none exist
func TestLoadEmptyInstances(t *testing.T) {
	// Create a temporary directory for testing
	tmpDir, err := os.MkdirTemp("", "aicred-test-*")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	// Load instances (should return empty slice)
	instances, err := LoadInstances(tmpDir)
	if err != nil {
		t.Fatalf("LoadInstances failed: %v", err)
	}

	if len(instances) != 0 {
		t.Errorf("Expected 0 instances, got %d", len(instances))
	}
}

// TestSaveLoadInstances tests saving and loading instances
func TestSaveLoadInstances(t *testing.T) {
	// Create a temporary directory for testing
	tmpDir, err := os.MkdirTemp("", "aicred-test-*")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	// Create test instances
	testInstances := []ProviderInstance{
		{
			ID:           "test1",
			DisplayName:  "Test OpenAI",
			ProviderType: "openai",
			BaseURL:      "https://api.openai.com/v1",
			APIKey:       "sk-test123",
			Active:       true,
			Models: []Model{
				{ModelID: "gpt-4", Name: "GPT-4"},
				{ModelID: "gpt-3.5-turbo", Name: "GPT-3.5 Turbo"},
			},
		},
		{
			ID:           "test2",
			DisplayName:  "Test Anthropic",
			ProviderType: "anthropic",
			BaseURL:      "https://api.anthropic.com",
			APIKey:       "sk-ant-test456",
			Active:       true,
			Models: []Model{
				{ModelID: "claude-3-opus-20240229", Name: "Claude 3 Opus"},
			},
		},
	}

	// Save instances
	err = SaveInstances(tmpDir, testInstances)
	if err != nil {
		t.Fatalf("SaveInstances failed: %v", err)
	}

	// Load instances back
	loadedInstances, err := LoadInstances(tmpDir)
	if err != nil {
		t.Fatalf("LoadInstances failed: %v", err)
	}

	// Verify we got the same number of instances
	if len(loadedInstances) != len(testInstances) {
		t.Errorf("Expected %d instances, got %d", len(testInstances), len(loadedInstances))
	}

	// Verify each instance
	for i, loaded := range loadedInstances {
		expected := testInstances[i]
		if loaded.ID != expected.ID {
			t.Errorf("Instance %d: expected ID %s, got %s", i, expected.ID, loaded.ID)
		}
		if loaded.DisplayName != expected.DisplayName {
			t.Errorf("Instance %d: expected DisplayName %s, got %s", i, expected.DisplayName, loaded.DisplayName)
		}
		if loaded.ProviderType != expected.ProviderType {
			t.Errorf("Instance %d: expected ProviderType %s, got %s", i, expected.ProviderType, loaded.ProviderType)
		}
		if loaded.APIKey != expected.APIKey {
			t.Errorf("Instance %d: expected APIKey %s, got %s", i, expected.APIKey, loaded.APIKey)
		}
	}
}

// TestGetInstance tests retrieving a specific instance
func TestGetInstance(t *testing.T) {
	// Create a temporary directory for testing
	tmpDir, err := os.MkdirTemp("", "aicred-test-*")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	// Create and save test instance
	testInstance := ProviderInstance{
		ID:           "test-get",
		DisplayName:  "Test Get Instance",
		ProviderType: "groq",
		BaseURL:      "https://api.groq.com/openai/v1",
		APIKey:       "gsk-test789",
		Active:       true,
	}

	err = SaveInstances(tmpDir, []ProviderInstance{testInstance})
	if err != nil {
		t.Fatalf("SaveInstances failed: %v", err)
	}

	// Get the instance back
	loaded, err := GetInstance(tmpDir, "test-get")
	if err != nil {
		t.Fatalf("GetInstance failed: %v", err)
	}

	// Verify the instance
	if loaded.ID != testInstance.ID {
		t.Errorf("Expected ID %s, got %s", testInstance.ID, loaded.ID)
	}
	if loaded.DisplayName != testInstance.DisplayName {
		t.Errorf("Expected DisplayName %s, got %s", testInstance.DisplayName, loaded.DisplayName)
	}
	if loaded.ProviderType != testInstance.ProviderType {
		t.Errorf("Expected ProviderType %s, got %s", testInstance.ProviderType, loaded.ProviderType)
	}
}

// TestLoadEmptyLabels tests loading labels when none exist
func TestLoadEmptyLabels(t *testing.T) {
	// Create a temporary directory for testing
	tmpDir, err := os.MkdirTemp("", "aicred-test-*")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	// Load labels (should return empty slice)
	labels, err := LoadLabels(tmpDir)
	if err != nil {
		t.Fatalf("LoadLabels failed: %v", err)
	}

	if len(labels) != 0 {
		t.Errorf("Expected 0 labels, got %d", len(labels))
	}
}

// TestSaveLoadLabels tests saving and loading labels
func TestSaveLoadLabels(t *testing.T) {
	// Create a temporary directory for testing
	tmpDir, err := os.MkdirTemp("", "aicred-test-*")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	// Create test labels
	testLabels := []Label{
		{
			Name:        "fast",
			Description: "Fast models for quick tasks",
			Assignments: []Assignment{
				{InstanceID: "test1", ModelID: "gpt-3.5-turbo"},
				{InstanceID: "test2", ModelID: "llama3-70b-8192"},
			},
		},
		{
			Name:        "smart",
			Description: "High-quality models",
			Assignments: []Assignment{
				{InstanceID: "test1", ModelID: "gpt-4"},
				{InstanceID: "test2", ModelID: "claude-3-opus-20240229"},
			},
		},
	}

	// Save labels
	err = SaveLabels(tmpDir, testLabels)
	if err != nil {
		t.Fatalf("SaveLabels failed: %v", err)
	}

	// Load labels back
	loadedLabels, err := LoadLabels(tmpDir)
	if err != nil {
		t.Fatalf("LoadLabels failed: %v", err)
	}

	// Verify we got the same number of labels
	if len(loadedLabels) != len(testLabels) {
		t.Errorf("Expected %d labels, got %d", len(testLabels), len(loadedLabels))
	}

	// Verify each label
	for i, loaded := range loadedLabels {
		expected := testLabels[i]
		if loaded.Name != expected.Name {
			t.Errorf("Label %d: expected Name %s, got %s", i, expected.Name, loaded.Name)
		}
		if loaded.Description != expected.Description {
			t.Errorf("Label %d: expected Description %s, got %s", i, expected.Description, loaded.Description)
		}
		if len(loaded.Assignments) != len(expected.Assignments) {
			t.Errorf("Label %d: expected %d assignments, got %d", i, len(expected.Assignments), len(loaded.Assignments))
		}
	}
}

// TestLoadEmptyTags tests loading tags when none exist
func TestLoadEmptyTags(t *testing.T) {
	// Create a temporary directory for testing
	tmpDir, err := os.MkdirTemp("", "aicred-test-*")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	// Load tags (should return empty slice)
	tags, err := LoadTags(tmpDir)
	if err != nil {
		t.Fatalf("LoadTags failed: %v", err)
	}

	if len(tags) != 0 {
		t.Errorf("Expected 0 tags, got %d", len(tags))
	}
}

// TestSaveLoadTags tests saving and loading tags
func TestSaveLoadTags(t *testing.T) {
	// Create a temporary directory for testing
	tmpDir, err := os.MkdirTemp("", "aicred-test-*")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	// Create test tags
	testTags := []TagAssignment{
		{
			TagID:   "tag1",
			Name:    "Production",
			Target:  "instance:test1",
			TagType: "instance",
		},
		{
			TagID:   "tag2",
			Name:    "Development",
			Target:  "model:gpt-4",
			TagType: "model",
		},
	}

	// Save tags
	err = SaveTags(tmpDir, testTags)
	if err != nil {
		t.Fatalf("SaveTags failed: %v", err)
	}

	// Load tags back
	loadedTags, err := LoadTags(tmpDir)
	if err != nil {
		t.Fatalf("LoadTags failed: %v", err)
	}

	// Verify we got the same number of tags
	if len(loadedTags) != len(testTags) {
		t.Errorf("Expected %d tags, got %d", len(testTags), len(loadedTags))
	}

	// Verify each tag
	for i, loaded := range loadedTags {
		expected := testTags[i]
		if loaded.TagID != expected.TagID {
			t.Errorf("Tag %d: expected TagID %s, got %s", i, expected.TagID, loaded.TagID)
		}
		if loaded.Name != expected.Name {
			t.Errorf("Tag %d: expected Name %s, got %s", i, expected.Name, loaded.Name)
		}
		if loaded.Target != expected.Target {
			t.Errorf("Tag %d: expected Target %s, got %s", i, expected.Target, loaded.Target)
		}
	}
}

// TestConfigDirCreation tests that config directories are created automatically
func TestConfigDirCreation(t *testing.T) {
	// Create a temporary directory for testing
	tmpDir, err := os.MkdirTemp("", "aicred-test-*")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	// Save an instance to a nonexistent config dir
	testInstance := ProviderInstance{
		ID:           "test-create",
		DisplayName:  "Test Dir Creation",
		ProviderType: "openai",
		BaseURL:      "https://api.openai.com/v1",
		Active:       true,
	}

	err = SaveInstances(tmpDir, []ProviderInstance{testInstance})
	if err != nil {
		t.Fatalf("SaveInstances failed: %v", err)
	}

	// Verify the config directory was created
	configDir := filepath.Join(tmpDir, ".config", "aicred", "inference_services")
	if _, err := os.Stat(configDir); os.IsNotExist(err) {
		t.Errorf("Config directory was not created: %s", configDir)
	}

	// Load the instance back
	instances, err := LoadInstances(tmpDir)
	if err != nil {
		t.Fatalf("LoadInstances failed: %v", err)
	}

	if len(instances) != 1 {
		t.Errorf("Expected 1 instance, got %d", len(instances))
	}
}
