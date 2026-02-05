package aicred

import (
	"errors"
	"time"
)

// ProviderInstance represents a single provider configuration with models
type ProviderInstance struct {
	ID           string      `json:"id"`
	DisplayName  string      `json:"display_name"`
	ProviderType string      `json:"provider_type"`
	BaseURL      string      `json:"base_url"`
	APIKey      *string     `json:"api_key,omitempty"`
	Models       []*Model    `json:"models,omitempty"`
	Metadata     map[string]string `json:"metadata,omitempty"`
	Active       bool        `json:"active"`
	CreatedAt    time.Time   `json:"created_at"`
	UpdatedAt    time.Time   `json:"updated_at"`
}

// NewProviderInstance creates a new provider instance
func NewProviderInstance(id, displayName, providerType, baseURL string) *ProviderInstance {
	now := time.Now().UTC()
	return &ProviderInstance{
		ID:           id,
		DisplayName:  displayName,
		ProviderType: providerType,
		BaseURL:      baseURL,
		APIKey:       nil,
		Models:       []*Model{},
		Metadata:     nil,
		Active:       true,
		CreatedAt:    now,
		UpdatedAt:    now,
	}
}

func (pi *ProviderInstance) AddModel(model *Model) {
	pi.Models = append(pi.Models, model)
	pi.UpdatedAt = time.Now().UTC()
}

func (pi *ProviderInstance) SetAPIKey(apiKey string) {
	pi.APIKey = &apiKey
	pi.UpdatedAt = time.Now().UTC()
}

func (pi *ProviderInstance) GetAPIKey() *string {
	return pi.APIKey
}

func (pi *ProviderInstance) ModelCount() int {
	return len(pi.Models)
}

func (pi *ProviderInstance) Validate() error {
	if pi.ID == "" {
		return errors.New("instance ID cannot be empty")
	}
	if pi.DisplayName == "" {
		return errors.New("display name cannot be empty")
	}
	if pi.ProviderType == "" {
		return errors.New("provider type cannot be empty")
	}
	if pi.BaseURL == "" {
		return errors.New("base URL cannot be empty")
	}
	return nil
}
