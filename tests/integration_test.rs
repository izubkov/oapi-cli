use std::fs;
use std::path::Path;
use tempfile::TempDir;

// Helper function to create a test OpenAPI spec with multiple paths and components
fn create_test_spec_json() -> String {
    r##"{
  "openapi": "3.0.0",
  "info": {
    "title": "Test API",
    "version": "1.0.0"
  },
  "paths": {
    "/api/v1/users": {
      "get": {
        "summary": "Get users",
        "responses": {
          "200": {
            "description": "Success",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/User"
                }
              }
            }
          }
        }
      }
    },
    "/api/v1/posts": {
      "get": {
        "summary": "Get posts",
        "responses": {
          "200": {
            "description": "Success",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/Post"
                }
              }
            }
          }
        }
      }
    },
    "/api/v2/users": {
      "get": {
        "summary": "Get users v2",
        "responses": {
          "200": {
            "description": "Success",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/UserV2"
                }
              }
            }
          }
        }
      }
    }
  },
  "components": {
    "schemas": {
      "User": {
        "type": "object",
        "properties": {
          "id": {
            "type": "integer"
          },
          "name": {
            "type": "string"
          }
        }
      },
      "Post": {
        "type": "object",
        "properties": {
          "id": {
            "type": "integer"
          },
          "title": {
            "type": "string"
          },
          "authorId": {
            "type": "integer"
          }
        }
      },
      "UserV2": {
        "type": "object",
        "properties": {
          "id": {
            "type": "integer"
          },
          "name": {
            "type": "string"
          },
          "email": {
            "type": "string"
          }
        }
      },
      "UnusedSchema": {
        "type": "object",
        "properties": {
          "unused": {
            "type": "string"
          }
        }
      }
    }
  }
}"##.to_string()
}

fn create_test_spec_yaml() -> String {
    r##"openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths:
  /api/v1/users:
    get:
      summary: Get users
      responses:
        '200':
          description: Success
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/User'
  /api/v1/posts:
    get:
      summary: Get posts
      responses:
        '200':
          description: Success
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Post'
  /api/v2/users:
    get:
      summary: Get users v2
      responses:
        '200':
          description: Success
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/UserV2'
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: integer
        name:
          type: string
    Post:
      type: object
      properties:
        id:
          type: integer
        title:
          type: string
        authorId:
          type: integer
    UserV2:
      type: object
      properties:
        id:
          type: integer
        name:
          type: string
        email:
          type: string
    UnusedSchema:
      type: object
      properties:
        unused:
          type: string
"##.to_string()
}

fn run_filter_command(input: &Path, output: &Path, paths: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;
    
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--quiet")
        .arg("--")
        .arg("filter")
        .arg("-i")
        .arg(input)
        .arg("-o")
        .arg(output);
    
    for path in paths {
        cmd.arg("--path").arg(path);
    }
    
    let output_result = cmd.output()?;
    
    if !output_result.status.success() {
        return Err(format!(
            "Command failed with status: {:?}\nStderr: {}",
            output_result.status,
            String::from_utf8_lossy(&output_result.stderr)
        ).into());
    }
    
    Ok(())
}

#[test]
fn test_integration_filter_json_to_json() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.json");
    let output_path = temp_dir.path().join("output.json");
    
    fs::write(&input_path, create_test_spec_json()).unwrap();
    
    run_filter_command(&input_path, &output_path, &["/api/v1/*"]).unwrap();
    
    assert!(output_path.exists());
    let content = fs::read_to_string(&output_path).unwrap();
    
    // Should contain v1 paths
    assert!(content.contains("\"/api/v1/users\""));
    assert!(content.contains("\"/api/v1/posts\""));
    
    // Should not contain v2 paths
    assert!(!content.contains("\"/api/v2/users\""));
}

#[test]
fn test_integration_filter_yaml_to_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.yaml");
    let output_path = temp_dir.path().join("output.yaml");
    
    fs::write(&input_path, create_test_spec_yaml()).unwrap();
    
    run_filter_command(&input_path, &output_path, &["/api/v1/*"]).unwrap();
    
    assert!(output_path.exists());
    let content = fs::read_to_string(&output_path).unwrap();
    
    // Should contain v1 paths
    assert!(content.contains("/api/v1/users"));
    assert!(content.contains("/api/v1/posts"));
    
    // Should not contain v2 paths
    assert!(!content.contains("/api/v2/users"));
}

