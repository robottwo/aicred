#!/usr/bin/env python3
"""
AICred Configuration Validation Utility

This script validates AICred configuration files for integrity, consistency, and compatibility.
It checks both old and new configuration formats and provides detailed feedback.

Usage:
    python validate_config.py --config-dir <path>
    python validate_config.py --check-tags
    python validate_config.py --check-labels
    python validate_config.py --check-assignments
"""

import os
import sys
import json
import yaml
import argparse
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any, Optional, Set, Tuple


class ConfigValidator:
    """Validates AICred configuration files."""
    
    def __init__(self, config_dir: Optional[Path] = None):
        self.config_dir = config_dir or Path.home() / ".config" / "aicred"
        self.errors = []
        self.warnings = []
        self.info = []
        
    def validate_all(self) -> Dict[str, Any]:
        """Run all validation checks."""
        result = {
            "valid": True,
            "errors": [],
            "warnings": [],
            "info": [],
            "checks": {}
        }
        
        # Check configuration directory
        self._check_config_directory()
        
        # Validate provider instances
        self._validate_provider_instances()
        
        # Validate tags
        self._validate_tags()
        
        # Validate labels
        self._validate_labels()
        
        # Validate assignments
        self._validate_assignments()
        
        # Check referential integrity
        self._check_referential_integrity()
        
        # Check file permissions
        self._check_file_permissions()
        
        result["errors"] = self.errors
        result["warnings"] = self.warnings
        result["info"] = self.info
        result["valid"] = len(self.errors) == 0
        
        return result
        
    def _check_config_directory(self):
        """Check configuration directory structure."""
        if not self.config_dir.exists():
            self.warnings.append(f"Configuration directory does not exist: {self.config_dir}")
            return
            
        if not self.config_dir.is_dir():
            self.errors.append(f"Configuration path is not a directory: {self.config_dir}")
            return
            
        self.info.append(f"Configuration directory found: {self.config_dir}")
        
    def _validate_provider_instances(self):
        """Validate provider instances configuration."""
        instances_file = self.config_dir / "provider_instances.yaml"
        
        if not instances_file.exists():
            self.warnings.append("provider_instances.yaml not found")
            return
            
        try:
            with open(instances_file, 'r') as f:
                instances = yaml.safe_load(f) or []
                
            if not isinstance(instances, list):
                self.errors.append("provider_instances.yaml must contain a list")
                return
                
            for i, instance in enumerate(instances):
                self._validate_provider_instance(instance, i)
                
            self.info.append(f"Validated {len(instances)} provider instances")
            
        except yaml.YAMLError as e:
            self.errors.append(f"Invalid YAML in provider_instances.yaml: {e}")
        except Exception as e:
            self.errors.append(f"Error reading provider_instances.yaml: {e}")
            
    def _validate_provider_instance(self, instance: Dict[str, Any], index: int):
        """Validate a single provider instance."""
        required_fields = ["id", "display_name", "provider_type", "base_url"]
        
        for field in required_fields:
            if field not in instance:
                self.errors.append(f"Instance {index}: Missing required field '{field}'")
                
        # Validate ID format
        if "id" in instance:
            instance_id = instance["id"]
            if not instance_id or not isinstance(instance_id, str):
                self.errors.append(f"Instance {index}: Invalid ID format")
            elif " " in instance_id:
                self.warnings.append(f"Instance {index}: ID contains spaces")
                
        # Validate provider type
        if "provider_type" in instance:
            provider_type = instance["provider_type"]
            valid_types = ["openai", "anthropic", "huggingface", "ollama", "groq", "litellm"]
            if provider_type not in valid_types:
                self.warnings.append(f"Instance {index}: Unknown provider type '{provider_type}'")
                
        # Validate models
        if "models" in instance:
            models = instance["models"]
            if not isinstance(models, list):
                self.errors.append(f"Instance {index}: 'models' must be a list")
            else:
                for j, model in enumerate(models):
                    if not isinstance(model, dict):
                        self.errors.append(f"Instance {index}, model {j}: Model must be an object")
                    elif "id" not in model:
                        self.errors.append(f"Instance {index}, model {j}: Model missing 'id' field")
                        
    def _validate_tags(self):
        """Validate tags configuration."""
        tags_file = self.config_dir / "tags.yaml"
        
        if not tags_file.exists():
            self.info.append("tags.yaml not found (optional)")
            return
            
        try:
            with open(tags_file, 'r') as f:
                tags = yaml.safe_load(f) or []
                
            if not isinstance(tags, list):
                self.errors.append("tags.yaml must contain a list")
                return
                
            tag_names = set()
            tag_ids = set()
            
            for i, tag in enumerate(tags):
                self._validate_tag(tag, i, tag_names, tag_ids)
                
            self.info.append(f"Validated {len(tags)} tags")
            
        except yaml.YAMLError as e:
            self.errors.append(f"Invalid YAML in tags.yaml: {e}")
        except Exception as e:
            self.errors.append(f"Error reading tags.yaml: {e}")
            
    def _validate_tag(self, tag: Dict[str, Any], index: int, tag_names: Set[str], tag_ids: Set[str]):
        """Validate a single tag."""
        required_fields = ["id", "name"]
        
        for field in required_fields:
            if field not in tag:
                self.errors.append(f"Tag {index}: Missing required field '{field}'")
                
        # Validate name
        if "name" in tag:
            name = tag["name"]
            if not name or not isinstance(name, str):
                self.errors.append(f"Tag {index}: Invalid name")
            elif len(name) > 100:
                self.errors.append(f"Tag {index}: Name too long (max 100 characters)")
            elif name in tag_names:
                self.errors.append(f"Tag {index}: Duplicate tag name '{name}'")
            else:
                tag_names.add(name)
                
        # Validate ID
        if "id" in tag:
            tag_id = tag["id"]
            if not tag_id or not isinstance(tag_id, str):
                self.errors.append(f"Tag {index}: Invalid ID")
            elif tag_id in tag_ids:
                self.errors.append(f"Tag {index}: Duplicate tag ID '{tag_id}'")
            else:
                tag_ids.add(tag_id)
                
        # Validate color
        if "color" in tag:
            color = tag["color"]
            if color and not isinstance(color, str):
                self.errors.append(f"Tag {index}: Color must be a string")
            elif color and len(color) > 20:
                self.errors.append(f"Tag {index}: Color too long (max 20 characters)")
                
        # Validate description
        if "description" in tag:
            description = tag["description"]
            if description and len(description) > 500:
                self.errors.append(f"Tag {index}: Description too long (max 500 characters)")
                
        # Validate metadata
        if "metadata" in tag:
            metadata = tag["metadata"]
            if metadata and not isinstance(metadata, dict):
                self.errors.append(f"Tag {index}: Metadata must be an object")
                
    def _validate_labels(self):
        """Validate labels configuration."""
        labels_file = self.config_dir / "labels.yaml"
        
        if not labels_file.exists():
            self.info.append("labels.yaml not found (optional)")
            return
            
        try:
            with open(labels_file, 'r') as f:
                labels = yaml.safe_load(f) or []
                
            if not isinstance(labels, list):
                self.errors.append("labels.yaml must contain a list")
                return
                
            label_names = set()
            label_ids = set()
            
            for i, label in enumerate(labels):
                self._validate_label(label, i, label_names, label_ids)
                
            self.info.append(f"Validated {len(labels)} labels")
            
        except yaml.YAMLError as e:
            self.errors.append(f"Invalid YAML in labels.yaml: {e}")
        except Exception as e:
            self.errors.append(f"Error reading labels.yaml: {e}")
            
    def _validate_label(self, label: Dict[str, Any], index: int, label_names: Set[str], label_ids: Set[str]):
        """Validate a single label."""
        required_fields = ["id", "name"]
        
        for field in required_fields:
            if field not in label:
                self.errors.append(f"Label {index}: Missing required field '{field}'")
                
        # Validate name
        if "name" in label:
            name = label["name"]
            if not name or not isinstance(name, str):
                self.errors.append(f"Label {index}: Invalid name")
            elif len(name) > 100:
                self.errors.append(f"Label {index}: Name too long (max 100 characters)")
            elif name in label_names:
                self.errors.append(f"Label {index}: Duplicate label name '{name}'")
            else:
                label_names.add(name)
                
        # Validate ID
        if "id" in label:
            label_id = label["id"]
            if not label_id or not isinstance(label_id, str):
                self.errors.append(f"Label {index}: Invalid ID")
            elif label_id in label_ids:
                self.errors.append(f"Label {index}: Duplicate label ID '{label_id}'")
            else:
                label_ids.add(label_id)
                
        # Validate color
        if "color" in label:
            color = label["color"]
            if color and not isinstance(color, str):
                self.errors.append(f"Label {index}: Color must be a string")
            elif color and len(color) > 20:
                self.errors.append(f"Label {index}: Color too long (max 20 characters)")
                
        # Validate description
        if "description" in label:
            description = label["description"]
            if description and len(description) > 500:
                self.errors.append(f"Label {index}: Description too long (max 500 characters)")
                
        # Validate metadata
        if "metadata" in label:
            metadata = label["metadata"]
            if metadata and not isinstance(metadata, dict):
                self.errors.append(f"Label {index}: Metadata must be an object")
                
    def _validate_assignments(self):
        """Validate tag and label assignments."""
        self._validate_tag_assignments()
        self._validate_label_assignments()
        
    def _validate_tag_assignments(self):
        """Validate tag assignments."""
        assignments_file = self.config_dir / "tag_assignments.yaml"
        
        if not assignments_file.exists():
            self.info.append("tag_assignments.yaml not found (optional)")
            return
            
        try:
            with open(assignments_file, 'r') as f:
                assignments = yaml.safe_load(f) or []
                
            if not isinstance(assignments, list):
                self.errors.append("tag_assignments.yaml must contain a list")
                return
                
            assignment_ids = set()
            
            for i, assignment in enumerate(assignments):
                self._validate_tag_assignment(assignment, i, assignment_ids)
                
            self.info.append(f"Validated {len(assignments)} tag assignments")
            
        except yaml.YAMLError as e:
            self.errors.append(f"Invalid YAML in tag_assignments.yaml: {e}")
        except Exception as e:
            self.errors.append(f"Error reading tag_assignments.yaml: {e}")
            
    def _validate_tag_assignment(self, assignment: Dict[str, Any], index: int, assignment_ids: Set[str]):
        """Validate a single tag assignment."""
        required_fields = ["id", "tag_id", "target"]
        
        for field in required_fields:
            if field not in assignment:
                self.errors.append(f"Tag assignment {index}: Missing required field '{field}'")
                
        # Validate ID
        if "id" in assignment:
            assignment_id = assignment["id"]
            if not assignment_id or not isinstance(assignment_id, str):
                self.errors.append(f"Tag assignment {index}: Invalid ID")
            elif assignment_id in assignment_ids:
                self.errors.append(f"Tag assignment {index}: Duplicate assignment ID '{assignment_id}'")
            else:
                assignment_ids.add(assignment_id)
                
        # Validate target
        if "target" in assignment:
            target = assignment["target"]
            if not isinstance(target, dict):
                self.errors.append(f"Tag assignment {index}: Target must be an object")
            else:
                self._validate_assignment_target(target, index, "tag")
                
    def _validate_label_assignments(self):
        """Validate label assignments."""
        assignments_file = self.config_dir / "label_assignments.yaml"
        
        if not assignments_file.exists():
            self.info.append("label_assignments.yaml not found (optional)")
            return
            
        try:
            with open(assignments_file, 'r') as f:
                assignments = yaml.safe_load(f) or []
                
            if not isinstance(assignments, list):
                self.errors.append("label_assignments.yaml must contain a list")
                return
                
            assignment_ids = set()
            label_ids_seen = set()
            
            for i, assignment in enumerate(assignments):
                self._validate_label_assignment(assignment, i, assignment_ids, label_ids_seen)
                
            self.info.append(f"Validated {len(assignments)} label assignments")
            
        except yaml.YAMLError as e:
            self.errors.append(f"Invalid YAML in label_assignments.yaml: {e}")
        except Exception as e:
            self.errors.append(f"Error reading label_assignments.yaml: {e}")
            
    def _validate_label_assignment(self, assignment: Dict[str, Any], index: int, assignment_ids: Set[str], label_ids_seen: Set[str]):
        """Validate a single label assignment."""
        required_fields = ["id", "label_id", "target"]
        
        for field in required_fields:
            if field not in assignment:
                self.errors.append(f"Label assignment {index}: Missing required field '{field}'")
                
        # Validate ID
        if "id" in assignment:
            assignment_id = assignment["id"]
            if not assignment_id or not isinstance(assignment_id, str):
                self.errors.append(f"Label assignment {index}: Invalid ID")
            elif assignment_id in assignment_ids:
                self.errors.append(f"Label assignment {index}: Duplicate assignment ID '{assignment_id}'")
            else:
                assignment_ids.add(assignment_id)
                
        # Check label uniqueness
        if "label_id" in assignment:
            label_id = assignment["label_id"]
            if label_id in label_ids_seen:
                self.errors.append(f"Label assignment {index}: Label '{label_id}' already assigned")
            else:
                label_ids_seen.add(label_id)
                
        # Validate target
        if "target" in assignment:
            target = assignment["target"]
            if not isinstance(target, dict):
                self.errors.append(f"Label assignment {index}: Target must be an object")
            else:
                self._validate_assignment_target(target, index, "label")
                
    def _validate_assignment_target(self, target: Dict[str, Any], index: int, assignment_type: str):
        """Validate assignment target."""
        if "provider_instance" in target:
            instance_id = target["provider_instance"]
            if not instance_id or not isinstance(instance_id, str):
                self.errors.append(f"{assignment_type.capitalize()} assignment {index}: Invalid instance ID")
        elif "model" in target:
            model_target = target["model"]
            if not isinstance(model_target, dict):
                self.errors.append(f"{assignment_type.capitalize()} assignment {index}: Model target must be an object")
            else:
                if "instance_id" not in model_target:
                    self.errors.append(f"{assignment_type.capitalize()} assignment {index}: Model target missing instance_id")
                if "model_id" not in model_target:
                    self.errors.append(f"{assignment_type.capitalize()} assignment {index}: Model target missing model_id")
        else:
            self.errors.append(f"{assignment_type.capitalize()} assignment {index}: Invalid target type")
            
    def _check_referential_integrity(self):
        """Check referential integrity between configurations."""
        # Load all configurations
        tags = self._load_yaml_file("tags.yaml", [])
        labels = self._load_yaml_file("labels.yaml", [])
        tag_assignments = self._load_yaml_file("tag_assignments.yaml", [])
        label_assignments = self._load_yaml_file("label_assignments.yaml", [])
        instances = self._load_yaml_file("provider_instances.yaml", [])
        
        # Create lookup sets
        tag_ids = {tag["id"] for tag in tags if "id" in tag}
        label_ids = {label["id"] for label in labels if "id" in label}
        instance_ids = {instance["id"] for instance in instances if "id" in instance}
        
        # Check tag assignments refer to valid tags
        for assignment in tag_assignments:
            if "tag_id" in assignment:
                tag_id = assignment["tag_id"]
                if tag_id not in tag_ids:
                    self.errors.append(f"Tag assignment references unknown tag: {tag_id}")
                    
        # Check label assignments refer to valid labels
        for assignment in label_assignments:
            if "label_id" in assignment:
                label_id = assignment["label_id"]
                if label_id not in label_ids:
                    self.errors.append(f"Label assignment references unknown label: {label_id}")
                    
        # Check assignment targets refer to valid instances
        for assignment in tag_assignments + label_assignments:
            if "target" in assignment:
                target = assignment["target"]
                if "provider_instance" in target:
                    instance_id = target["provider_instance"]
                    if instance_id not in instance_ids:
                        self.errors.append(f"Assignment references unknown instance: {instance_id}")
                elif "model" in target:
                    instance_id = target.get("instance_id")
                    if instance_id and instance_id not in instance_ids:
                        self.errors.append(f"Assignment references unknown instance: {instance_id}")
                        
    def _check_file_permissions(self):
        """Check file permissions for security."""
        config_files = [
            "provider_instances.yaml",
            "provider_configs.yaml",
            "tags.yaml",
            "tag_assignments.yaml",
            "labels.yaml",
            "label_assignments.yaml"
        ]
        
        for filename in config_files:
            file_path = self.config_dir / filename
            if file_path.exists():
                stat = file_path.stat()
                mode = stat.st_mode & 0o777
                
                # Check if file is readable by others
                if mode & 0o004:
                    self.warnings.append(f"File {filename} is readable by others (permissions: {oct(mode)})")
                    
                # Check if file is writable by others
                if mode & 0o002:
                    self.errors.append(f"File {filename} is writable by others (permissions: {oct(mode)})")
                    
    def _load_yaml_file(self, filename: str, default: Any) -> Any:
        """Load a YAML file safely."""
        file_path = self.config_dir / filename
        if not file_path.exists():
            return default
            
        try:
            with open(file_path, 'r') as f:
                return yaml.safe_load(f) or default
        except Exception:
            return default


