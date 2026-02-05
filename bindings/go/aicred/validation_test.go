package aicred

import (
	"testing"
)

func TestValidateConfig(t *testing.T) {
	tests := []struct {
		name    string
		config  *Config
		wantErr bool
	}{
		{
			name:    "nil config",
			config:  nil,
			wantErr: true,
		},
		{
			name: "empty version",
			config: &Config{
				Version: "",
				HomeDir: "/home/user",
			},
			wantErr: true,
		},
		{
			name: "empty home dir",
			config: &Config{
				Version: "1.0",
				HomeDir: "",
			},
			wantErr: true,
		},
		{
			name: "valid config",
			config: &Config{
				Version: "1.0",
				HomeDir: "/home/user",
			},
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateConfig(tt.config)
			if (err != nil) != tt.wantErr {
				t.Errorf("ValidateConfig() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestValidateProviderInstance(t *testing.T) {
	tests := []struct {
		name     string
		instance *ProviderInstance
		wantErr  bool
	}{
		{
			name:     "nil instance",
			instance: nil,
			wantErr:  true,
		},
		{
			name: "valid instance",
			instance: &ProviderInstance{
				ID:           "test-1",
				ProviderType: "openai",
				BaseURL:      "https://api.openai.com",
			},
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateProviderInstance(tt.instance)
			if (err != nil) != tt.wantErr {
				t.Errorf("ValidateProviderInstance() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestValidateModel(t *testing.T) {
	tests := []struct {
		name    string
		model   *Model
		wantErr bool
	}{
		{
			name:    "nil model",
			model:   nil,
			wantErr: true,
		},
		{
			name: "valid model",
			model: &Model{
				ModelID: "gpt-4",
			},
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateModel(tt.model)
			if (err != nil) != tt.wantErr {
				t.Errorf("ValidateModel() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestValidateTag(t *testing.T) {
	tests := []struct {
		name    string
		tag     *Tag
		wantErr bool
	}{
		{
			name:    "nil tag",
			tag:     nil,
			wantErr: true,
		},
		{
			name: "valid tag",
			tag: &Tag{
				Name: "production",
			},
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateTag(tt.tag)
			if (err != nil) != tt.wantErr {
				t.Errorf("ValidateTag() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestValidateLabel(t *testing.T) {
	tests := []struct {
		name    string
		label   *Label
		wantErr bool
	}{
		{
			name:    "nil label",
			label:   nil,
			wantErr: true,
		},
		{
			name: "valid label",
			label: &Label{
				Key:   "env",
				Value: "prod",
			},
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateLabel(tt.label)
			if (err != nil) != tt.wantErr {
				t.Errorf("ValidateLabel() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestSanitizeString(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		expected string
	}{
		{
			name:     "no control characters",
			input:    "hello world",
			expected: "hello world",
		},
		{
			name:     "preserve tabs",
			input:    "hello\tworld",
			expected: "hello\tworld",
		},
		{
			name:     "preserve newlines",
			input:    "hello\nworld",
			expected: "hello\nworld",
		},
		{
			name:     "remove control characters",
			input:    "hello\x00\x01\x02world",
			expected: "helloworld",
		},
		{
			name:     "remove DEL character",
			input:    "hello\x7Fworld",
			expected: "helloworld",
		},
		{
			name:     "preserve tabs and newlines while removing others",
			input:    "hello\t\n\x00\x01world",
			expected: "hello\t\nworld",
		},
		{
			name:     "empty string",
			input:    "",
			expected: "",
		},
		{
			name:     "only control characters",
			input:    "\x00\x01\x02\x7F",
			expected: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := SanitizeString(tt.input)
			if result != tt.expected {
				t.Errorf("SanitizeString() = %q, want %q", result, tt.expected)
			}
		})
	}
}

func TestValidatePath(t *testing.T) {
	tests := []struct {
		name    string
		path    string
		wantErr bool
	}{
		{
			name:    "empty path",
			path:    "",
			wantErr: true,
		},
		{
			name:    "valid path",
			path:    "/home/user/file.txt",
			wantErr: false,
		},
		{
			name:    "path traversal attempt",
			path:    "../etc/passwd",
			wantErr: true,
		},
		{
			name:    "path traversal in middle",
			path:    "/home/../etc/passwd",
			wantErr: true,
		},
		{
			name:    "valid relative path",
			path:    "config/settings.yaml",
			wantErr: false,
		},
		{
			name:    "valid absolute path",
			path:    "/var/log/app.log",
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidatePath(tt.path)
			if (err != nil) != tt.wantErr {
				t.Errorf("ValidatePath() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}
