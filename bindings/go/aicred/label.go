package aicred

import (
	"errors"
	"time"
)

// ProviderModelTuple represents a provider:model tuple
type ProviderModelTuple struct {
	Provider string `json:"provider"`
	Model    string `json:"model"`
}

// Label represents a unique identifier
type Label struct {
	ID                 string             `json:"id"`
	Name               string             `json:"name"`
	Description        *string            `json:"description,omitempty"`
	Color              *string            `json:"color,omitempty"`
	ProviderModelTuple *ProviderModelTuple `json:"provider_model_tuple,omitempty"`
	Metadata           map[string]string  `json:"metadata,omitempty"`
	CreatedAt          time.Time          `json:"created_at"`
	UpdatedAt          time.Time          `json:"updated_at"`
}

// NewLabel creates a new label
func NewLabel(id, name string) *Label {
	now := time.Now().UTC()
	return &Label{
		ID:                 id,
		Name:               name,
		Description:        nil,
		Color:              nil,
		ProviderModelTuple: nil,
		Metadata:           nil,
		CreatedAt:          now,
		UpdatedAt:          now,
	}
}

func (l *Label) Validate() error {
	if l.ID == "" {
		return errors.New("label ID cannot be empty")
	}
	if l.Name == "" {
		return errors.New("label name cannot be empty")
	}
	return nil
}

// LabelAssignment represents assignment of a label to a target
type LabelAssignment struct {
	ID        string               `json:"id"`
	LabelID   string               `json:"label_id"`
	Target    *LabelTargetInfo     `json:"target"`
	Metadata  map[string]string    `json:"metadata,omitempty"`
	CreatedAt time.Time            `json:"created_at"`
	UpdatedAt time.Time            `json:"updated_at"`
}

// LabelTargetInfo contains information about a label target
type LabelTargetInfo struct {
	Type      string `json:"type"`
	InstanceID string `json:"instance_id"`
	ModelID   string `json:"model_id,omitempty"`
}

// NewLabelAssignment creates a new label assignment
func NewLabelAssignment(id, labelID, targetType, instanceID, modelID string) *LabelAssignment {
	now := time.Now().UTC()
	return &LabelAssignment{
		ID:     id,
		LabelID: labelID,
		Target: &LabelTargetInfo{
			Type:      targetType,
			InstanceID: instanceID,
			ModelID:   modelID,
		},
		CreatedAt: now,
		UpdatedAt: now,
	}
}

// LabelRepository manages labels and their assignments
type LabelRepository struct {
	labels      map[string]*Label
	assignments map[string]*LabelAssignment
}

// NewLabelRepository creates a new label repository
func NewLabelRepository() *LabelRepository {
	return &LabelRepository{
		labels:      make(map[string]*Label),
		assignments: make(map[string]*LabelAssignment),
	}
}

func (lr *LabelRepository) AddLabel(label *Label) error {
	if label == nil {
		return errors.New("label cannot be nil")
	}
	if err := label.Validate(); err != nil {
		return err
	}
	lr.labels[label.ID] = label
	return nil
}

func (lr *LabelRepository) GetLabel(labelID string) (*Label, error) {
	label, exists := lr.labels[labelID]
	if !exists {
		return nil, ErrLabelNotFound
	}
	return label, nil
}

func (lr *LabelRepository) ListLabels() []*Label {
	labels := make([]*Label, 0, len(lr.labels))
	for _, label := range lr.labels {
		labels = append(labels, label)
	}
	return labels
}
