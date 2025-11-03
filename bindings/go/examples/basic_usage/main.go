package main

import (
	"encoding/json"
	"fmt"
	"log"
	"os"

	aicred "github.com/robottwo/aicred/bindings/go/aicred"
)

func main() {
	fmt.Println("AICred - Go Example")
	fmt.Printf("Version: %s\n\n", aicred.Version())

	// List available providers
	fmt.Println("Available Providers:")
	for _, provider := range aicred.ListProviders() {
		fmt.Printf("  - %s\n", provider)
	}

	fmt.Println("\nAvailable Scanners:")
	for _, scanner := range aicred.ListScanners() {
		fmt.Printf("  - %s\n", scanner)
	}

	// Perform scan
	fmt.Println("\nScanning for credentials...")
	options := aicred.ScanOptions{
		HomeDir:           ".",   // Use current directory
		IncludeFullValues: false, // Keep secrets redacted
		OnlyProviders:     []string{"openai", "anthropic"},
	}

	result, err := aicred.Scan(options)
	if err != nil {
		log.Fatalf("Scan failed: %v", err)
	}

	// Display results
	fmt.Printf("\nFound %d keys\n", len(result.Keys))
	fmt.Printf("Found %d config instances\n", len(result.ConfigInstances))

	if len(result.Keys) > 0 {
		fmt.Println("\nDiscovered Keys:")
		for _, key := range result.Keys {
			fmt.Printf("  %s: %s (confidence: %s)\n",
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

	if err := os.WriteFile("scan_result.json", jsonData, 0600); err != nil {
		log.Fatalf("Failed to write file: %v", err)
	}

	fmt.Println("\nResults saved to scan_result.json")
}
