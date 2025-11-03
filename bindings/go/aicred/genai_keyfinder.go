package aicred

/*
#cgo LDFLAGS: -L../../../target/release -laicred_ffi
#cgo darwin LDFLAGS: -Wl,-rpath,../../../target/release
#cgo linux LDFLAGS: -Wl,-rpath,../../../target/release
#cgo windows LDFLAGS: -lws2_32 -luserenv -ladvapi32 -lbcrypt -lntdll -lkernel32 -luser32
#include <stdlib.h>

// Declare the FFI functions that might not be in the header yet
extern char* aicred_list_providers();
extern char* aicred_list_scanners();
extern char* aicred_scan(const char* home_path, const char* options_json);
extern void aicred_free(char* ptr);
extern const char* aicred_version(void);
extern const char* aicred_last_error(void);

// Include the header for existing functions
#include "../../../ffi/include/genai_keyfinder.h"
*/
import "C"
import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
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
	Provider   string `json:"provider"`
	Source     string `json:"source"`
	ValueType  string `json:"value_type"`
	Value      string `json:"value,omitempty"`
	Confidence string `json:"confidence"`
	Hash       string `json:"hash"`
	Redacted   string `json:"redacted"`
	Locked     bool   `json:"locked"`
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
	// Validate HomeDir if provided
	if options.HomeDir != "" {
		info, err := os.Stat(options.HomeDir)
		if err != nil || !info.IsDir() {
			return nil, fmt.Errorf("invalid HomeDir: %s", options.HomeDir)
		}
	}

	// Convert options to JSON
	optionsJSON, err := json.Marshal(options)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal options to JSON: %v", err)
	}

	// Convert home directory to C string
	var homeDir *C.char
	if options.HomeDir != "" {
		homeDir = C.CString(options.HomeDir)
	} else {
		homeDir = C.CString("")
	}
	defer C.free(unsafe.Pointer(homeDir))

	// Convert options JSON to C string
	optionsStr := C.CString(string(optionsJSON))
	defer C.free(unsafe.Pointer(optionsStr))

	// Call C function with error handling
	resultPtr := C.aicred_scan(homeDir, optionsStr)
	if resultPtr == nil {
		// Get error message
		errPtr := C.aicred_last_error()
		if errPtr != nil {
			errMsg := C.GoString(errPtr)
			return nil, fmt.Errorf("FFI scan failed: %s", errMsg)
		}
		return nil, errors.New("scan failed with unknown error (FFI returned null)")
	}
	defer C.aicred_free(resultPtr)

	// Convert result to Go string
	resultJSON := C.GoString(resultPtr)
	if resultJSON == "" {
		return nil, errors.New("FFI returned empty result")
	}

	// Parse JSON result
	var result ScanResult
	if err := json.Unmarshal([]byte(resultJSON), &result); err != nil {
		return nil, fmt.Errorf("failed to parse JSON result: %v (raw: %s)", err, resultJSON)
	}

	return &result, nil
}

// Version returns the library version
func Version() string {
	versionPtr := C.aicred_version()
	return C.GoString(versionPtr)
}

// ListProviders returns a list of available provider plugins
func ListProviders() []string {
	// Call the FFI function to get the list of providers
	providersPtr := C.aicred_list_providers()
	if providersPtr == nil {
		// If FFI is not available, return empty slice to avoid misleading consumers
		return []string{}
	}
	defer C.aicred_free(providersPtr)

	// Convert C string to Go string
	providersJSON := C.GoString(providersPtr)

	// Parse JSON array
	var providers []string
	if err := json.Unmarshal([]byte(providersJSON), &providers); err != nil {
		// If parsing fails, return empty slice
		return []string{}
	}

	return providers
}

// ListScanners returns a list of available application scanners
func ListScanners() []string {
	// Call the FFI function to get the list of scanners
	scannersPtr := C.aicred_list_scanners()
	if scannersPtr == nil {
		// If FFI is not available, return empty slice to avoid misleading consumers
		return []string{}
	}
	defer C.aicred_free(scannersPtr)

	// Convert C string to Go string
	scannersJSON := C.GoString(scannersPtr)

	// Parse JSON array
	var scanners []string
	if err := json.Unmarshal([]byte(scannersJSON), &scanners); err != nil {
		// If parsing fails, return empty slice
		return []string{}
	}

	return scanners
}
