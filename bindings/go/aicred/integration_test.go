package aicred

import (
	"path/filepath"
	"testing"
)

// TestIntegrationConfigWorkflow tests a complete workflow with config management
func TestIntegrationConfigWorkflow(t *testing.T) {
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "config.json")

	// Create config
	config := NewConfig("/home/testuser", tmpDir)

	// Add provider instances
	openaiInstance := NewProviderInstance("openai-prod", "OpenAI Production", "openai", "https://api.openai.com")
	openaiInstance.SetAPIKey("sk-test-key-12345")

	gpt4Model := NewModel("gpt-4", "GPT-4")
	gpt4Model.WithCapabilities(&Capabilities{
		TextGeneration:  true,
		CodeGeneration:  true,
		FunctionCalling: true,
		Streaming:      true,
	}).WithContextWindow(128000).WithTemperature(0.7)

	openaiInstance.AddModel(gpt4Model)
	config.AddInstance(openaiInstance)

	anthropicInstance := NewProviderInstance("anthropic-prod", "Anthropic Production", "anthropic", "https://api.anthropic.com")
	anthropicInstance.SetAPIKey("sk-ant-test-key-67890")

	claudeModel := NewModel("claude-3-opus-20240229", "Claude 3 Opus")
	anthropicInstance.AddModel(claudeModel)
	config.AddInstance(anthropicInstance)

	// Add tags
	fastTag := NewTag("tag-fast", "Fast Models")
	fastTag.Description = stringPtr("Models optimized for speed")
	config.AddTag(fastTag)

	accurateTag := NewTag("tag-accurate", "Accurate Models")
	accurateTag.Description = stringPtr("Models optimized for accuracy")
	config.AddTag(accurateTag)

	// Add labels
	prodLabel := NewLabel("label-prod", "Production")
	prodLabel.Description = stringPtr("Production grade models")
	prodLabel.Color = stringPtr("#ff0000")
	config.AddLabel(prodLabel)

	devLabel := NewLabel("label-dev", "Development")
	devLabel.Color = stringPtr("#00ff00")
	config.AddLabel(devLabel)

	// Set metadata
	config.SetMetadata("environment", "production")
	config.SetMetadata("owner", "test-team")

	// Save config
	if err := config.SaveWithFile(configPath); err != nil {
		t.Fatalf("Failed to save config: %v", err)
	}

	// Load config
	loadedConfig, err := LoadConfig(configPath)
	if err != nil {
		t.Fatalf("Failed to load config: %v", err)
	}

	// Verify instances
	instances := loadedConfig.ListInstances()
	if len(instances) != 2 {
		t.Fatalf("Expected 2 instances, got %d", len(instances))
	}

	// Verify OpenAI instance
	openai, err := loadedConfig.GetInstance("openai-prod")
	if err != nil {
		t.Fatalf("Failed to get OpenAI instance: %v", err)
	}

	if openai.DisplayName != "OpenAI Production" {
		t.Errorf("Expected 'OpenAI Production', got %s", openai.DisplayName)
	}

	if openai.GetAPIKey() == nil || *openai.GetAPIKey() != "sk-test-key-12345" {
		t.Error("API key not preserved correctly")
	}

	if openai.ModelCount() != 1 {
		t.Errorf("Expected 1 model, got %d", openai.ModelCount())
	}

	if openai.Models[0].ModelID != "gpt-4" {
		t.Errorf("Expected model ID 'gpt-4', got %s", openai.Models[0].ModelID)
	}

	if openai.Models[0].Capabilities == nil || !openai.Models[0].Capabilities.TextGeneration {
		t.Error("Capabilities not preserved correctly")
	}

	if openai.Models[0].ContextWindow == nil || *openai.Models[0].ContextWindow != 128000 {
		t.Error("Context window not preserved correctly")
	}

	if openai.Models[0].Temperature == nil || *openai.Models[0].Temperature != 0.7 {
		t.Error("Temperature not preserved correctly")
	}

	// Verify Anthropic instance
	anthropic, err := loadedConfig.GetInstance("anthropic-prod")
	if err != nil {
		t.Fatalf("Failed to get Anthropic instance: %v", err)
	}

	if anthropic.DisplayName != "Anthropic Production" {
		t.Errorf("Expected 'Anthropic Production', got %s", anthropic.DisplayName)
	}

	if anthropic.ModelCount() != 1 {
		t.Errorf("Expected 1 model, got %d", anthropic.ModelCount())
	}

	// Verify tags
	tags := loadedConfig.ListTags()
	if len(tags) != 2 {
		t.Fatalf("Expected 2 tags, got %d", len(tags))
	}

	fastTagLoaded, err := loadedConfig.GetTag("tag-fast")
	if err != nil {
		t.Fatalf("Failed to get fast tag: %v", err)
	}

	if fastTagLoaded.Name != "Fast Models" {
		t.Errorf("Expected 'Fast Models', got %s", fastTagLoaded.Name)
	}

	if fastTagLoaded.Description == nil || *fastTagLoaded.Description != "Models optimized for speed" {
		t.Error("Tag description not preserved correctly")
	}

	// Verify labels
	labels := loadedConfig.ListLabels()
	if len(labels) != 2 {
		t.Fatalf("Expected 2 labels, got %d", len(labels))
	}

	prodLabelLoaded, err := loadedConfig.GetLabel("label-prod")
	if err != nil {
		t.Fatalf("Failed to get prod label: %v", err)
	}

	if prodLabelLoaded.Name != "Production" {
		t.Errorf("Expected 'Production', got %s", prodLabelLoaded.Name)
	}

	if prodLabelLoaded.Color == nil || *prodLabelLoaded.Color != "#ff0000" {
		t.Error("Label color not preserved correctly")
	}

	// Verify metadata
	env, exists := loadedConfig.GetMetadata("environment")
	if !exists {
		t.Error("Metadata 'environment' not found")
	}

	if env != "production" {
		t.Errorf("Expected 'production', got %s", env)
	}

	owner, exists := loadedConfig.GetMetadata("owner")
	if !exists {
		t.Error("Metadata 'owner' not found")
	}

	if owner != "test-team" {
		t.Errorf("Expected 'test-team', got %s", owner)
	}

	// Test update workflow
	openai.DisplayName = "OpenAI Production (Updated)"
	if err := loadedConfig.UpdateInstance(openai); err != nil {
		t.Fatalf("Failed to update instance: %v", err)
	}

	// Save updated config
	if err := loadedConfig.Save(); err != nil {
		t.Fatalf("Failed to save updated config: %v", err)
	}

	// Reload and verify update
	updatedConfig, err := LoadConfig(configPath)
	if err != nil {
		t.Fatalf("Failed to reload config: %v", err)
	}

	updatedInstance, err := updatedConfig.GetInstance("openai-prod")
	if err != nil {
		t.Fatalf("Failed to get updated instance: %v", err)
	}

	if updatedInstance.DisplayName != "OpenAI Production (Updated)" {
		t.Errorf("Expected updated display name, got %s", updatedInstance.DisplayName)
	}

	// Test remove workflow
	if err := updatedConfig.RemoveInstance("anthropic-prod"); err != nil {
		t.Fatalf("Failed to remove instance: %v", err)
	}

	instances = updatedConfig.ListInstances()
	if len(instances) != 1 {
		t.Errorf("Expected 1 instance after removal, got %d", len(instances))
	}

	// Save and verify persistence
	if err := updatedConfig.Save(); err != nil {
		t.Fatalf("Failed to save after removal: %v", err)
	}

	finalConfig, err := LoadConfig(configPath)
	if err != nil {
		t.Fatalf("Failed to load final config: %v", err)
	}

	instances = finalConfig.ListInstances()
	if len(instances) != 1 {
		t.Errorf("Expected 1 instance in final config, got %d", len(instances))
	}

	if _, err := finalConfig.GetInstance("anthropic-prod"); err == nil {
		t.Error("Anthropic instance should have been removed")
	}
}

