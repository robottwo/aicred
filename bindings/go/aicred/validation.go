package aicred

import (
	"fmt"
)

// ValidateConfig validates a Config struct
func ValidateConfig(config *Config) error {
	if config == nil {
		return NewValidationError("config cannot be nil", "")
	}
	if config.Version == "" {
		return NewValidationError("config version cannot be empty", "version")
	}
	if config.HomeDir == "" {
		return NewValidationError("home directory cannot be empty", "home_dir")
	}
	return nil
}

// ValidateProviderInstance validates a ProviderInstance struct
func ValidateProviderInstance(instance *ProviderInstance) error {
	if instance == nil {
		return NewValidationError("instance cannot be nil", "")
	}
	return instance.Validate()
}

// ValidateModel validates a Model struct
func ValidateModel(model *Model) error {
	if model == nil {
		return NewValidationError("model cannot be nil", "")
	}
	return model.Validate()
}

// ValidateTag validates a Tag struct
func ValidateTag(tag *Tag) error {
	if tag == nil {
		return NewValidationError("tag cannot be nil", "")
	}
	return tag.Validate()
}

// ValidateLabel validates a Label struct
func ValidateLabel(label *Label) error {
	if label == nil {
		return NewValidationError("label cannot be nil", "")
	}
	return label.Validate()
}

// SanitizeString sanitizes a string by removing control characters but preserving tabs and newlines
func SanitizeString(s string) string {
	result := make([]rune, 0, len(s))
	for _, r := range s {
		// Keep printable characters (>= 32), space (32), tab (9), and newline (10)
		// Only remove DEL (127) and other control characters (< 32 except tab/newline)
		if r >= 32 && r != 127 {
			result = append(result, r)
		} else if r == 9 || r == 10 {
			// Preserve tab (9) and newline (10)
			result = append(result, r)
		}
	}
	return string(result)
}

// ValidatePath validates a file path
func ValidatePath(path string) error {
	if path == "" {
		return fmt.Errorf("path cannot be empty")
	}
	// Check for path traversal attempts
	for i := 0; i < len(path)-1; i++ {
		if path[i] == '.' && path[i+1] == '.' {
			return fmt.Errorf("path contains '..' (possible path traversal)")
		}
	}
	return nil
}
