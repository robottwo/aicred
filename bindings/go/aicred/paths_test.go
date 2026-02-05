package aicred

import (
	"os"
	"path/filepath"
	"runtime"
	"testing"
)

func TestGetHomeDir(t *testing.T) {
	homeDir, err := GetHomeDir()
	if err != nil {
		t.Fatalf("GetHomeDir() error = %v", err)
	}

	if homeDir == "" {
		t.Error("HomeDir should not be empty")
	}

	// Verify it's an absolute path
	if !filepath.IsAbs(homeDir) {
		t.Errorf("HomeDir should be absolute path, got %s", homeDir)
	}
}

func TestGetConfigDir(t *testing.T) {
	configDir, err := GetConfigDir()
	if err != nil {
		t.Fatalf("GetConfigDir() error = %v", err)
	}

	if configDir == "" {
		t.Error("ConfigDir should not be empty")
	}

	// Verify it's an absolute path
	if !filepath.IsAbs(configDir) {
		t.Errorf("ConfigDir should be absolute path, got %s", configDir)
	}

	// Verify it contains the expected path components
	expectedSuffix := "aicred"
	if len(configDir) < len(expectedSuffix) || configDir[len(configDir)-len(expectedSuffix):] != expectedSuffix {
		t.Errorf("ConfigDir should end with 'aicred', got %s", configDir)
	}

	// Verify platform-specific path
	switch runtime.GOOS {
	case "windows":
		appData := os.Getenv("APPDATA")
		if appData != "" && !filepath.HasPrefix(configDir, appData) {
			t.Errorf("Windows: ConfigDir should be under APPDATA, got %s", configDir)
		}
	case "darwin":
		homeDir, _ := GetHomeDir()
		expected := filepath.Join(homeDir, "Library", "Application Support", "aicred")
		if configDir != expected {
			t.Errorf("macOS: ConfigDir should be %s, got %s", expected, configDir)
		}
	default:
		homeDir, _ := GetHomeDir()
		expected := filepath.Join(homeDir, ".config", "aicred")
		if configDir != expected {
			t.Errorf("Linux/Unix: ConfigDir should be %s, got %s", expected, configDir)
		}
	}
}

func TestPathExists(t *testing.T) {
	// Test with existing directory
	homeDir, _ := GetHomeDir()
	if !PathExists(homeDir) {
		t.Errorf("PathExists(%s) should return true", homeDir)
	}

	// Test with non-existent path
	nonExistent := filepath.Join(homeDir, ".this_should_not_exist_12345")
	if PathExists(nonExistent) {
		t.Errorf("PathExists(%s) should return false", nonExistent)
	}
}

func TestFileExists(t *testing.T) {
	// Create a temp file
	tmpFile := t.TempDir() + "/test_file.txt"
	if err := os.WriteFile(tmpFile, []byte("test"), 0644); err != nil {
		t.Fatalf("Failed to create temp file: %v", err)
	}

	if !FileExists(tmpFile) {
		t.Errorf("FileExists(%s) should return true", tmpFile)
	}

	// Test with directory
	tmpDir := t.TempDir()
	if FileExists(tmpDir) {
		t.Errorf("FileExists(%s) should return false for directory", tmpDir)
	}

	// Test with non-existent path
	nonExistent := tmpDir + "/nonexistent.txt"
	if FileExists(nonExistent) {
		t.Errorf("FileExists(%s) should return false", nonExistent)
	}
}

func TestDirExists(t *testing.T) {
	// Create a temp directory
	tmpDir := t.TempDir()
	if !DirExists(tmpDir) {
		t.Errorf("DirExists(%s) should return true", tmpDir)
	}

	// Test with file
	tmpFile := tmpDir + "/test_file.txt"
	if err := os.WriteFile(tmpFile, []byte("test"), 0644); err != nil {
		t.Fatalf("Failed to create temp file: %v", err)
	}

	if DirExists(tmpFile) {
		t.Errorf("DirExists(%s) should return false for file", tmpFile)
	}

	// Test with non-existent path
	nonExistent := tmpDir + "/nonexistent"
	if DirExists(nonExistent) {
		t.Errorf("DirExists(%s) should return false", nonExistent)
	}
}

func TestGetFileExtension(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"config.json", "json"},
		{"document.pdf", "pdf"},
		{"archive.tar.gz", "gz"}, // Gets last extension
		{"no_extension", ""},
		{"hidden/.gitignore", "gitignore"},
		{"./file.txt", "txt"},
	}

	for _, tt := range tests {
		t.Run(tt.input, func(t *testing.T) {
			result := GetFileExtension(tt.input)
			if result != tt.expected {
				t.Errorf("GetFileExtension(%q) = %q, want %q", tt.input, result, tt.expected)
			}
		})
	}
}

func TestHasExtension(t *testing.T) {
	tests := []struct {
		input     string
		extension string
		expected  bool
	}{
		{"config.json", "json", true},
		{"config.json", ".json", true},
		{"document.pdf", "pdf", true},
		{"document.pdf", "json", false},
		{"no_extension", "txt", false},
		{"archive.tar.gz", "gz", true},
		{"archive.tar.gz", "tar.gz", false}, // Only checks last extension
	}

	for _, tt := range tests {
		t.Run(tt.input+"_"+tt.extension, func(t *testing.T) {
			result := HasExtension(tt.input, tt.extension)
			if result != tt.expected {
				t.Errorf("HasExtension(%q, %q) = %v, want %v", tt.input, tt.extension, result, tt.expected)
			}
		})
	}
}

func TestIsPathAbsolute(t *testing.T) {
	tests := []struct {
		input    string
		expected bool
	}{
		{"/home/user/config.json", true},
		{"/absolute/path", true},
		{"relative/path", false},
		{"./relative/path", false},
		{"../parent/path", false},
		{"config.json", false},
	}

	// Windows-specific paths
	if runtime.GOOS == "windows" {
		tests = append(tests, []struct {
			input    string
			expected bool
		}{
			{"C:\\Users\\config.json", true},
			{"C:/Users/config.json", true},
			{"relative\\path", false},
		}...)
	}

	for _, tt := range tests {
		t.Run(tt.input, func(t *testing.T) {
			result := IsPathAbsolute(tt.input)
			if result != tt.expected {
				t.Errorf("IsPathAbsolute(%q) = %v, want %v", tt.input, result, tt.expected)
			}
		})
	}
}
