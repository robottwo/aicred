package aicred

import (
	"testing"
	"time"
)

func TestNewLabel(t *testing.T) {
	label := NewLabel("label-1", "Test Label")

	if label.ID != "label-1" {
		t.Errorf("Expected ID label-1, got %s", label.ID)
	}

	if label.Name != "Test Label" {
		t.Errorf("Expected Name 'Test Label', got %s", label.Name)
	}

	if label.Description != nil {
		t.Error("Description should be nil initially")
	}

	if label.Color != nil {
		t.Error("Color should be nil initially")
	}

	if label.ProviderModelTuple != nil {
		t.Error("ProviderModelTuple should be nil initially")
	}

	if label.Metadata != nil {
		t.Error("Metadata should be nil initially")
	}

	if time.Since(label.CreatedAt) > time.Second {
		t.Error("CreatedAt should be recent")
	}

	if time.Since(label.UpdatedAt) > time.Second {
		t.Error("UpdatedAt should be recent")
	}
}

func TestLabelValidate(t *testing.T) {
	tests := []struct {
		name      string
		label     *Label
		wantError bool
	}{
		{
			name:      "valid label",
			label:     NewLabel("label-1", "Test Label"),
			wantError: false,
		},
		{
			name:      "empty ID",
			label:     &Label{Name: "Test Label"},
			wantError: true,
		},
		{
			name:      "empty name",
			label:     &Label{ID: "label-1"},
			wantError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.label.Validate()
			if (err != nil) != tt.wantError {
				t.Errorf("Validate() error = %v, wantError %v", err, tt.wantError)
			}
		})
	}
}

func TestNewLabelAssignment(t *testing.T) {
	assignment := NewLabelAssignment("assign-1", "label-1", "instance", "instance-1", "model-1")

	if assignment.ID != "assign-1" {
		t.Errorf("Expected ID assign-1, got %s", assignment.ID)
	}

	if assignment.LabelID != "label-1" {
		t.Errorf("Expected LabelID label-1, got %s", assignment.LabelID)
	}

	if assignment.Target == nil {
		t.Fatal("Target should be set")
	}

	if assignment.Target.Type != "instance" {
		t.Errorf("Expected Type 'instance', got %s", assignment.Target.Type)
	}

	if assignment.Target.InstanceID != "instance-1" {
		t.Errorf("Expected InstanceID instance-1, got %s", assignment.Target.InstanceID)
	}

	if assignment.Target.ModelID != "model-1" {
		t.Errorf("Expected ModelID model-1, got %s", assignment.Target.ModelID)
	}

	if time.Since(assignment.CreatedAt) > time.Second {
		t.Error("CreatedAt should be recent")
	}
}

func TestLabelRepositoryAddLabel(t *testing.T) {
	repo := NewLabelRepository()
	label := NewLabel("label-1", "Test Label")

	err := repo.AddLabel(label)
	if err != nil {
		t.Fatalf("AddLabel() error = %v", err)
	}

	retrieved, err := repo.GetLabel("label-1")
	if err != nil {
		t.Fatalf("GetLabel() error = %v", err)
	}

	if retrieved.ID != label.ID {
		t.Errorf("Expected ID %s, got %s", label.ID, retrieved.ID)
	}
}

func TestLabelRepositoryAddLabelNil(t *testing.T) {
	repo := NewLabelRepository()

	err := repo.AddLabel(nil)
	if err == nil {
		t.Error("Expected error when adding nil label")
	}
}

func TestLabelRepositoryAddLabelInvalid(t *testing.T) {
	repo := NewLabelRepository()
	label := &Label{} // Invalid - missing required fields

	err := repo.AddLabel(label)
	if err == nil {
		t.Error("Expected error when adding invalid label")
	}
}

func TestLabelRepositoryGetLabelNotFound(t *testing.T) {
	repo := NewLabelRepository()

	_, err := repo.GetLabel("nonexistent")
	if err != ErrLabelNotFound {
		t.Errorf("Expected ErrLabelNotFound, got %v", err)
	}
}

func TestLabelRepositoryListLabels(t *testing.T) {
	repo := NewLabelRepository()

	// Add multiple labels
	for i := 0; i < 3; i++ {
		label := NewLabel("label-"+string(rune('0'+i)), "Label "+string(rune('0'+i)))
		if err := repo.AddLabel(label); err != nil {
			t.Fatalf("Failed to add label: %v", err)
		}
	}

	labels := repo.ListLabels()
	if len(labels) != 3 {
		t.Errorf("Expected 3 labels, got %d", len(labels))
	}
}

func TestProviderModelTuple(t *testing.T) {
	tuple := &ProviderModelTuple{
		Provider: "openai",
		Model:    "gpt-4",
	}

	if tuple.Provider != "openai" {
		t.Errorf("Expected Provider 'openai', got %s", tuple.Provider)
	}

	if tuple.Model != "gpt-4" {
		t.Errorf("Expected Model 'gpt-4', got %s", tuple.Model)
	}
}
