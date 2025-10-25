use anyhow::Result;
use genai_keyfinder_core::ScanResult;

pub fn output_json(result: &ScanResult) -> Result<()> {
    let json = serde_json::to_string_pretty(result)?;
    println!("{}", json);
    Ok(())
}
