use aicred_core::ScanResult;
use anyhow::Result;

pub fn output_json(result: &ScanResult, _verbose: bool) -> Result<()> {
    let json = serde_json::to_string_pretty(result)?;
    println!("{}", json);
    Ok(())
}
