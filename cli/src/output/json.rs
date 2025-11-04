use aicred_core::ScanResult;
use anyhow::Result;

pub fn output_json(result: &ScanResult, _verbose: bool) -> Result<()> {
    // Enhance the result with tag/label information
    let enhanced_result = enhance_result_with_tags_labels(result)?;
    let json = serde_json::to_string_pretty(&enhanced_result)?;
    println!("{}", json);
    Ok(())
}

/// Enhance scan result with tag and label information
fn enhance_result_with_tags_labels(result: &ScanResult) -> Result<serde_json::Value> {
    let mut enhanced = serde_json::to_value(result)?;

    // Add tags and labels information to each config instance
    if let Some(instances) = enhanced.get_mut("config_instances") {
        if let Some(instances_array) = instances.as_array_mut() {
            for instance in instances_array {
                if let Some(provider_instances) = instance.get_mut("provider_instances") {
                    if let Some(providers_array) = provider_instances.as_array_mut() {
                        for provider in providers_array.iter_mut() {
                            // Extract instance ID first
                            let instance_id =
                                provider.get("id").and_then(|v| v.as_str()).unwrap_or("");

                            // Get tags and labels for this instance
                            let tags = get_tags_for_instance(instance_id)?;
                            let labels = get_labels_for_instance(instance_id)?;

                            // Clone provider to avoid borrow conflicts
                            let mut provider_obj = provider.as_object().unwrap().clone();
                            provider_obj.insert("tags".to_string(), serde_json::to_value(&tags)?);
                            provider_obj
                                .insert("labels".to_string(), serde_json::to_value(&labels)?);

                            // Add tags and labels to models within this provider instance
                            if let Some(models) = provider_obj.get_mut("models") {
                                if let Some(models_array) = models.as_array_mut() {
                                    for model in models_array.iter_mut() {
                                        let model_id = model
                                            .get("model_id")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("");

                                        // Get tags and labels for this model
                                        let model_tags = get_tags_for_model(instance_id, model_id)?;
                                        let model_labels =
                                            get_labels_for_model(instance_id, model_id)?;

                                        // Add to model
                                        if let Some(model_obj) = model.as_object_mut() {
                                            model_obj.insert(
                                                "tags".to_string(),
                                                serde_json::to_value(&model_tags)?,
                                            );
                                            model_obj.insert(
                                                "labels".to_string(),
                                                serde_json::to_value(&model_labels)?,
                                            );
                                        }
                                    }
                                }
                            }

                            // Replace the provider with the enhanced version
                            *provider = serde_json::Value::Object(provider_obj);
                        }
                    }
                }
            }
        }
    }

    Ok(enhanced)
}

/// Get tags for a specific instance
fn get_tags_for_instance(instance_id: &str) -> Result<Vec<serde_json::Value>> {
    use crate::commands::tags::get_tags_for_target;

    let tags = get_tags_for_target(instance_id, None)?;
    let tags_json: Result<Vec<serde_json::Value>, _> =
        tags.iter().map(|tag| serde_json::to_value(tag)).collect();

    Ok(tags_json?)
}

/// Get labels for a specific instance
fn get_labels_for_instance(instance_id: &str) -> Result<Vec<serde_json::Value>> {
    use crate::commands::labels::get_labels_for_target;

    let labels = get_labels_for_target(instance_id, None)?;
    let labels_json: Result<Vec<serde_json::Value>, _> = labels
        .iter()
        .map(|label| serde_json::to_value(label))
        .collect();

    Ok(labels_json?)
}

/// Get tags for a specific model
fn get_tags_for_model(instance_id: &str, model_id: &str) -> Result<Vec<serde_json::Value>> {
    use crate::commands::tags::get_tags_for_target;

    let tags = get_tags_for_target(instance_id, Some(model_id))?;
    let tags_json: Result<Vec<serde_json::Value>, _> =
        tags.iter().map(|tag| serde_json::to_value(tag)).collect();

    Ok(tags_json?)
}

/// Get labels for a specific model
fn get_labels_for_model(instance_id: &str, model_id: &str) -> Result<Vec<serde_json::Value>> {
    use crate::commands::labels::get_labels_for_target;

    let labels = get_labels_for_target(instance_id, Some(model_id))?;
    let labels_json: Result<Vec<serde_json::Value>, _> = labels
        .iter()
        .map(|label| serde_json::to_value(label))
        .collect();

    Ok(labels_json?)
}
