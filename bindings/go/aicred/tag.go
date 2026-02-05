package aicred

import (
	"errors"
	"time"
)

// Tag represents a shared identifier that can be applied to multiple targets
type Tag struct {
	ID          string            `json:"id"`
	Name        string            `json:"name"`
	Description *string           `json:"description,omitempty"`
	Color       *string           `json:"color,omitempty"`
	Metadata    map[string]string `json:"metadata,omitempty"`
	CreatedAt   time.Time         `json:"created_at"`
	UpdatedAt   time.Time         `json:"updated_at"`
}

// NewTag creates a new tag
func NewTag(id, name string) *Tag {
	now := time.Now().UTC()
	return &Tag{
		ID:          id,
		Name:        name,
		Description: nil,
		Color:       nil,
		Metadata:    nil,
		CreatedAt:   now,
		UpdatedAt:   now,
	}
}

func (t *Tag) Validate() error {
	if t.ID == "" {
		return errors.New("tag ID cannot be empty")
	}
	if t.Name == "" {
		return errors.New("tag name cannot be empty")
	}
	return nil
}

// TagAssignment represents assignment of a tag to a target
type TagAssignment struct {
	ID        string              `json:"id"`
	TagID     string              `json:"tag_id"`
	Target    *TagTargetInfo      `json:"target"`
	Metadata  map[string]string   `json:"metadata,omitempty"`
	CreatedAt time.Time          `json:"created_at"`
	UpdatedAt time.Time          `json:"updated_at"`
}

// TagTargetInfo contains information about a tag target
type TagTargetInfo struct {
	Type      string `json:"type"`
	InstanceID string `json:"instance_id"`
	ModelID   string `json:"model_id,omitempty"`
}

// NewTagAssignment creates a new tag assignment
func NewTagAssignment(id, tagID, targetType, instanceID, modelID string) *TagAssignment {
	now := time.Now().UTC()
	return &TagAssignment{
		ID:     id,
		TagID:  tagID,
		Target: &TagTargetInfo{
			Type:      targetType,
			InstanceID: instanceID,
			ModelID:   modelID,
		},
		CreatedAt: now,
		UpdatedAt: now,
	}
}

// TagRepository manages tags and their assignments
type TagRepository struct {
	tags        map[string]*Tag
	assignments map[string][]*TagAssignment
}

// NewTagRepository creates a new tag repository
func NewTagRepository() *TagRepository {
	return &TagRepository{
		tags:        make(map[string]*Tag),
		assignments: make(map[string][]*TagAssignment),
	}
}

func (tr *TagRepository) AddTag(tag *Tag) error {
	if tag == nil {
		return errors.New("tag cannot be nil")
	}
	if err := tag.Validate(); err != nil {
		return err
	}
	tr.tags[tag.ID] = tag
	return nil
}

func (tr *TagRepository) GetTag(tagID string) (*Tag, error) {
	tag, exists := tr.tags[tagID]
	if !exists {
		return nil, ErrTagNotFound
	}
	return tag, nil
}

func (tr *TagRepository) ListTags() []*Tag {
	tags := make([]*Tag, 0, len(tr.tags))
	for _, tag := range tr.tags {
		tags = append(tags, tag)
	}
	return tags
}
