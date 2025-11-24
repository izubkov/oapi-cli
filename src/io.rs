use anyhow::{Context, Result};
use openapiv3::OpenAPI;
use std::fs;
use std::path::Path;

/// Load a spec from JSON or YAML. Returns typed OpenAPI struct.
pub fn load_spec(path: &Path) -> Result<OpenAPI> {
    let s = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file {}", path.display()))?;

    // Heuristic: if it starts with '{' or '[', parse as JSON; otherwise try YAML then JSON fallback
    let trimmed = s.trim_start();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        let spec: OpenAPI = serde_json::from_str(&s)
            .with_context(|| format!("Failed to parse OpenAPI JSON for {}", path.display()))?;
        return Ok(spec);
    }

    // Try YAML first
    match serde_yaml::from_str::<OpenAPI>(&s) {
        Ok(spec) => Ok(spec),
        Err(yaml_err) => {
            // Fallback to JSON parse with friendly error
            serde_json::from_str(&s).with_context(|| {
                format!(
                    "Failed to parse OpenAPI YAML or JSON for {}: {}",
                    path.display(),
                    yaml_err
                )
            })
        }
    }
}

/// Save a spec as JSON or YAML depending on output extension (.yaml/.yml => YAML, else JSON)
pub fn save_spec(spec: &OpenAPI, path: &Path) -> Result<()> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if ext == "yaml" || ext == "yml" {
        let s = serde_yaml::to_string(spec)
            .with_context(|| format!("Failed to serialize OpenAPI to YAML for {}", path.display()))?;
        fs::write(path, s).with_context(|| format!("Failed to write file {}", path.display()))?;
    } else {
        let s = serde_json::to_string_pretty(spec)
            .with_context(|| format!("Failed to serialize OpenAPI to JSON for {}", path.display()))?;
        fs::write(path, s).with_context(|| format!("Failed to write file {}", path.display()))?;
    }

    Ok(())
}
