// Package aicred provides a thin Go adapter layer for aicred.
//
// This is a THIN WRAPPER around Rust FFI functions. All business logic,
// validation, file I/O, and YAML parsing happen in Rust.
//
// The Go layer is responsible for:
//   - Struct definitions that mirror Rust types
//   - CGO wrapper functions
//   - JSON serialization/deserialization
//   - Error handling
//
// For more details on the architecture, see ADAPTER_ARCHITECTURE.md

package aicred

/*
#cgo LDFLAGS: -L../../../target/release -laicred_ffi
#cgo darwin LDFLAGS: -Wl,-rpath,../../../target/release
#cgo linux LDFLAGS: -Wl,-rpath,../../../target/release
#cgo windows LDFLAGS: -lws2_32 -luserenv -ladvapi32 -lbcrypt -lntdll -lkernel32 -luser32
#include <stdlib.h>

// Declare the FFI functions for config management
extern char* aicred_load_instances(const char* home_dir);
extern char* aicred_save_instances(const char* home_dir, const char* instances_json);
extern char* aicred_get_instance(const char* home_dir, const char* instance_id);
extern char* aicred_load_labels(const char* home_dir);
extern char* aicred_save_labels(const char* home_dir, const char* labels_json);
extern char* aicred_load_tags(const char* home_dir);
extern char* aicred_save_tags(const char* home_dir, const char* tags_json);
*/
import "C"
import (
	"encoding/json"
	"errors"
	"fmt"
	"unsafe"
)

// ProviderInstance represents a provider configuration instance
// This mirrors the Rust ProviderInstance type
type ProviderInstance struct {
	ID           string   `json:"id"`
	DisplayName  string   `json:"display_name"`
	ProviderType string   `json:"provider_type"`
	BaseURL      string   `json:"base_url"`
	APIKey       string   `json:"api_key,omitempty"`
	Active       bool     `json:"active"`
	Models       []Model  `json:"models,omitempty"`
	CreatedAt    string   `json:"created_at,omitempty"`
	UpdatedAt    string   `json:"updated_at,omitempty"`
}

// Model represents a model available from a provider
// This mirrors the Rust Model type
type Model struct {
	ModelID  string `json:"model_id"`
	Name     string `json:"name"`
}

// Label represents a semantic label for provider:model combinations
// This mirrors the Rust UnifiedLabel type
type Label struct {
	Name        string       `json:"name"`
	Description string       `json:"description"`
	Assignments []Assignment `json:"assignments"`
}

// Assignment represents a label assignment to a provider:model combination
type Assignment struct {
	InstanceID string `json:"instance_id"`
	ModelID    string `json:"model_id"`
}

// TagAssignment represents a tag assignment
// This mirrors the Rust TagAssignment type
type TagAssignment struct {
	TagID   string `json:"tag_id"`
	Name    string `json:"name"`
	Target  string `json:"target"`
	TagType string `json:"tag_type"`
}

// LoadInstances loads all provider instances from the configuration directory.
// This is a thin wrapper - Rust handles all file I/O and YAML parsing.
// Pass empty string for homeDir to use the default home directory.
func LoadInstances(homeDir string) ([]ProviderInstance, error) {
	var homeDirC *C.char
	if homeDir != "" {
		homeDirC = C.CString(homeDir)
	} else {
		homeDirC = C.CString("")
	}
	defer C.free(unsafe.Pointer(homeDirC))

	resultPtr := C.aicred_load_instances(homeDirC)
	if resultPtr == nil {
		errPtr := C.aicred_last_error()
		if errPtr != nil {
			errMsg := C.GoString(errPtr)
			return nil, fmt.Errorf("FFI load instances failed: %s", errMsg)
		}
		return nil, errors.New("load instances failed with unknown error (FFI returned null)")
	}
	defer C.aicred_free(resultPtr)

	resultJSON := C.GoString(resultPtr)
	if resultJSON == "" {
		return []ProviderInstance{}, nil
	}

	var instances []ProviderInstance
	if err := json.Unmarshal([]byte(resultJSON), &instances); err != nil {
		return nil, fmt.Errorf("failed to parse instances JSON: %v", err)
	}

	return instances, nil
}

// SaveInstances saves all provider instances to the configuration directory.
// This is a thin wrapper - Rust handles all file I/O and YAML serialization.
// Pass empty string for homeDir to use the default home directory.
func SaveInstances(homeDir string, instances []ProviderInstance) error {
	instancesJSON, err := json.Marshal(instances)
	if err != nil {
		return fmt.Errorf("failed to marshal instances: %v", err)
	}

	var homeDirC *C.char
	if homeDir != "" {
		homeDirC = C.CString(homeDir)
	} else {
		homeDirC = C.CString("")
	}
	defer C.free(unsafe.Pointer(homeDirC))

	instancesStr := C.CString(string(instancesJSON))
	defer C.free(unsafe.Pointer(instancesStr))

	success := C.aicred_save_instances(homeDirC, instancesStr)
	if !success {
		errPtr := C.aicred_last_error()
		if errPtr != nil {
			errMsg := C.GoString(errPtr)
			return fmt.Errorf("FFI save instances failed: %s", errMsg)
		}
		return errors.New("save instances failed with unknown error")
	}

	return nil
}

