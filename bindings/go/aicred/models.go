package aicred

import "encoding/json"

// TokenCost represents token cost tracking
type TokenCost struct {
	InputCostPerMillion     *float64 `json:"input_cost_per_million,omitempty"`
	OutputCostPerMillion    *float64 `json:"output_cost_per_million,omitempty"`
	CachedInputCostModifier *float64 `json:"cached_input_cost_modifier,omitempty"`
}

// Capabilities represents model capabilities
type Capabilities struct {
	TextGeneration  bool   `json:"text_generation"`
	ImageGeneration bool   `json:"image_generation"`
	CodeGeneration  bool   `json:"code_generation"`
	FunctionCalling bool   `json:"function_calling"`
	Streaming      bool   `json:"streaming"`
	Multimodal     bool   `json:"multimodal"`
}

// Model represents an AI model configuration
type Model struct {
	ModelID       string           `json:"model_id"`
	Name          string           `json:"name"`
	Quantization  *string          `json:"quantization,omitempty"`
	ContextWindow *uint32          `json:"context_window,omitempty"`
	Capabilities  *Capabilities    `json:"capabilities,omitempty"`
	Temperature   *float32         `json:"temperature,omitempty"`
	Tags          []string         `json:"tags,omitempty"`
	Cost          *TokenCost       `json:"cost,omitempty"`
	Metadata      map[string]interface{} `json:"metadata,omitempty"`
}

// NewModel creates a new model
func NewModel(modelID, name string) *Model {
	return &Model{
		ModelID:      modelID,
		Name:         name,
		Tags:         []string{},
	}
}

func (m *Model) WithCapabilities(capabilities *Capabilities) *Model {
	m.Capabilities = capabilities
	return m
}

func (m *Model) WithContextWindow(size uint32) *Model {
	m.ContextWindow = &size
	return m
}

func (m *Model) WithTemperature(temperature float32) *Model {
	m.Temperature = &temperature
	return m
}

func (m *Model) WithTags(tags []string) *Model {
	m.Tags = tags
	return m
}

func (m *Model) Validate() error {
	if m.ModelID == "" {
		return NewValidationError("model ID cannot be empty", "model_id")
	}
	if m.Name == "" {
		return NewValidationError("model name cannot be empty", "name")
	}
	return nil
}

func (m *Model) Clone() *Model {
	data, _ := json.Marshal(m)
	var clone Model
	json.Unmarshal(data, &clone)
	return &clone
}
