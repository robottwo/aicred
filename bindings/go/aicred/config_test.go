package aicred

import (
	"os"
	"path/filepath"
	"testing"
	"time"
)

func TestNewConfig(t *testing.T) {
	config := NewConfig("/home/user", "/home/user/.config/aicred")

	if config.Version != DefaultConfigVersion {
		t.Errorf("Expected version %s, got %s", DefaultConfigVersion, config.Version)
	}

	if config.HomeDir != "/home/user" {
		t.Errorf("Expected home dir /home/user, got %s", config.HomeDir)
	}

	if config.ConfigDir != "/home/user/.config/aicred" {
		t.Errorf("Expected config dir /home/user/.config/aicred, got %s", config.ConfigDir)
	}

	if config.Instances == nil {
		t.Error("Instances map should be initialized")
	}

	if config.Tags == nil {
		t.Error("Tags repository should be initialized")
	}

	if config.Labels == nil {
		t.Error("Labels repository should be initialized")
	}

	if time.Since(config.CreatedAt) > time.Second {
		t.Error("CreatedAt should be recent")
	}
}

func TestConfigSaveAndLoad(t *testing.T) {
	// Create temp directory
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "config.json")

	// Create config
	config := NewConfig("/home/user", tmpDir)

	// Add an instance
	instance := NewProviderInstance("test-1", "Test Provider", "openai", "https://api.openai.com")
	if err := config.AddInstance(instance); err != nil {
		t.Fatalf("Failed to add instance: %v", err)
	}

	// Save config
	if err := config.SaveWithFile(configPath); err != nil {
		t.Fatalf("Failed to save config: %v", err)
	}

	// Load config
	loaded, err := LoadConfig(configPath)
	if err != nil {
		t.Fatalf("Failed to load config: %v", err)
	}

	// Verify loaded config
	if loaded.Version != config.Version {
		t.Errorf("Expected version %s, got %s", config.Version, loaded.Version)
	}

	if loaded.HomeDir != config.HomeDir {
		t.Errorf("Expected home dir %s, got %s", config.HomeDir, loaded.HomeDir)
	}

	if len(loaded.Instances) != 1 {
		t.Errorf("Expected 1 instance, got %d", len(loaded.Instances))
	}

	loadedInstance, err := loaded.GetInstance("test-1")
	if err != nil {
		t.Fatalf("Failed to get instance: %v", err)
	}

	if loadedInstance.ID != instance.ID {
		t.Errorf("Expected instance ID %s, got %s", instance.ID, loadedInstance.ID)
	}
}

func TestConfigAddInstance(t *testing.T) {
	config := NewConfig("/home/user", "/home/user/.config/aicred")

	instance := NewProviderInstance("test-1", "Test Provider", "openai", "https://api.openai.com")

	if err := config.AddInstance(instance); err != nil {
		t.Fatalf("Failed to add instance: %v", err)
	}

	retrieved, err := config.GetInstance("test-1")
	if err != nil {
		t.Fatalf("Failed to retrieve instance: %v", err)
	}

	if retrieved.ID != instance.ID {
		t.Errorf("Expected ID %s, got %s", instance.ID, retrieved.ID)
	}

	if retrieved.DisplayName != instance.DisplayName {
		t.Errorf("Expected DisplayName %s, got %s", instance.DisplayName, retrieved.DisplayName)
	}
}

func TestConfigAddInstanceNil(t *testing.T) {
	config := NewConfig("/home/user", "/home/user/.config/aicred")

	err := config.AddInstance(nil)
	if err == nil {
		t.Error("Expected error when adding nil instance")
	}
}

func TestConfigAddInstanceInvalid(t *testing.T) {
	config := NewConfig("/home/user", "/home/user/.config/aicred")

	instance := &ProviderInstance{} // Invalid - missing required fields

	err := config.AddInstance(instance)
	if err == nil {
		t.Error("Expected error when adding invalid instance")
	}
}

func TestConfigGetInstanceNotFound(t *testing.T) {
	config := NewConfig("/home/user", "/home/user/.config/aicred")

	_, err := config.GetInstance("nonexistent")
	if err != ErrInstanceNotFound {
		t.Errorf("Expected ErrInstanceNotFound, got %v", err)
	}
}

func TestConfigUpdateInstance(t *testing.T) {
	config := NewConfig("/home/user", "/home/user/.config/aicred")

	instance := NewProviderInstance("test-1", "Test Provider", "openai", "https://api.openai.com")
	if err := config.AddInstance(instance); err != nil {
		t.Fatalf("Failed to add instance: %v", err)
	}

	// Update instance
	instance.DisplayName = "Updated Provider"
	if err := config.UpdateInstance(instance); err != nil {
		t.Fatalf("Failed to update instance: %v", err)
	}

	retrieved, err := config.GetInstance("test-1")
	if err != nil {
		t.Fatalf("Failed to retrieve instance: %v", err)
	}

	if retrieved.DisplayName != "Updated Provider" {
		t.Errorf("Expected DisplayName 'Updated Provider', got %s", retrieved.DisplayName)
	}
}

