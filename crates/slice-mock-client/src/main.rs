use anyhow::{bail, Result};

fn main() -> Result<()> {
    let report = slice_mock_client::run_mock_client()?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    if !report.passed {
        bail!("SLICE mock client validation failed");
    }
    Ok(())
}
