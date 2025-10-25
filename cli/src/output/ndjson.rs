use anyhow::Result;
use genai_keyfinder_core::ScanResult;

pub fn output_ndjson(result: &ScanResult) -> Result<()> {
    for key in &result.keys {
        let json = serde_json::to_string(key)?;
        println!("{}", json);
    }
    for instance in &result.config_instances {
        let json = serde_json::to_string(instance)?;
        println!("{}", json);
    }
    Ok(())
}
