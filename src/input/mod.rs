pub mod csv;
pub mod models;
pub mod xlsx;
pub mod yaml;

use anyhow::{bail, Result};
use std::path::Path;

use crate::cli::ResourceType;
use models::InputFile;

/// Detect the file format from the extension and parse the file.
///
/// YAML – type inferred from top-level keys; `resource_type` is ignored.
/// CSV  – `resource_type` is required.
/// XLSX – `resource_type` optional; auto-detected from sheet names otherwise.
pub fn parse_file(path: &Path, resource_type: Option<&ResourceType>) -> Result<InputFile> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "yaml" | "yml" => yaml::parse(path),
        "csv" => {
            let rt = resource_type.ok_or_else(|| {
                anyhow::anyhow!(
                    "--type is required for CSV files.\n  \
                     Use: --type objective | epic | story"
                )
            })?;
            csv::parse(path, rt)
        }
        "xlsx" | "xls" => xlsx::parse(path, resource_type),
        other => bail!("Unsupported file extension '.{other}'.  Use .yaml, .csv, or .xlsx"),
    }
}
