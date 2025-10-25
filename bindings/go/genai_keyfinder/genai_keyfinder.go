package genai_keyfinder

/*
#cgo LDFLAGS: -L../../../target/release -lgenai_keyfinder_ffi
#cgo darwin LDFLAGS: -Wl,-rpath,../../../target/release
#cgo linux LDFLAGS: -Wl,-rpath,../../../target/release
#include <stdlib.h>
#include "../../../ffi/include/genai_keyfinder.h"
*/
import "C"
import (
	"encoding/json"
	"errors"
	"unsafe"
)

// ScanOptions contains options for scanning
type ScanOptions struct {
	HomeDir           string   `json:"home_dir,omitempty"`
	IncludeFullValues bool     `json:"include_full_values"`
	MaxFileSize       int      `json:"max_file_size"`
	OnlyProviders     []string `json:"only_providers,omitempty"`
	ExcludeProviders  []string `json:"exclude_providers,omitempty"`
}

// DiscoveredKey represents a discovered API key
type DiscoveredKey struct {
	Provider   string  `json:"provider"`
	Source     string  `json:"source"`
	ValueType  string  `json:"value_type"`
	Value      string  `json:"value,omitempty"`
	Confidence float64 `json:"confidence"`
	Hash       string  `json:"hash"`
	Redacted   string  `json:"redacted"`
	Locked     bool    `json:"locked"`
}

// ConfigInstance represents an application configuration instance
type ConfigInstance struct {
	InstanceID   string            `json:"instance_id"`
	AppName      string            `json:"app_name"`
	ConfigPath   string            `json:"config_path"`
	DiscoveredAt string            `json:"discovered_at"`
	Keys         []DiscoveredKey   `json:"keys"`
	Metadata     map[string]string `json:"metadata"`
}

// ScanResult contains the results of a scan
type ScanResult struct {
	Keys             []DiscoveredKey  `json:"keys"`
	ConfigInstances  []ConfigInstance `json:"config_instances"`
	HomeDir          string           `json:"home_directory"`
	ScannedAt        string           `json:"scan_started_at"`
	ProvidersScanned []string         `json:"providers_scanned"`
}

// Scan performs a scan for GenAI credentials and configurations
func Scan(options ScanOptions) (*ScanResult, error) {
	// Convert options to JSON
	optionsJSON, err := json.Marshal(options)
	if err != nil {
		return nil, err
	}

	// Convert home directory to C string
	var homeDir *C.char
	if options.HomeDir != "" {
		homeDir = C.CString(options.HomeDir)
		defer C.free(unsafe.Pointer(homeDir))
	}

	// Convert options JSON to C string
	optionsStr := C.CString(string(optionsJSON))
	defer C.free(unsafe.Pointer(optionsStr))

	// Call C function
	resultPtr := C.keyfinder_scan(homeDir, optionsStr)
	if resultPtr == nil {
		// Get error message
		errPtr := C.keyfinder_last_error()
		if errPtr != nil {
			errMsg := C.GoString(errPtr)
			return nil, errors.New(errMsg)
		}
		return nil, errors.New("scan failed with unknown error")
	}
	defer C.keyfinder_free(resultPtr)

	// Convert result to Go string
	resultJSON := C.GoString(resultPtr)

	// Parse JSON result
	var result ScanResult
	if err := json.Unmarshal([]byte(resultJSON), &result); err != nil {
		return nil, err
	}

	return &result, nil
}

// Version returns the library version
func Version() string {
	versionPtr := C.keyfinder_version()
	return C.GoString(versionPtr)
}

// ListProviders returns a list of available provider plugins
func ListProviders() []string {
	return []string{
		"openai",
		"anthropic",
		"huggingface",
		"ollama",
		"langchain",
		"litellm",
	}
}

// ListScanners returns a list of available application scanners
func ListScanners() []string {
	return []string{
		"roo-code",
		"claude-desktop",
		"ragit",
		"langchain-app",
	}
}
