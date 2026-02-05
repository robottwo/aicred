package main

import (
	"fmt"
	"log"

	"github.com/robottwo/aicred/bindings/go/aicred"
)

func main() {
	fmt.Println("=== Tag and Label Management Example ===\n")

	// Create a tag repository
	tagRepo := aicred.NewTagRepository()

	// Create tags
	production := aicred.NewTag("prod", "Production").
		WithDescription("Production environment").
		WithColor("#FF0000")
	tagRepo.AddTag(production)

	development := aicred.NewTag("dev", "Development").
		WithDescription("Development environment").
		WithColor("#00FF00")
	tagRepo.AddTag(development)

	testing := aicred.NewTag("test", "Testing").
		WithDescription("Testing environment").
		WithColor("#FFFF00")
	tagRepo.AddTag(testing)

	// Assign tags to targets
	tagRepo.Assign(aicred.NewTagAssignment("assign-1", "prod", "provider_instance", "openai-prod", ""))
	tagRepo.Assign(aicred.NewTagAssignment("assign-2", "prod", "provider_instance", "anthropic-prod", ""))
	tagRepo.Assign(aicred.NewTagAssignment("assign-3", "dev", "provider_instance", "openai-dev", ""))
	tagRepo.Assign(aicred.NewTagAssignment("assign-4", "test", "provider_instance", "cohere-test", ""))

	// Print tags
	fmt.Println("Tags:")
	for _, tag := range tagRepo.ListTags() {
		fmt.Printf("  - %s (%s)\n", tag.Name, tag.ID)
		if tag.Description != nil {
			fmt.Printf("    Description: %s\n", *tag.Description)
		}
		if tag.Color != nil {
			fmt.Printf("    Color: %s\n", *tag.Color)
		}
		fmt.Println()
	}

	// Get tags for a specific target
	fmt.Println("Tags for 'openai-prod':")
	prodTags := tagRepo.GetTagsForTarget("provider_instance", "openai-prod", "")
	for _, tag := range prodTags {
		fmt.Printf("  - %s\n", tag.Name)
	}
	fmt.Println()

	// Create a label repository (labels are unique)
	labelRepo := aicred.NewLabelRepository()

	// Create labels
	primary := aicred.NewLabel("primary", "Primary").
		WithDescription("Primary instance").
		WithColor("#0000FF")
	labelRepo.AddLabel(primary)

	backup := aicred.NewLabel("backup", "Backup").
		WithDescription("Backup instance").
		WithColor("#808080")
	labelRepo.AddLabel(backup)

	deprecated := aicred.NewLabel("deprecated", "Deprecated").
		WithDescription("Deprecated instance").
		WithColor("#FF00FF")
	labelRepo.AddLabel(deprecated)

	// Assign labels to targets (enforces uniqueness)
	labelRepo.Assign(aicred.NewLabelAssignment("l-assign-1", "primary", "provider_instance", "openai-prod", ""))
	labelRepo.Assign(aicred.NewLabelAssignment("l-assign-2", "backup", "provider_instance", "anthropic-prod", ""))
	labelRepo.Assign(aicred.NewLabelAssignment("l-assign-3", "deprecated", "provider_instance", "legacy", ""))

	// Print labels
	fmt.Println("Labels:")
	for _, label := range labelRepo.ListLabels() {
		fmt.Printf("  - %s (%s)\n", label.Name, label.ID)
		if label.Description != nil {
			fmt.Printf("    Description: %s\n", *label.Description)
		}
		if label.Color != nil {
			fmt.Printf("    Color: %s\n", *label.Color)
		}
		fmt.Printf("    Uniqueness Scope: %s\n", label.UniquenessScope())
		fmt.Println()
	}

	// Get label for a specific target
	fmt.Println("Label for 'openai-prod':")
	prodLabel, err := labelRepo.GetLabelForTarget("provider_instance", "openai-prod", "")
	if err == nil {
		fmt.Printf("  - %s\n", prodLabel.Name)
		fmt.Println()
	}

	// Try to assign the same label to a different target (should fail)
	fmt.Println("Attempting to assign 'primary' label to another target...")
	err = labelRepo.Assign(aicred.NewLabelAssignment("l-assign-4", "primary", "provider_instance", "another-instance", ""))
	if err != nil {
		fmt.Printf("  Error: %v (expected, labels are unique)\n", err)
	}
	fmt.Println()

	// Print repository statistics
	fmt.Println("Repository Statistics:")
	fmt.Printf("  Tags: %d\n", tagRepo.TagCount())
	fmt.Printf("  Tag Assignments: %d\n", tagRepo.AssignmentCount())
	fmt.Printf("  Labels: %d\n", labelRepo.LabelCount())
	fmt.Printf("  Label Assignments: %d\n", labelRepo.AssignmentCount())

	// Remove a tag and its assignments
	fmt.Println("\nRemoving 'test' tag...")
	tagRepo.RemoveTag("test")
	fmt.Printf("  Tags: %d\n", tagRepo.TagCount())
	fmt.Printf("  Tag Assignments: %d\n", tagRepo.AssignmentCount())

	// Verify 'cohere-test' no longer has the 'test' tag
	testTags := tagRepo.GetTagsForTarget("provider_instance", "cohere-test", "")
	fmt.Printf("  Tags for 'cohere-test': %d\n", len(testTags))
}
