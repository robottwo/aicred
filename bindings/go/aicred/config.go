package aicred

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sync"
	"time"
)

// Config represents the main configuration structure for aicred
type Config struct {
	Version    string                     `json:"version"`
	HomeDir    string                     `json:"home_dir"`
	ConfigDir  string                     `json:"config_dir,omitempty"`
	Instances  map[string]*ProviderInstance `json:"instances"`
	Tags       *TagRepository             `json:"tags"`
	Labels     *LabelRepository           `json:"labels"`
	Metadata   map[string]string          `json:"metadata,omitempty"`
	CreatedAt  time.Time                  `json:"created_at"`
	UpdatedAt  time.Time                  `json:"updated_at"`
	mu         sync.RWMutex               `json:"-"`
	configPath string                     `json:"-"`
}

// DefaultConfigVersion is the default version for new configs
const DefaultConfigVersion = "1.0.0"

// DefaultConfigFilename is the default filename for config files
const DefaultConfigFilename = "config.json"

// NewConfig creates a new Config with default values
func NewConfig(homeDir, configDir string) *Config {
	now := time.Now().UTC()
	return &Config{
		Version:   DefaultConfigVersion,
		HomeDir:   homeDir,
		ConfigDir: configDir,
		Instances: make(map[string]*ProviderInstance),
		Tags:      NewTagRepository(),
		Labels:    NewLabelRepository(),
		Metadata:  make(map[string]string),
		CreatedAt: now,
		UpdatedAt: now,
	}
}

// LoadConfig loads a configuration from a file
func LoadConfig(configPath string) (*Config, error) {
	// Validate path
	if err := ValidatePath(configPath); err != nil {
		return nil, fmt.Errorf("invalid config path: %w", err)
	}

	// Read file
	data, err := os.ReadFile(configPath)
	if err != nil {
		return nil, fmt.Errorf("failed to read config file: %w", err)
	}

	// Parse JSON
	var config Config
	if err := json.Unmarshal(data, &config); err != nil {
		return nil, fmt.Errorf("failed to parse config JSON: %w", err)
	}

	// Initialize repositories if nil
	if config.Tags == nil {
		config.Tags = NewTagRepository()
	}
	if config.Labels == nil {
		config.Labels = NewLabelRepository()
	}
	if config.Instances == nil {
		config.Instances = make(map[string]*ProviderInstance)
	}

	// Set config path
	config.configPath = configPath

	// Validate config
	if err := ValidateConfig(&config); err != nil {
		return nil, fmt.Errorf("config validation failed: %w", err)
	}

	return &config, nil
}

// LoadDefaultConfig loads the default configuration from the default location
func LoadDefaultConfig() (*Config, error) {
	configDir, err := GetConfigDir()
	if err != nil {
		return nil, fmt.Errorf("failed to get config directory: %w", err)
	}

	configPath := filepath.Join(configDir, DefaultConfigFilename)
	return LoadConfig(configPath)
}

// Save saves the configuration to its file
func (c *Config) Save() error {
	c.mu.Lock()
	defer c.mu.Unlock()

	if c.configPath == "" {
		return fmt.Errorf("config path not set, use SaveWithFile instead")
	}

	c.UpdatedAt = time.Now().UTC()

	// Create directory if it doesn't exist
	if err := os.MkdirAll(filepath.Dir(c.configPath), 0755); err != nil {
		return fmt.Errorf("failed to create config directory: %w", err)
	}

	// Marshal to JSON with indentation
	data, err := json.MarshalIndent(c, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal config: %w", err)
	}

	// Write file
	if err := os.WriteFile(c.configPath, data, 0600); err != nil {
		return fmt.Errorf("failed to write config file: %w", err)
	}

	return nil
}

// SaveWithFile saves the configuration to a specific file
func (c *Config) SaveWithFile(path string) error {
	c.mu.Lock()
	defer c.mu.Unlock()

	if err := ValidatePath(path); err != nil {
		return fmt.Errorf("invalid save path: %w", err)
	}

	c.configPath = path
	c.UpdatedAt = time.Now().UTC()

	// Create directory if it doesn't exist
	if err := os.MkdirAll(filepath.Dir(path), 0755); err != nil {
		return fmt.Errorf("failed to create config directory: %w", err)
	}

	// Marshal to JSON with indentation
	data, err := json.MarshalIndent(c, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal config: %w", err)
	}

	// Write file
	if err := os.WriteFile(path, data, 0600); err != nil {
		return fmt.Errorf("failed to write config file: %w", err)
	}

	return nil
}

// GetInstance retrieves a provider instance by ID
func (c *Config) GetInstance(instanceID string) (*ProviderInstance, error) {
	c.mu.RLock()
	defer c.mu.RUnlock()

	instance, exists := c.Instances[instanceID]
	if !exists {
		return nil, ErrInstanceNotFound
	}
	return instance, nil
}

