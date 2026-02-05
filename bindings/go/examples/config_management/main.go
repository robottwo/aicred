// Example: Config Management with AICred Go Adapter
//
// This example demonstrates how to use the AICred Go adapter to manage
// provider instances, labels, and tags.
//
// NOTE: This is a THIN adapter - all business logic and file I/O happens in Rust.
// The Go code is just a CGO wrapper around Rust FFI functions.

package main

import (
	"fmt"
	"log"
	"os"

	aicred "github.com/robottwo/aicred/bindings/go/aicred"
)

func main() {
	// Get home directory (or use empty string for default)
	homeDir := os.Getenv("HOME")

	fmt.Println("=== AICred Go Adapter - Config Management Example ===\n")

	// 1. Load existing instances
	fmt.Println("1. Loading existing provider instances...")
	instances, err := aicred.LoadInstances(homeDir)
	if err != nil {
		log.Printf("Warning: Failed to load instances: %v", err)
		instances = []aicred.ProviderInstance{}
	}
	fmt.Printf("   Found %d existing instances\n", len(instances))

	// 2. Add a new instance if none exist
	if len(instances) == 0 {
		fmt.Println("\n2. Creating a new test instance...")
		newInstance := aicred.ProviderInstance{
			ID:           "test-example",
			DisplayName:  "Example OpenAI",
			ProviderType: "openai",
			BaseURL:      "https://api.openai.com/v1",
			APIKey:       "sk-test-key-placeholder",
			Active:       true,
			Models: []aicred.Model{
				{ModelID: "gpt-4", Name: "GPT-4"},
				{ModelID: "gpt-3.5-turbo", Name: "GPT-3.5 Turbo"},
			},
		}

		err = aicred.SaveInstances(homeDir, []aicred.ProviderInstance{newInstance})
		if err != nil {
			log.Fatalf("Failed to save instance: %v", err)
		}
		fmt.Println("   Created test instance successfully")
	}

	// 3. Load instances again and display
	fmt.Println("\n3. Loading and displaying instances...")
	instances, err = aicred.LoadInstances(homeDir)
	if err != nil {
		log.Fatalf("Failed to load instances: %v", err)
	}

	for i, inst := range instances {
		fmt.Printf("\n   Instance %d:\n", i+1)
		fmt.Printf("     ID: %s\n", inst.ID)
		fmt.Printf("     Name: %s\n", inst.DisplayName)
		fmt.Printf("     Provider: %s\n", inst.ProviderType)
		fmt.Printf("     Base URL: %s\n", inst.BaseURL)
		fmt.Printf("     Active: %t\n", inst.Active)
		if len(inst.Models) > 0 {
			fmt.Printf("     Models (%d):\n", len(inst.Models))
			for _, m := range inst.Models {
				fmt.Printf("       - %s (%s)\n", m.Name, m.ModelID)
			}
		}
	}

	// 4. Get a specific instance by ID
	if len(instances) > 0 {
		fmt.Printf("\n4. Retrieving instance by ID (%s)...\n", instances[0].ID)
		specific, err := aicred.GetInstance(homeDir, instances[0].ID)
		if err != nil {
			log.Printf("Warning: Failed to get instance: %v", err)
		} else {
			fmt.Printf("   Retrieved: %s (%s)\n", specific.DisplayName, specific.ProviderType)
		}
	}

	// 5. Load existing labels
	fmt.Println("\n5. Loading existing labels...")
	labels, err := aicred.LoadLabels(homeDir)
	if err != nil {
		log.Printf("Warning: Failed to load labels: %v", err)
		labels = []aicred.Label{}
	}
	fmt.Printf("   Found %d existing labels\n", len(labels))

	// 6. Add a new label if none exist
	if len(labels) == 0 && len(instances) > 0 {
		fmt.Println("\n6. Creating a new test label...")
		newLabel := aicred.Label{
			Name:        "fast",
			Description: "Fast models for quick tasks",
			Assignments: []aicred.Assignment{
				{InstanceID: instances[0].ID, ModelID: "gpt-3.5-turbo"},
			},
		}

		err = aicred.SaveLabels(homeDir, []aicred.Label{newLabel})
		if err != nil {
			log.Fatalf("Failed to save label: %v", err)
		}
		fmt.Println("   Created test label successfully")
	}

	// 7. Load labels again and display
	fmt.Println("\n7. Loading and displaying labels...")
	labels, err = aicred.LoadLabels(homeDir)
	if err != nil {
		log.Fatalf("Failed to load labels: %v", err)
	}

	for i, label := range labels {
		fmt.Printf("\n   Label %d:\n", i+1)
		fmt.Printf("     Name: %s\n", label.Name)
		fmt.Printf("     Description: %s\n", label.Description)
		if len(label.Assignments) > 0 {
			fmt.Printf("     Assignments (%d):\n", len(label.Assignments))
			for _, a := range label.Assignments {
				fmt.Printf("       - Instance: %s, Model: %s\n", a.InstanceID, a.ModelID)
			}
		}
	}

	// 8. Load existing tags
	fmt.Println("\n8. Loading existing tags...")
	tags, err := aicred.LoadTags(homeDir)
	if err != nil {
		log.Printf("Warning: Failed to load tags: %v", err)
		tags = []aicred.TagAssignment{}
	}
	fmt.Printf("   Found %d existing tags\n", len(tags))

	// 9. Add a new tag if none exist
	if len(tags) == 0 && len(instances) > 0 {
		fmt.Println("\n9. Creating a new test tag...")
		newTag := aicred.TagAssignment{
			TagID:   "tag-test",
			Name:    "Test",
			Target:  fmt.Sprintf("instance:%s", instances[0].ID),
			TagType: "instance",
		}

		err = aicred.SaveTags(homeDir, []aicred.TagAssignment{newTag})
		if err != nil {
			log.Fatalf("Failed to save tag: %v", err)
		}
		fmt.Println("   Created test tag successfully")
	}

	// 10. Load tags again and display
	fmt.Println("\n10. Loading and displaying tags...")
	tags, err = aicred.LoadTags(homeDir)
	if err != nil {
		log.Fatalf("Failed to load tags: %v", err)
	}

	for i, tag := range tags {
		fmt.Printf("\n   Tag %d:\n", i+1)
		fmt.Printf("     ID: %s\n", tag.TagID)
		fmt.Printf("     Name: %s\n", tag.Name)
		fmt.Printf("     Target: %s\n", tag.Target)
		fmt.Printf("     Type: %s\n", tag.TagType)
	}

	fmt.Println("\n=== Example Complete ===")
	fmt.Println("\nNote: This is a THIN adapter - the Go code is just a wrapper.")
	fmt.Println("All business logic (validation, YAML parsing, file I/O) happens in Rust.")
}
