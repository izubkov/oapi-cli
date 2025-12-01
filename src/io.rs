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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_minimal_spec() -> OpenAPI {
        OpenAPI {
            openapi: "3.0.0".to_string(),
            info: openapiv3::Info {
                title: "Test API".to_string(),
                version: "1.0.0".to_string(),
                description: Some("A test API".to_string()),
                ..Default::default()
            },
            paths: Default::default(),
            ..Default::default()
        }
    }

    fn create_test_json_content() -> String {
        r#"{
  "openapi": "3.0.0",
  "info": {
    "title": "Test API",
    "version": "1.0.0"
  },
  "paths": {}
}"#.to_string()
    }

    fn create_test_yaml_content() -> String {
        r#"openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths: {}
"#.to_string()
    }

    #[test]
    fn test_load_spec_json() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");
        fs::write(&file_path, create_test_json_content()).unwrap();

        let result = load_spec(&file_path);
        assert!(result.is_ok());
        
        let spec = result.unwrap();
        assert_eq!(spec.openapi, "3.0.0");
        assert_eq!(spec.info.title, "Test API");
        assert_eq!(spec.info.version, "1.0.0");
    }

    #[test]
    fn test_load_spec_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.yaml");
        fs::write(&file_path, create_test_yaml_content()).unwrap();

        let result = load_spec(&file_path);
        assert!(result.is_ok());
        
        let spec = result.unwrap();
        assert_eq!(spec.openapi, "3.0.0");
        assert_eq!(spec.info.title, "Test API");
        assert_eq!(spec.info.version, "1.0.0");
    }

    #[test]
    fn test_load_spec_invalid_path() {
        let result = load_spec(Path::new("/nonexistent/path/to/file.json"));
        assert!(result.is_err());
        
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Failed to read file"));
    }

    #[test]
    fn test_load_spec_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid.json");
        fs::write(&file_path, "{ this is not valid json }").unwrap();

        let result = load_spec(&file_path);
        assert!(result.is_err());
        
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Failed to parse"));
    }

    #[test]
    fn test_load_spec_invalid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid.yaml");
        fs::write(&file_path, "openapi: 3.0.0\ninfo:\n  - this is invalid yaml structure").unwrap();

        let result = load_spec(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_spec_json() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.json");
        let spec = create_minimal_spec();

        let result = save_spec(&spec, &file_path);
        assert!(result.is_ok());
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("\"openapi\": \"3.0.0\""));
        assert!(content.contains("\"title\": \"Test API\""));
        
        // Verify it can be parsed back
        let loaded: OpenAPI = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded.info.title, "Test API");
    }

    #[test]
    fn test_save_spec_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.yaml");
        let spec = create_minimal_spec();

        let result = save_spec(&spec, &file_path);
        assert!(result.is_ok());
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("openapi:"));
        assert!(content.contains("3.0.0"));
        assert!(content.contains("title: Test API"));
        
        // Verify it can be parsed back
        let loaded: OpenAPI = serde_yaml::from_str(&content).unwrap();
        assert_eq!(loaded.info.title, "Test API");
    }

    #[test]
    fn test_save_spec_yml() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.yml");
        let spec = create_minimal_spec();

        let result = save_spec(&spec, &file_path);
        assert!(result.is_ok());
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("openapi:"));
        assert!(content.contains("3.0.0"));
        
        // Verify it's YAML format
        let loaded: OpenAPI = serde_yaml::from_str(&content).unwrap();
        assert_eq!(loaded.info.title, "Test API");
    }

    #[test]
    fn test_save_spec_default_format() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output");
        let spec = create_minimal_spec();

        let result = save_spec(&spec, &file_path);
        assert!(result.is_ok());
        assert!(file_path.exists());

        // Should default to JSON
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("\"openapi\": \"3.0.0\""));
        
        // Verify it's valid JSON
        let loaded: OpenAPI = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded.info.title, "Test API");
    }

    #[test]
    fn test_load_save_roundtrip_json() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.json");
        let output_path = temp_dir.path().join("output.json");
        
        let original_spec = create_minimal_spec();
        save_spec(&original_spec, &input_path).unwrap();
        
        let loaded_spec = load_spec(&input_path).unwrap();
        save_spec(&loaded_spec, &output_path).unwrap();
        
        let final_spec = load_spec(&output_path).unwrap();
        assert_eq!(final_spec.info.title, original_spec.info.title);
        assert_eq!(final_spec.openapi, original_spec.openapi);
    }

    #[test]
    fn test_load_save_roundtrip_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.yaml");
        let output_path = temp_dir.path().join("output.yaml");
        
        let original_spec = create_minimal_spec();
        save_spec(&original_spec, &input_path).unwrap();
        
        let loaded_spec = load_spec(&input_path).unwrap();
        save_spec(&loaded_spec, &output_path).unwrap();
        
        let final_spec = load_spec(&output_path).unwrap();
        assert_eq!(final_spec.info.title, original_spec.info.title);
        assert_eq!(final_spec.openapi, original_spec.openapi);
    }

    #[test]
    fn test_load_json_save_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("input.json");
        let yaml_path = temp_dir.path().join("output.yaml");
        
        fs::write(&json_path, create_test_json_content()).unwrap();
        
        let spec = load_spec(&json_path).unwrap();
        save_spec(&spec, &yaml_path).unwrap();
        
        let yaml_content = fs::read_to_string(&yaml_path).unwrap();
        assert!(yaml_content.contains("openapi: "));
        assert!(yaml_content.contains("title: Test API"));
    }

    #[test]
    fn test_load_yaml_save_json() {
        let temp_dir = TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join("input.yaml");
        let json_path = temp_dir.path().join("output.json");
        
        fs::write(&yaml_path, create_test_yaml_content()).unwrap();
        
        let spec = load_spec(&yaml_path).unwrap();
        save_spec(&spec, &json_path).unwrap();
        
        let json_content = fs::read_to_string(&json_path).unwrap();
        assert!(json_content.contains("\"openapi\": \"3.0.0\""));
        assert!(json_content.contains("\"title\": \"Test API\""));
    }
}