// AddInstance adds a provider instance to the configuration
func (c *Config) AddInstance(instance *ProviderInstance) error {
	c.mu.Lock()
	defer c.mu.Unlock()

	if instance == nil {
		return NewValidationError("instance cannot be nil", "")
	}

	if err := instance.Validate(); err != nil {
		return fmt.Errorf("instance validation failed: %w", err)
	}

	c.Instances[instance.ID] = instance
	c.UpdatedAt = time.Now().UTC()

	return nil
}

// UpdateInstance updates a provider instance in the configuration
func (c *Config) UpdateInstance(instance *ProviderInstance) error {
	c.mu.Lock()
	defer c.mu.Unlock()

	if instance == nil {
		return NewValidationError("instance cannot be nil", "")
	}

	// Check if instance exists
	if _, exists := c.Instances[instance.ID]; !exists {
		return ErrInstanceNotFound
	}

	if err := instance.Validate(); err != nil {
		return fmt.Errorf("instance validation failed: %w", err)
	}

	c.Instances[instance.ID] = instance
	c.UpdatedAt = time.Now().UTC()

	return nil
}

// RemoveInstance removes a provider instance from the configuration
func (c *Config) RemoveInstance(instanceID string) error {
	c.mu.Lock()
	defer c.mu.Unlock()

	if _, exists := c.Instances[instanceID]; !exists {
		return ErrInstanceNotFound
	}

	delete(c.Instances, instanceID)
	c.UpdatedAt = time.Now().UTC()

	return nil
}

// ListInstances returns all provider instances
func (c *Config) ListInstances() []*ProviderInstance {
	c.mu.RLock()
	defer c.mu.RUnlock()

	instances := make([]*ProviderInstance, 0, len(c.Instances))
	for _, instance := range c.Instances {
		instances = append(instances, instance)
	}
	return instances
}

// AddTag adds a tag to the configuration
func (c *Config) AddTag(tag *Tag) error {
	if err := c.Tags.AddTag(tag); err != nil {
		return err
	}
	c.mu.Lock()
	c.UpdatedAt = time.Now().UTC()
	c.mu.Unlock()
	return nil
}

// GetTag retrieves a tag by ID
func (c *Config) GetTag(tagID string) (*Tag, error) {
	return c.Tags.GetTag(tagID)
}

// ListTags returns all tags
func (c *Config) ListTags() []*Tag {
	return c.Tags.ListTags()
}

// AddLabel adds a label to the configuration
func (c *Config) AddLabel(label *Label) error {
	if err := c.Labels.AddLabel(label); err != nil {
		return err
	}
	c.mu.Lock()
	c.UpdatedAt = time.Now().UTC()
	c.mu.Unlock()
	return nil
}

// GetLabel retrieves a label by ID
func (c *Config) GetLabel(labelID string) (*Label, error) {
	return c.Labels.GetLabel(labelID)
}

// ListLabels returns all labels
func (c *Config) ListLabels() []*Label {
	return c.Labels.ListLabels()
}

// SetMetadata sets a metadata key-value pair
func (c *Config) SetMetadata(key, value string) {
	c.mu.Lock()
	defer c.mu.Unlock()

	if c.Metadata == nil {
		c.Metadata = make(map[string]string)
	}
	c.Metadata[key] = value
	c.UpdatedAt = time.Now().UTC()
}

// GetMetadata retrieves a metadata value by key
func (c *Config) GetMetadata(key string) (string, bool) {
	c.mu.RLock()
	defer c.mu.RUnlock()

	if c.Metadata == nil {
		return "", false
	}
	value, exists := c.Metadata[key]
	return value, exists
}

// RemoveMetadata removes a metadata key
func (c *Config) RemoveMetadata(key string) {
	c.mu.Lock()
	defer c.mu.Unlock()

	if c.Metadata != nil {
		delete(c.Metadata, key)
		c.UpdatedAt = time.Now().UTC()
	}
}

// GetConfigPath returns the config file path
func (c *Config) GetConfigPath() string {
	return c.configPath
}

// SetConfigPath sets the config file path
func (c *Config) SetConfigPath(path string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.configPath = path
}

// Clone creates a deep copy of the config
func (c *Config) Clone() *Config {
	c.mu.RLock()
	defer c.mu.RUnlock()

	data, _ := json.Marshal(c)
	var clone Config
	json.Unmarshal(data, &clone)
	clone.mu = sync.RWMutex{}
	return &clone
}

// Validate validates the entire configuration
func (c *Config) Validate() error {
	return ValidateConfig(c)
}
