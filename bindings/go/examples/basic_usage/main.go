package main

import (
	"encoding/json"
	"fmt"
	"log"
	"os"

	"github.com/yourusername/genai-keyfinder/bindings/go/genai_keyfinder"
)

func main() {
	fmt.Println("GenAI Key Finder - Go Example")
	fmt.Printf("Version: %s\n\n", genai_keyfinder.Version())

	// List available providers
	fmt.Println("Available Providers:")
	for _, provider := range genai_keyfinder.ListProviders() {
		fmt.Printf("  - %s\n", provider)
	}

	fmt.Println("\nAvailable Scanners:")
	for _, scanner := range genai_keyfinder.ListScanners() {
		fmt.Printf("  - %s\n", scanner)
	}

	// Perform scan
	fmt.Println("\nScanning for credentials...")
	options := genai_keyfinder.ScanOptions{
		HomeDir:           ".",   // Use current directory
		IncludeFullValues: false, // Keep secrets redacted
		OnlyProviders:     []string{"openai", "anthropic"},
	}

	result, err := genai_keyfinder.Scan(options)
	if err != nil {
		log.Fatalf("Scan failed: %v", err)
	}

	// Display results
	fmt.Printf("\nFound %d keys\n", len(result.Keys))
	fmt.Printf("Found %d config instances\n", len(result.ConfigInstances))

	if len(result.Keys) > 0 {
		fmt.Println("\nDiscovered Keys:")
		for _, key := range result.Keys {
			fmt.Printf("  %s: %s (confidence: %.2f)\n",
				key.Provider, key.Redacted, key.Confidence)
		}
	}

	if len(result.ConfigInstances) > 0 {
		fmt.Println("\nConfig Instances:")
		for _, instance := range result.ConfigInstances {
			fmt.Printf("  %s: %s\n", instance.AppName, instance.ConfigPath)
		}
	}

	// Save to JSON
	jsonData, err := json.MarshalIndent(result, "", "  ")
	if err != nil {
		log.Fatalf("Failed to marshal JSON: %v", err)
	}

	if err := os.WriteFile("scan_result.json", jsonData, 0644); err != nil {
		log.Fatalf("Failed to write file: %v", err)
	}

	fmt.Println("\nResults saved to scan_result.json")
}
