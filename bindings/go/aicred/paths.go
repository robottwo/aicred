package aicred

import (
	"os"
	"path/filepath"
	"runtime"
)

// GetHomeDir returns the user's home directory
func GetHomeDir() (string, error) {
	return os.UserHomeDir()
}

// GetConfigDir returns the aicred configuration directory
func GetConfigDir() (string, error) {
	homeDir, err := GetHomeDir()
	if err != nil {
		return "", err
	}

	var configDir string
	switch runtime.GOOS {
	case "windows":
		appData := os.Getenv("APPDATA")
		if appData == "" {
			return homeDir, nil
		}
		configDir = filepath.Join(appData, "aicred")
	case "darwin":
		configDir = filepath.Join(homeDir, "Library", "Application Support", "aicred")
	default:
		configDir = filepath.Join(homeDir, ".config", "aicred")
	}

	return configDir, nil
}

// PathExists checks if a path exists
func PathExists(path string) bool {
	_, err := os.Stat(path)
	return err == nil
}

// FileExists checks if a path is a regular file
func FileExists(path string) bool {
	info, err := os.Stat(path)
	if err != nil {
		return false
	}
	return !info.IsDir()
}

// DirExists checks if a path is a directory
func DirExists(path string) bool {
	info, err := os.Stat(path)
	if err != nil {
		return false
	}
	return info.IsDir()
}

// GetFileExtension returns the file extension (without dot)
func GetFileExtension(path string) string {
	ext := filepath.Ext(path)
	if len(ext) > 0 {
		return ext[1:]
	}
	return ""
}

// HasExtension checks if a path has a specific extension
func HasExtension(path, extension string) bool {
	if !filepath.HasPrefix(extension, ".") {
		extension = "." + extension
	}
	return filepath.Ext(path) == extension
}

// IsPathAbsolute checks if a path is absolute
func IsPathAbsolute(path string) bool {
	return filepath.IsAbs(path)
}
