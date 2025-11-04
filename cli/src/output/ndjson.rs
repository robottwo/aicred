use crate::commands::{get_labels_for_target, get_tags_for_target};
use aicred_core::ScanResult;
use anyhow::Result;

pub fn output_ndjson(result: &ScanResult, _verbose: bool) -> Result<()> {
    for key in &result.keys {
        let json = serde_json::to_string(key)?;
        println!("{}", json);
    }
    for instance in &result.config_instances {
        // Create enhanced instance with tag/label information
        let mut enhanced_instance = instance.clone();

        // Add tags and labels to provider instances
        for provider_instance in enhanced_instance.provider_instances.all_instances_mut() {
            // Get tags for this provider instance
            if let Ok(tags) = get_tags_for_target(&instance.instance_id, None, None) {
                if !tags.is_empty() {
                    // Add tags to provider instance metadata or create a new field
                    if provider_instance.metadata.is_none() {
                        provider_instance.metadata = Some(std::collections::HashMap::new());
                    }
                    if let Some(metadata) = &mut provider_instance.metadata {
                        let tag_names: Vec<String> =
                            tags.iter().map(|tag| tag.name.clone()).collect();
                        let tags_json = serde_json::to_string(&tag_names)
                            .map_err(|e| anyhow::anyhow!("Failed to serialize tags: {}", e))?;
                        metadata.insert("tags".to_string(), tags_json);
                    }
                }
            }

            // Get labels for this provider instance
            if let Ok(labels) = get_labels_for_target(&instance.instance_id, None, None) {
                if !labels.is_empty() {
                    if provider_instance.metadata.is_none() {
                        provider_instance.metadata = Some(std::collections::HashMap::new());
                    }
                    if let Some(metadata) = &mut provider_instance.metadata {
                        let label_names: Vec<String> =
                            labels.iter().map(|label| label.name.clone()).collect();
                        let labels_json = serde_json::to_string(&label_names)
                            .map_err(|e| anyhow::anyhow!("Failed to serialize labels: {}", e))?;
                        metadata.insert("labels".to_string(), labels_json);
                    }
                }
            }

            // Add tags and labels to models
            for model in &mut provider_instance.models {
                // Get tags for this model
                if let Ok(tags) =
                    get_tags_for_target(&instance.instance_id, Some(&model.name), None)
                {
                    if !tags.is_empty() {
                        if model.metadata.is_none() {
                            model.metadata = Some(std::collections::HashMap::new());
                        }
                        if let Some(metadata) = &mut model.metadata {
                            let tag_names: Vec<String> =
                                tags.iter().map(|tag| tag.name.clone()).collect();
                            metadata.insert("tags".to_string(), serde_json::to_value(&tag_names)?);
                        }
                    }
                }

                // Get labels for this model
                if let Ok(labels) =
                    get_labels_for_target(&instance.instance_id, Some(&model.name), None)
                {
                    if !labels.is_empty() {
                        if model.metadata.is_none() {
                            model.metadata = Some(std::collections::HashMap::new());
                        }
                        if let Some(metadata) = &mut model.metadata {
                            let label_names: Vec<String> =
                                labels.iter().map(|label| label.name.clone()).collect();
                            metadata
                                .insert("labels".to_string(), serde_json::to_value(&label_names)?);
                        }
                    }
                }
            }
        }

        let json = serde_json::to_string(&enhanced_instance)?;
        println!("{}", json);
    }
    Ok(())
}