// TestIntegrationCloneAndIndependence tests that cloned configs are independent
func TestIntegrationCloneAndIndependence(t *testing.T) {
	config := NewConfig("/home/user", "/tmp")

	instance := NewProviderInstance("test-1", "Test", "openai", "https://api.openai.com")
	instance.SetAPIKey("secret-key")
	config.AddInstance(instance)

	clone := config.Clone()

	// Modify original
	instance.DisplayName = "Modified Original"
	instance.SetAPIKey("new-secret")

	// Modify clone
	clonedInstance, _ := clone.GetInstance("test-1")
	clonedInstance.DisplayName = "Modified Clone"

	// Verify original is unchanged
	originalInstance, _ := config.GetInstance("test-1")
	if originalInstance.DisplayName != "Modified Original" {
		t.Errorf("Original should have 'Modified Original', got %s", originalInstance.DisplayName)
	}

	if originalInstance.GetAPIKey() == nil || *originalInstance.GetAPIKey() != "new-secret" {
		t.Error("Original API key should be updated")
	}

	// Verify clone is independent
	if clonedInstance.DisplayName != "Modified Clone" {
		t.Errorf("Clone should have 'Modified Clone', got %s", clonedInstance.DisplayName)
	}

	if clonedInstance.GetAPIKey() == nil || *clonedInstance.GetAPIKey() != "secret-key" {
		t.Error("Clone API key should be original value")
	}
}

func stringPtr(s string) *string {
	return &s
}
