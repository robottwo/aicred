package aicred

import (
	"testing"
	"time"
)

func TestNewProviderInstance(t *testing.T) {
	instance := NewProviderInstance("test-1", "Test Provider", "openai", "https://api.openai.com")

	if instance.ID != "test-1" {
		t.Errorf("Expected ID test-1, got %s", instance.ID)
	}

	if instance.DisplayName != "Test Provider" {
		t.Errorf("Expected DisplayName 'Test Provider', got %s", instance.DisplayName)
	}

	if instance.ProviderType != "openai" {
		t.Errorf("Expected ProviderType 'openai', got %s", instance.ProviderType)
	}

	if instance.BaseURL != "https://api.openai.com" {
		t.Errorf("Expected BaseURL 'https://api.openai.com', got %s", instance.BaseURL)
	}

	if instance.APIKey != nil {
		t.Error("APIKey should be nil initially")
	}

	if instance.Models == nil {
		t.Error("Models should be initialized as empty slice")
	}

	if !instance.Active {
		t.Error("Active should be true initially")
	}

	if time.Since(instance.CreatedAt) > time.Second {
		t.Error("CreatedAt should be recent")
	}

	if time.Since(instance.UpdatedAt) > time.Second {
		t.Error("UpdatedAt should be recent")
	}
}

func TestProviderInstanceAddModel(t *testing.T) {
	instance := NewProviderInstance("test-1", "Test Provider", "openai", "https://api.openai.com")

	model := NewModel("gpt-4", "GPT-4")
	instance.AddModel(model)

	if len(instance.Models) != 1 {
		t.Errorf("Expected 1 model, got %d", len(instance.Models))
	}

	if instance.Models[0].ModelID != "gpt-4" {
		t.Errorf("Expected model ID gpt-4, got %s", instance.Models[0].ModelID)
	}
}

func TestProviderInstanceSetAPIKey(t *testing.T) {
	instance := NewProviderInstance("test-1", "Test Provider", "openai", "https://api.openai.com")

	instance.SetAPIKey("sk-test-12345")

	if instance.APIKey == nil {
		t.Error("APIKey should be set")
	}

	if *instance.APIKey != "sk-test-12345" {
		t.Errorf("Expected APIKey 'sk-test-12345', got %s", *instance.APIKey)
	}
}

func TestProviderInstanceGetAPIKey(t *testing.T) {
	instance := NewProviderInstance("test-1", "Test Provider", "openai", "https://api.openai.com")

	// Test nil API key
	key := instance.GetAPIKey()
	if key != nil {
		t.Error("Expected nil API key")
	}

	// Set and get API key
	instance.SetAPIKey("sk-test-12345")
	key = instance.GetAPIKey()
	if key == nil || *key != "sk-test-12345" {
		t.Errorf("Expected APIKey 'sk-test-12345', got %v", key)
	}
}

func TestProviderInstanceModelCount(t *testing.T) {
	instance := NewProviderInstance("test-1", "Test Provider", "openai", "https://api.openai.com")

	if instance.ModelCount() != 0 {
		t.Errorf("Expected 0 models, got %d", instance.ModelCount())
	}

	instance.AddModel(NewModel("gpt-4", "GPT-4"))
	instance.AddModel(NewModel("gpt-3.5-turbo", "GPT-3.5 Turbo"))

	if instance.ModelCount() != 2 {
		t.Errorf("Expected 2 models, got %d", instance.ModelCount())
	}
}

func TestProviderInstanceValidate(t *testing.T) {
	tests := []struct {
		name      string
		instance  *ProviderInstance
		wantError bool
	}{
		{
			name:      "valid instance",
			instance:  NewProviderInstance("test-1", "Test Provider", "openai", "https://api.openai.com"),
			wantError: false,
		},
		{
			name:      "empty ID",
			instance:  &ProviderInstance{DisplayName: "Test", ProviderType: "openai", BaseURL: "https://api.openai.com"},
			wantError: true,
		},
		{
			name:      "empty display name",
			instance:  &ProviderInstance{ID: "test-1", ProviderType: "openai", BaseURL: "https://api.openai.com"},
			wantError: true,
		},
		{
			name:      "empty provider type",
			instance:  &ProviderInstance{ID: "test-1", DisplayName: "Test", BaseURL: "https://api.openai.com"},
			wantError: true,
		},
		{
			name:      "empty base URL",
			instance:  &ProviderInstance{ID: "test-1", DisplayName: "Test", ProviderType: "openai"},
			wantError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.instance.Validate()
			if (err != nil) != tt.wantError {
				t.Errorf("Validate() error = %v, wantError %v", err, tt.wantError)
			}
		})
	}
}
