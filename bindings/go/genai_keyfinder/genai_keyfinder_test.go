package genai_keyfinder

import (
	"os"
	"sync"
	"testing"
)

func TestVersion(t *testing.T) {
	version := Version()
	if version == "" {
		t.Error("Version should not be empty")
	}
	t.Logf("Version: %s", version)
}

func TestListProviders(t *testing.T) {
	providers := ListProviders()
	if len(providers) == 0 {
		t.Error("Should have at least one provider")
	}

	expectedProviders := map[string]bool{
		"openai":      true,
		"anthropic":   true,
		"huggingface": true,
	}

	for provider := range expectedProviders {
		found := false
		for _, p := range providers {
			if p == provider {
				found = true
				break
			}
		}
		if !found {
			t.Errorf("Expected provider %s not found", provider)
		}
	}
}

func TestListScanners(t *testing.T) {
	scanners := ListScanners()
	if len(scanners) == 0 {
		t.Error("Should have at least one scanner")
	}

	expectedScanners := map[string]bool{
		"roo-code":       true,
		"claude-desktop": true,
	}

	for scanner := range expectedScanners {
		found := false
		for _, s := range scanners {
			if s == scanner {
				found = true
				break
			}
		}
		if !found {
			t.Errorf("Expected scanner %s not found", scanner)
		}
	}
}

func TestScanBasic(t *testing.T) {
	// Create temporary directory for testing
	tmpDir, err := os.MkdirTemp("", "genai-keyfinder-test-*")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	options := ScanOptions{
		HomeDir:           tmpDir,
		IncludeFullValues: false,
		MaxFileSize:       1048576,
	}

	result, err := Scan(options)
	if err != nil {
		t.Fatalf("Scan failed: %v", err)
	}

	if result == nil {
		t.Fatal("Result should not be nil")
	}

	if result.Keys == nil {
		t.Error("Keys should not be nil")
	}

	if result.ConfigInstances == nil {
		t.Error("ConfigInstances should not be nil")
	}

	if result.HomeDir != tmpDir {
		t.Errorf("HomeDir should be %s, got %s", tmpDir, result.HomeDir)
	}

	if result.ScannedAt == "" {
		t.Error("ScannedAt should not be empty")
	}
}

func TestScanWithOptions(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "genai-keyfinder-test-*")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	options := ScanOptions{
		HomeDir:          tmpDir,
		MaxFileSize:      512000,
		OnlyProviders:    []string{"openai", "anthropic"},
		ExcludeProviders: []string{},
	}

	result, err := Scan(options)
	if err != nil {
		t.Fatalf("Scan with options failed: %v", err)
	}

	if result == nil {
		t.Fatal("Result should not be nil")
	}
}

func TestScanWithExclude(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "genai-keyfinder-test-*")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	options := ScanOptions{
		HomeDir:          tmpDir,
		ExcludeProviders: []string{"ollama"},
	}

	result, err := Scan(options)
	if err != nil {
		t.Fatalf("Scan with exclude failed: %v", err)
	}

	if result == nil {
		t.Fatal("Result should not be nil")
	}
}

func TestScanInvalidHome(t *testing.T) {
	options := ScanOptions{
		HomeDir: "/nonexistent/path/that/does/not/exist",
	}

	_, err := Scan(options)
	if err == nil {
		t.Error("Expected error for invalid home directory")
	}
}

func TestScanWithFullValues(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "genai-keyfinder-test-*")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	options := ScanOptions{
		HomeDir:           tmpDir,
		IncludeFullValues: true,
	}
	result, err := Scan(options)
	if err != nil {
		t.Fatalf("Scan with full values failed: %v", err)
	}
	if result == nil {
		t.Fatal("Result should not be nil")
	}
}

func TestConcurrentScans(t *testing.T) {
	// Create temporary directory for concurrent testing
	tmpDir, err := os.MkdirTemp("", "genai-keyfinder-concurrent-test-*")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	var wg sync.WaitGroup
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			_, _ = Scan(ScanOptions{HomeDir: tmpDir})
		}()
	}
	wg.Wait()
}