// GetInstance retrieves a specific provider instance by ID.
// This is a thin wrapper - Rust handles file I/O and YAML parsing.
// Pass empty string for homeDir to use the default home directory.
func GetInstance(homeDir, instanceID string) (*ProviderInstance, error) {
	var homeDirC *C.char
	if homeDir != "" {
		homeDirC = C.CString(homeDir)
	} else {
		homeDirC = C.CString("")
	}
	defer C.free(unsafe.Pointer(homeDirC))

	instanceIDC := C.CString(instanceID)
	defer C.free(unsafe.Pointer(instanceIDC))

	resultPtr := C.aicred_get_instance(homeDirC, instanceIDC)
	if resultPtr == nil {
		errPtr := C.aicred_last_error()
		if errPtr != nil {
			errMsg := C.GoString(errPtr)
			return nil, fmt.Errorf("FFI get instance failed: %s", errMsg)
		}
		return nil, errors.New("get instance failed: instance not found or unknown error")
	}
	defer C.aicred_free(resultPtr)

	resultJSON := C.GoString(resultPtr)
	if resultJSON == "" {
		return nil, errors.New("get instance failed: empty result")
	}

	var instance ProviderInstance
	if err := json.Unmarshal([]byte(resultJSON), &instance); err != nil {
		return nil, fmt.Errorf("failed to parse instance JSON: %v", err)
	}

	return &instance, nil
}

// LoadLabels loads all label assignments from the configuration directory.
// This is a thin wrapper - Rust handles all file I/O and YAML parsing.
// Pass empty string for homeDir to use the default home directory.
func LoadLabels(homeDir string) ([]Label, error) {
	var homeDirC *C.char
	if homeDir != "" {
		homeDirC = C.CString(homeDir)
	} else {
		homeDirC = C.CString("")
	}
	defer C.free(unsafe.Pointer(homeDirC))

	resultPtr := C.aicred_load_labels(homeDirC)
	if resultPtr == nil {
		errPtr := C.aicred_last_error()
		if errPtr != nil {
			errMsg := C.GoString(errPtr)
			return nil, fmt.Errorf("FFI load labels failed: %s", errMsg)
		}
		return nil, errors.New("load labels failed with unknown error (FFI returned null)")
	}
	defer C.aicred_free(resultPtr)

	resultJSON := C.GoString(resultPtr)
	if resultJSON == "" {
		return []Label{}, nil
	}

	var labels []Label
	if err := json.Unmarshal([]byte(resultJSON), &labels); err != nil {
		return nil, fmt.Errorf("failed to parse labels JSON: %v", err)
	}

	return labels, nil
}

// SaveLabels saves all label assignments to the configuration directory.
// This is a thin wrapper - Rust handles all file I/O and YAML serialization.
// Pass empty string for homeDir to use the default home directory.
func SaveLabels(homeDir string, labels []Label) error {
	labelsJSON, err := json.Marshal(labels)
	if err != nil {
		return fmt.Errorf("failed to marshal labels: %v", err)
	}

	var homeDirC *C.char
	if homeDir != "" {
		homeDirC = C.CString(homeDir)
	} else {
		homeDirC = C.CString("")
	}
	defer C.free(unsafe.Pointer(homeDirC))

	labelsStr := C.CString(string(labelsJSON))
	defer C.free(unsafe.Pointer(labelsStr))

	success := C.aicred_save_labels(homeDirC, labelsStr)
	if !success {
		errPtr := C.aicred_last_error()
		if errPtr != nil {
			errMsg := C.GoString(errPtr)
			return fmt.Errorf("FFI save labels failed: %s", errMsg)
		}
		return errors.New("save labels failed with unknown error")
	}

	return nil
}

// LoadTags loads all tag assignments from the configuration directory.
// This is a thin wrapper - Rust handles all file I/O and YAML parsing.
// Pass empty string for homeDir to use the default home directory.
func LoadTags(homeDir string) ([]TagAssignment, error) {
	var homeDirC *C.char
	if homeDir != "" {
		homeDirC = C.CString(homeDir)
	} else {
		homeDirC = C.CString("")
	}
	defer C.free(unsafe.Pointer(homeDirC))

	resultPtr := C.aicred_load_tags(homeDirC)
	if resultPtr == nil {
		errPtr := C.aicred_last_error()
		if errPtr != nil {
			errMsg := C.GoString(errPtr)
			return nil, fmt.Errorf("FFI load tags failed: %s", errMsg)
		}
		return nil, errors.New("load tags failed with unknown error (FFI returned null)")
	}
	defer C.aicred_free(resultPtr)

	resultJSON := C.GoString(resultPtr)
	if resultJSON == "" {
		return []TagAssignment{}, nil
	}

	var tags []TagAssignment
	if err := json.Unmarshal([]byte(resultJSON), &tags); err != nil {
		return nil, fmt.Errorf("failed to parse tags JSON: %v", err)
	}

	return tags, nil
}

// SaveTags saves all tag assignments to the configuration directory.
// This is a thin wrapper - Rust handles all file I/O and YAML serialization.
// Pass empty string for homeDir to use the default home directory.
func SaveTags(homeDir string, tags []TagAssignment) error {
	tagsJSON, err := json.Marshal(tags)
	if err != nil {
		return fmt.Errorf("failed to marshal tags: %v", err)
	}

	var homeDirC *C.char
	if homeDir != "" {
		homeDirC = C.CString(homeDir)
	} else {
		homeDirC = C.CString("")
	}
	defer C.free(unsafe.Pointer(homeDirC))

	tagsStr := C.CString(string(tagsJSON))
	defer C.free(unsafe.Pointer(tagsStr))

	success := C.aicred_save_tags(homeDirC, tagsStr)
	if !success {
		errPtr := C.aicred_last_error()
		if errMsg := C.GoString(errPtr); errMsg != "" {
			return fmt.Errorf("FFI save tags failed: %s", errMsg)
		}
		return errors.New("save tags failed with unknown error")
	}

	return nil
}