#[test]
fn test_integration_filter_json_to_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.json");
    let output_path = temp_dir.path().join("output.yaml");
    
    fs::write(&input_path, create_test_spec_json()).unwrap();
    
    run_filter_command(&input_path, &output_path, &["/api/v1/users"]).unwrap();
    
    assert!(output_path.exists());
    let content = fs::read_to_string(&output_path).unwrap();
    
    // Output should be YAML format
    assert!(content.contains("openapi: "));
    assert!(content.contains("/api/v1/users"));
    
    // Should be valid YAML
    let _: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();
}

#[test]
fn test_integration_filter_yaml_to_json() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.yaml");
    let output_path = temp_dir.path().join("output.json");
    
    fs::write(&input_path, create_test_spec_yaml()).unwrap();
    
    run_filter_command(&input_path, &output_path, &["/api/v1/users"]).unwrap();
    
    assert!(output_path.exists());
    let content = fs::read_to_string(&output_path).unwrap();
    
    // Output should be JSON format
    assert!(content.contains("\"openapi\":"));
    assert!(content.contains("\"/api/v1/users\""));
    
    // Should be valid JSON
    let _: serde_json::Value = serde_json::from_str(&content).unwrap();
}

#[test]
fn test_integration_components_pruned() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.json");
    let output_path = temp_dir.path().join("output.json");
    
    fs::write(&input_path, create_test_spec_json()).unwrap();
    
    // Filter to only keep /api/v1/users path
    run_filter_command(&input_path, &output_path, &["/api/v1/users"]).unwrap();
    
    let content = fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    // Should only contain User schema (referenced by /api/v1/users)
    let schemas = &json["components"]["schemas"];
    assert!(schemas["User"].is_object());
    
    // Should not contain Post, UserV2, or UnusedSchema
    assert!(schemas["Post"].is_null());
    assert!(schemas["UserV2"].is_null());
    assert!(schemas["UnusedSchema"].is_null());
}

#[test]
fn test_integration_components_preserved() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.json");
    let output_path = temp_dir.path().join("output.json");
    
    fs::write(&input_path, create_test_spec_json()).unwrap();
    
    // Filter to keep both v1 paths
    run_filter_command(&input_path, &output_path, &["/api/v1/*"]).unwrap();
    
    let content = fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    // Should contain both User and Post schemas (referenced by /api/v1/* paths)
    let schemas = &json["components"]["schemas"];
    assert!(schemas["User"].is_object());
    assert!(schemas["Post"].is_object());
    
    // Should not contain UserV2 or UnusedSchema
    assert!(schemas["UserV2"].is_null());
    assert!(schemas["UnusedSchema"].is_null());
}

#[test]
fn test_integration_exact_path_match() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.json");
    let output_path = temp_dir.path().join("output.json");
    
    fs::write(&input_path, create_test_spec_json()).unwrap();
    
    // Use exact path without wildcard
    run_filter_command(&input_path, &output_path, &["/api/v1/users"]).unwrap();
    
    let content = fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    let paths = &json["paths"];
    
    // Should only contain exact match
    assert!(paths["/api/v1/users"].is_object());
    assert!(paths["/api/v1/posts"].is_null());
    assert!(paths["/api/v2/users"].is_null());
}

#[test]
fn test_integration_multiple_exact_paths() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.json");
    let output_path = temp_dir.path().join("output.json");
    
    fs::write(&input_path, create_test_spec_json()).unwrap();
    
    // Use multiple exact paths
    run_filter_command(&input_path, &output_path, &["/api/v1/users", "/api/v2/users"]).unwrap();
    
    let content = fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    let paths = &json["paths"];
    
    // Should contain both exact matches
    assert!(paths["/api/v1/users"].is_object());
    assert!(paths["/api/v2/users"].is_object());
    assert!(paths["/api/v1/posts"].is_null());
}

#[test]
fn test_integration_empty_result() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.json");
    let output_path = temp_dir.path().join("output.json");
    
    fs::write(&input_path, create_test_spec_json()).unwrap();
    
    // Filter with path that doesn't exist
    run_filter_command(&input_path, &output_path, &["/nonexistent/*"]).unwrap();
    
    let content = fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    let paths = &json["paths"];
    
    // Should be empty object
    assert!(paths.is_object());
    assert_eq!(paths.as_object().unwrap().len(), 0);
}

#[test]
fn test_cli_requires_path_argument() {
    use std::process::Command;
    
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.json");
    let output_path = temp_dir.path().join("output.json");
    
    fs::write(&input_path, create_test_spec_json()).unwrap();
    
    // Run without --path argument
    let output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--")
        .arg("filter")
        .arg("-i")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .unwrap();
    
    // Should fail with exit code 2
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("at least one --path must be provided"));
}