func TestConfigUpdateInstanceNotFound(t *testing.T) {
	config := NewConfig("/home/user", "/home/user/.config/aicred")

	instance := NewProviderInstance("test-1", "Test Provider", "openai", "https://api.openai.com")

	err := config.UpdateInstance(instance)
	if err != ErrInstanceNotFound {
		t.Errorf("Expected ErrInstanceNotFound, got %v", err)
	}
}

func TestConfigRemoveInstance(t *testing.T) {
	config := NewConfig("/home/user", "/home/user/.config/aicred")

	instance := NewProviderInstance("test-1", "Test Provider", "openai", "https://api.openai.com")
	if err := config.AddInstance(instance); err != nil {
		t.Fatalf("Failed to add instance: %v", err)
	}

	if err := config.RemoveInstance("test-1"); err != nil {
		t.Fatalf("Failed to remove instance: %v", err)
	}

	_, err := config.GetInstance("test-1")
	if err != ErrInstanceNotFound {
		t.Errorf("Expected ErrInstanceNotFound after removal, got %v", err)
	}
}

func TestConfigRemoveInstanceNotFound(t *testing.T) {
	config := NewConfig("/home/user", "/home/user/.config/aicred")

	err := config.RemoveInstance("nonexistent")
	if err != ErrInstanceNotFound {
		t.Errorf("Expected ErrInstanceNotFound, got %v", err)
	}
}

func TestConfigListInstances(t *testing.T) {
	config := NewConfig("/home/user", "/home/user/.config/aicred")

	// Add multiple instances
	for i := 0; i < 3; i++ {
		instance := NewProviderInstance(
			"test-"+string(rune('0'+i)),
			"Test Provider "+string(rune('0'+i)),
			"openai",
			"https://api.openai.com",
		)
		if err := config.AddInstance(instance); err != nil {
			t.Fatalf("Failed to add instance: %v", err)
		}
	}

	instances := config.ListInstances()
	if len(instances) != 3 {
		t.Errorf("Expected 3 instances, got %d", len(instances))
	}
}

func TestConfigTags(t *testing.T) {
	config := NewConfig("/home/user", "/home/user/.config/aicred")

	tag := NewTag("tag-1", "Test Tag")

	if err := config.AddTag(tag); err != nil {
		t.Fatalf("Failed to add tag: %v", err)
	}

	retrieved, err := config.GetTag("tag-1")
	if err != nil {
		t.Fatalf("Failed to get tag: %v", err)
	}

	if retrieved.ID != tag.ID {
		t.Errorf("Expected tag ID %s, got %s", tag.ID, retrieved.ID)
	}

	tags := config.ListTags()
	if len(tags) != 1 {
		t.Errorf("Expected 1 tag, got %d", len(tags))
	}
}

func TestConfigLabels(t *testing.T) {
	config := NewConfig("/home/user", "/home/user/.config/aicred")

	label := NewLabel("label-1", "Test Label")

	if err := config.AddLabel(label); err != nil {
		t.Fatalf("Failed to add label: %v", err)
	}

	retrieved, err := config.GetLabel("label-1")
	if err != nil {
		t.Fatalf("Failed to get label: %v", err)
	}

	if retrieved.ID != label.ID {
		t.Errorf("Expected label ID %s, got %s", label.ID, retrieved.ID)
	}

	labels := config.ListLabels()
	if len(labels) != 1 {
		t.Errorf("Expected 1 label, got %d", len(labels))
	}
}

func TestConfigMetadata(t *testing.T) {
	config := NewConfig("/home/user", "/home/user/.config/aicred")

	config.SetMetadata("key1", "value1")
	config.SetMetadata("key2", "value2")

	value, exists := config.GetMetadata("key1")
	if !exists {
		t.Error("Expected key1 to exist")
	}
	if value != "value1" {
		t.Errorf("Expected value1, got %s", value)
	}

	config.RemoveMetadata("key1")

	_, exists = config.GetMetadata("key1")
	if exists {
		t.Error("Expected key1 to be removed")
	}
}

func TestConfigClone(t *testing.T) {
	config := NewConfig("/home/user", "/home/user/.config/aicred")

	instance := NewProviderInstance("test-1", "Test Provider", "openai", "https://api.openai.com")
	config.AddInstance(instance)

	clone := config.Clone()

	if clone.Version != config.Version {
		t.Errorf("Clone version mismatch")
	}

	if len(clone.Instances) != len(config.Instances) {
		t.Errorf("Clone instances count mismatch")
	}

	// Modify original
	instance.DisplayName = "Modified"

	// Verify clone is unchanged
	clonedInstance, _ := clone.GetInstance("test-1")
	if clonedInstance.DisplayName == "Modified" {
		t.Error("Clone should be independent of original")
	}
}

func TestLoadConfigInvalidPath(t *testing.T) {
	_, err := LoadConfig("/nonexistent/path/config.json")
	if err == nil {
		t.Error("Expected error for invalid path")
	}
}

func TestLoadConfigInvalidJSON(t *testing.T) {
	tmpFile := t.TempDir() + "/invalid.json"
	os.WriteFile(tmpFile, []byte("invalid json"), 0644)

	_, err := LoadConfig(tmpFile)
	if err == nil {
		t.Error("Expected error for invalid JSON")
	}
}
