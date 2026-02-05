package aicred

import (
	"testing"
	"time"
)

func TestNewTag(t *testing.T) {
	tag := NewTag("tag-1", "Test Tag")

	if tag.ID != "tag-1" {
		t.Errorf("Expected ID tag-1, got %s", tag.ID)
	}

	if tag.Name != "Test Tag" {
		t.Errorf("Expected Name 'Test Tag', got %s", tag.Name)
	}

	if tag.Description != nil {
		t.Error("Description should be nil initially")
	}

	if tag.Color != nil {
		t.Error("Color should be nil initially")
	}

	if tag.Metadata != nil {
		t.Error("Metadata should be nil initially")
	}

	if time.Since(tag.CreatedAt) > time.Second {
		t.Error("CreatedAt should be recent")
	}

	if time.Since(tag.UpdatedAt) > time.Second {
		t.Error("UpdatedAt should be recent")
	}
}

func TestTagValidate(t *testing.T) {
	tests := []struct {
		name      string
		tag       *Tag
		wantError bool
	}{
		{
			name:      "valid tag",
			tag:       NewTag("tag-1", "Test Tag"),
			wantError: false,
		},
		{
			name:      "empty ID",
			tag:       &Tag{Name: "Test Tag"},
			wantError: true,
		},
		{
			name:      "empty name",
			tag:       &Tag{ID: "tag-1"},
			wantError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.tag.Validate()
			if (err != nil) != tt.wantError {
				t.Errorf("Validate() error = %v, wantError %v", err, tt.wantError)
			}
		})
	}
}

func TestNewTagAssignment(t *testing.T) {
	assignment := NewTagAssignment("assign-1", "tag-1", "instance", "instance-1", "model-1")

	if assignment.ID != "assign-1" {
		t.Errorf("Expected ID assign-1, got %s", assignment.ID)
	}

	if assignment.TagID != "tag-1" {
		t.Errorf("Expected TagID tag-1, got %s", assignment.TagID)
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

func TestTagRepositoryAddTag(t *testing.T) {
	repo := NewTagRepository()
	tag := NewTag("tag-1", "Test Tag")

	err := repo.AddTag(tag)
	if err != nil {
		t.Fatalf("AddTag() error = %v", err)
	}

	retrieved, err := repo.GetTag("tag-1")
	if err != nil {
		t.Fatalf("GetTag() error = %v", err)
	}

	if retrieved.ID != tag.ID {
		t.Errorf("Expected ID %s, got %s", tag.ID, retrieved.ID)
	}
}

func TestTagRepositoryAddTagNil(t *testing.T) {
	repo := NewTagRepository()

	err := repo.AddTag(nil)
	if err == nil {
		t.Error("Expected error when adding nil tag")
	}
}

func TestTagRepositoryAddTagInvalid(t *testing.T) {
	repo := NewTagRepository()
	tag := &Tag{} // Invalid - missing required fields

	err := repo.AddTag(tag)
	if err == nil {
		t.Error("Expected error when adding invalid tag")
	}
}

func TestTagRepositoryGetTagNotFound(t *testing.T) {
	repo := NewTagRepository()

	_, err := repo.GetTag("nonexistent")
	if err != ErrTagNotFound {
		t.Errorf("Expected ErrTagNotFound, got %v", err)
	}
}

func TestTagRepositoryListTags(t *testing.T) {
	repo := NewTagRepository()

	// Add multiple tags
	for i := 0; i < 3; i++ {
		tag := NewTag("tag-"+string(rune('0'+i)), "Tag "+string(rune('0'+i)))
		if err := repo.AddTag(tag); err != nil {
			t.Fatalf("Failed to add tag: %v", err)
		}
	}

	tags := repo.ListTags()
	if len(tags) != 3 {
		t.Errorf("Expected 3 tags, got %d", len(tags))
	}
}