def main():
    parser = argparse.ArgumentParser(description="AICred Configuration Validation Utility")
    parser.add_argument("--config-dir", type=Path, help="Configuration directory path")
    parser.add_argument("--check-tags", action="store_true", help="Check only tags configuration")
    parser.add_argument("--check-labels", action="store_true", help="Check only labels configuration")
    parser.add_argument("--check-assignments", action="store_true", help="Check only assignments")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    
    args = parser.parse_args()
    
    validator = ConfigValidator(args.config_dir)
    
    if args.check_tags:
        validator._validate_tags()
    elif args.check_labels:
        validator._validate_labels()
    elif args.check_assignments:
        validator._validate_assignments()
    else:
        result = validator.validate_all()
        
    # Output results
    if validator.errors:
        print("❌ ERRORS:")
        for error in validator.errors:
            print(f"  - {error}")
            
    if validator.warnings:
        print("\n⚠️  WARNINGS:")
        for warning in validator.warnings:
            print(f"  - {warning}")
            
    if args.verbose and validator.info:
        print("\nℹ️  INFO:")
        for info in validator.info:
            print(f"  - {info}")
            
    # Summary
    if validator.errors:
        print(f"\n❌ Validation failed with {len(validator.errors)} errors")
        sys.exit(1)
    elif validator.warnings:
        print(f"\n⚠️  Validation completed with {len(validator.warnings)} warnings")
        sys.exit(0)
    else:
        print("\n✅ Configuration is valid")
        sys.exit(0)


if __name__ == "__main__":
    main()