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
        for provider_instance in enhanced_instance.provider_instances.instances.values_mut() {
            // Get tags for this provider instance
            if let Ok(tags) = get_tags_for_target(&instance.instance_id, None, None) {
                if !tags.is_empty() {
                    let tag_names: Vec<String> = tags.iter().map(|tag| tag.name.clone()).collect();
                    let tags_json = serde_json::to_string(&tag_names)
                        .map_err(|e| anyhow::anyhow!("Failed to serialize tags: {}", e))?;
                    provider_instance.metadata.insert("tags".to_string(), tags_json);
                }
            }

            // Get labels for this provider instance
            if let Ok(labels) = get_labels_for_target(&instance.instance_id, None, None) {
                if !labels.is_empty() {
                    let label_names: Vec<String> = labels.iter().map(|label| label.name.clone()).collect();
                    let labels_json = serde_json::to_string(&label_names)
                        .map_err(|e| anyhow::anyhow!("Failed to serialize labels: {}", e))?;
                    provider_instance.metadata.insert("labels".to_string(), labels_json);
                }
            }

            // Note: Models are now Vec<String>, so we can't attach metadata to individual models.
            // Tags/labels for models could be stored in provider_instance.metadata with model-prefixed keys
            // if needed, but skipping for now since models don't have a metadata field.
        }

        let json = serde_json::to_string(&enhanced_instance)?;
        println!("{}", json);
    }
    Ok(())
}
