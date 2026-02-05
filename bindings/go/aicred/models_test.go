package aicred

import (
	"testing"
)

func TestNewModel(t *testing.T) {
	model := NewModel("gpt-4", "GPT-4")

	if model.ModelID != "gpt-4" {
		t.Errorf("Expected ModelID gpt-4, got %s", model.ModelID)
	}

	if model.Name != "GPT-4" {
		t.Errorf("Expected Name 'GPT-4', got %s", model.Name)
	}

	if model.Tags == nil {
		t.Error("Tags should be initialized as empty slice")
	}

	if len(model.Tags) != 0 {
		t.Errorf("Expected 0 tags, got %d", len(model.Tags))
	}
}

func TestModelWithCapabilities(t *testing.T) {
	model := NewModel("gpt-4", "GPT-4")
	capabilities := &Capabilities{
		TextGeneration:  true,
		FunctionCalling: true,
	}

	result := model.WithCapabilities(capabilities)

	if result.Capabilities == nil {
		t.Error("Capabilities should be set")
	}

	if !result.Capabilities.TextGeneration {
		t.Error("TextGeneration should be true")
	}

	if !result.Capabilities.FunctionCalling {
		t.Error("FunctionCalling should be true")
	}
}

func TestModelWithContextWindow(t *testing.T) {
	model := NewModel("gpt-4", "GPT-4")
	window := uint32(128000)

	result := model.WithContextWindow(window)

	if result.ContextWindow == nil {
		t.Error("ContextWindow should be set")
	}

	if *result.ContextWindow != window {
		t.Errorf("Expected ContextWindow %d, got %d", window, *result.ContextWindow)
	}
}

func TestModelWithTemperature(t *testing.T) {
	model := NewModel("gpt-4", "GPT-4")
	temp := float32(0.7)

	result := model.WithTemperature(temp)

	if result.Temperature == nil {
		t.Error("Temperature should be set")
	}

	if *result.Temperature != temp {
		t.Errorf("Expected Temperature %f, got %f", temp, *result.Temperature)
	}
}

func TestModelWithTags(t *testing.T) {
	model := NewModel("gpt-4", "GPT-4")
	tags := []string{"fast", "accurate"}

	result := model.WithTags(tags)

	if len(result.Tags) != 2 {
		t.Errorf("Expected 2 tags, got %d", len(result.Tags))
	}

	if result.Tags[0] != "fast" {
		t.Errorf("Expected tag 'fast', got %s", result.Tags[0])
	}
}

func TestModelValidate(t *testing.T) {
	tests := []struct {
		name      string
		model     *Model
		wantError bool
	}{
		{
			name:      "valid model",
			model:     NewModel("gpt-4", "GPT-4"),
			wantError: false,
		},
		{
			name:      "empty model ID",
			model:     &Model{Name: "GPT-4"},
			wantError: true,
		},
		{
			name:      "empty name",
			model:     &Model{ModelID: "gpt-4"},
			wantError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.model.Validate()
			if (err != nil) != tt.wantError {
				t.Errorf("Validate() error = %v, wantError %v", err, tt.wantError)
			}
		})
	}
}

func TestModelClone(t *testing.T) {
	original := NewModel("gpt-4", "GPT-4")
	original.Temperature = float32Ptr(0.7)
	original.Tags = []string{"fast", "accurate"}

	clone := original.Clone()

	if clone.ModelID != original.ModelID {
		t.Errorf("Clone ModelID mismatch")
	}

	if clone.Name != original.Name {
		t.Errorf("Clone Name mismatch")
	}

	if clone.Temperature == nil || *clone.Temperature != *original.Temperature {
		t.Errorf("Clone Temperature mismatch")
	}

	// Verify independence
	original.Tags[0] = "modified"
	if clone.Tags[0] == "modified" {
		t.Error("Clone should be independent of original")
	}
}

func float32Ptr(f float32) *float32 {
	return &f
}
