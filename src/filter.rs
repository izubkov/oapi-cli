use indexmap::IndexMap;
use openapiv3::OpenAPI;
use serde_json::Value;
use std::collections::HashSet;

/// Keep only the provided paths (exact match) or prefixes (if path ends with '*')
/// Behavior: if a provided path is an exact path (no trailing '*') we match exact. If it ends with
/// a trailing '*' we treat it as prefix (e.g. '/api/v1/*' keeps '/api/v1/..').
pub fn filter_paths(spec: &mut OpenAPI, keep_paths: &[String]) {
    let old_paths = std::mem::take(&mut spec.paths.paths);
    spec.paths.paths = old_paths
        .into_iter()
        .filter(|(path, _)| should_keep(path, keep_paths))
        .collect();
}

fn should_keep(path: &str, keep_paths: &[String]) -> bool {
    for kp in keep_paths {
        if kp.ends_with('*') {
            let prefix = kp.trim_end_matches('*');
            if path.starts_with(prefix) {
                return true;
            }
        } else if path == kp {
            return true;
        }
    }
    false
}

/// Recursively collect all "$ref" string values in the document
/// We convert to JSON value to traverse all nested structures
pub fn collect_refs(spec: &OpenAPI) -> HashSet<String> {
    let mut refs = HashSet::new();
    // Convert to JSON to easily traverse all nested structures
    if let Ok(value) = serde_json::to_value(spec) {
        collect_refs_inner(&value, &mut refs);
    }
    refs
}

fn collect_refs_inner(v: &Value, out: &mut HashSet<String>) {
    match v {
        Value::Object(map) => {
            if let Some(Value::String(r)) = map.get("$ref") {
                out.insert(r.clone());
            }
            for val in map.values() {
                collect_refs_inner(val, out);
            }
        }
        Value::Array(arr) => {
            for val in arr {
                collect_refs_inner(val, out);
            }
        }
        _ => {}
    }
}

/// Prune all components.* maps to keep only referenced items
pub fn prune_components(spec: &mut OpenAPI, refs: &HashSet<String>) {
    let used_schemas = filter_refs(refs, "#/components/schemas/");
    let used_params = filter_refs(refs, "#/components/parameters/");
    let used_responses = filter_refs(refs, "#/components/responses/");
    let used_bodies = filter_refs(refs, "#/components/requestBodies/");
    let used_headers = filter_refs(refs, "#/components/headers/");
    let used_security = filter_refs(refs, "#/components/securitySchemes/");
    let used_examples = filter_refs(refs, "#/components/examples/");
    let used_links = filter_refs(refs, "#/components/links/");
    let used_callbacks = filter_refs(refs, "#/components/callbacks/");

    if let Some(components) = &mut spec.components {
        prune_map(&mut components.schemas, &used_schemas);
        prune_map(&mut components.parameters, &used_params);
        prune_map(&mut components.responses, &used_responses);
        prune_map(&mut components.request_bodies, &used_bodies);
        prune_map(&mut components.headers, &used_headers);
        prune_map(&mut components.security_schemes, &used_security);
        prune_map(&mut components.examples, &used_examples);
        prune_map(&mut components.links, &used_links);
        prune_map(&mut components.callbacks, &used_callbacks);
    }
}

/// Extract all refs for a specific component namespace
fn filter_refs(refs: &HashSet<String>, prefix: &str) -> HashSet<String> {
    refs.iter()
        .filter_map(|r| r.strip_prefix(prefix).map(|s| s.to_string()))
        .collect()
}

/// Remove entries from OpenAPI component maps
fn prune_map<T>(map: &mut IndexMap<String, T>, keep: &HashSet<String>) {
    map.retain(|k, _| keep.contains(k));
}

#[cfg(test)]
mod tests {
    use super::*;
    use openapiv3::{OpenAPI, Paths, ReferenceOr, Schema, Type, SchemaKind, Components};
    use serde_json::json;

    fn create_minimal_spec() -> OpenAPI {
        OpenAPI {
            openapi: "3.0.0".to_string(),
            info: openapiv3::Info {
                title: "Test API".to_string(),
                version: "1.0.0".to_string(),
                ..Default::default()
            },
            paths: Paths::default(),
            ..Default::default()
        }
    }

    #[test]
    fn test_filter_paths_exact_match() {
        let mut spec = create_minimal_spec();
        spec.paths.paths.insert("/api/v1/users".to_string(), ReferenceOr::Item(openapiv3::PathItem::default()));
        spec.paths.paths.insert("/api/v1/posts".to_string(), ReferenceOr::Item(openapiv3::PathItem::default()));
        spec.paths.paths.insert("/api/v2/users".to_string(), ReferenceOr::Item(openapiv3::PathItem::default()));

        filter_paths(&mut spec, &["/api/v1/users".to_string()]);

        assert_eq!(spec.paths.paths.len(), 1);
        assert!(spec.paths.paths.contains_key("/api/v1/users"));
    }

    #[test]
    fn test_filter_paths_prefix_match() {
        let mut spec = create_minimal_spec();
        spec.paths.paths.insert("/api/v1/users".to_string(), ReferenceOr::Item(openapiv3::PathItem::default()));
        spec.paths.paths.insert("/api/v1/posts".to_string(), ReferenceOr::Item(openapiv3::PathItem::default()));
        spec.paths.paths.insert("/api/v2/users".to_string(), ReferenceOr::Item(openapiv3::PathItem::default()));

        filter_paths(&mut spec, &["/api/v1/*".to_string()]);

        assert_eq!(spec.paths.paths.len(), 2);
        assert!(spec.paths.paths.contains_key("/api/v1/users"));
        assert!(spec.paths.paths.contains_key("/api/v1/posts"));
        assert!(!spec.paths.paths.contains_key("/api/v2/users"));
    }

    #[test]
    fn test_filter_paths_mixed_patterns() {
        let mut spec = create_minimal_spec();
        spec.paths.paths.insert("/api/v1/users".to_string(), ReferenceOr::Item(openapiv3::PathItem::default()));
        spec.paths.paths.insert("/api/v1/posts".to_string(), ReferenceOr::Item(openapiv3::PathItem::default()));
        spec.paths.paths.insert("/api/v2/users".to_string(), ReferenceOr::Item(openapiv3::PathItem::default()));
        spec.paths.paths.insert("/special".to_string(), ReferenceOr::Item(openapiv3::PathItem::default()));

        filter_paths(&mut spec, &["/api/v1/*".to_string(), "/special".to_string()]);

        assert_eq!(spec.paths.paths.len(), 3);
        assert!(spec.paths.paths.contains_key("/api/v1/users"));
        assert!(spec.paths.paths.contains_key("/api/v1/posts"));
        assert!(spec.paths.paths.contains_key("/special"));
        assert!(!spec.paths.paths.contains_key("/api/v2/users"));
    }

    #[test]
    fn test_filter_paths_no_matches() {
        let mut spec = create_minimal_spec();
        spec.paths.paths.insert("/api/v1/users".to_string(), ReferenceOr::Item(openapiv3::PathItem::default()));
        spec.paths.paths.insert("/api/v1/posts".to_string(), ReferenceOr::Item(openapiv3::PathItem::default()));

        filter_paths(&mut spec, &["/api/v2/*".to_string()]);

        assert_eq!(spec.paths.paths.len(), 0);
    }

    #[test]
    fn test_filter_paths_empty_list() {
        let mut spec = create_minimal_spec();
        spec.paths.paths.insert("/api/v1/users".to_string(), ReferenceOr::Item(openapiv3::PathItem::default()));
        spec.paths.paths.insert("/api/v1/posts".to_string(), ReferenceOr::Item(openapiv3::PathItem::default()));

        filter_paths(&mut spec, &[]);

        assert_eq!(spec.paths.paths.len(), 0);
    }

    #[test]
    fn test_should_keep_exact() {
        assert!(should_keep("/api/v1/users", &["/api/v1/users".to_string()]));
        assert!(!should_keep("/api/v1/posts", &["/api/v1/users".to_string()]));
    }

    #[test]
    fn test_should_keep_prefix() {
        let keep_paths = vec!["/api/v1/*".to_string()];
        assert!(should_keep("/api/v1/users", &keep_paths));
        assert!(should_keep("/api/v1/posts", &keep_paths));
        assert!(should_keep("/api/v1/", &keep_paths));
        assert!(!should_keep("/api/v2/users", &keep_paths));
    }

    #[test]
    fn test_should_keep_no_match() {
        assert!(!should_keep("/api/v1/users", &["/api/v2/*".to_string()]));
        assert!(!should_keep("/different", &["/api/*".to_string()]));
    }

    #[test]
    fn test_collect_refs_simple() {
        // Create a minimal spec with a path that references a schema
        let json_data = json!({
            "openapi": "3.0.0",
            "info": {
                "title": "Test",
                "version": "1.0.0"
            },
            "paths": {
                "/api/users": {
                    "get": {
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
                }
            },
            "components": {
                "schemas": {
                    "User": {
                        "type": "object",
                        "properties": {
                            "id": {
                                "type": "integer"
                            }
                        }
                    }
                }
            }
        });

        if let Ok(spec) = serde_json::from_value::<OpenAPI>(json_data) {
            let refs = collect_refs(&spec);
            assert!(refs.contains("#/components/schemas/User"));
            assert_eq!(refs.len(), 1);
        } else {
            panic!("Failed to create test spec");
        }
    }

    #[test]
    fn test_collect_refs_nested() {
        let json_data = json!({
            "openapi": "3.0.0",
            "info": {
                "title": "Test",
                "version": "1.0.0"
            },
            "paths": {
                "/test": {
                    "get": {
                        "parameters": [{
                            "$ref": "#/components/parameters/TestParam"
                        }],
                        "responses": {
                            "200": {
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "allOf": [
                                                { "$ref": "#/components/schemas/Base" },
                                                { "$ref": "#/components/schemas/Extended" }
                                            ]
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        if let Ok(spec) = serde_json::from_value::<OpenAPI>(json_data) {
            let refs = collect_refs(&spec);
            assert!(refs.contains("#/components/parameters/TestParam"));
            assert!(refs.contains("#/components/schemas/Base"));
            assert!(refs.contains("#/components/schemas/Extended"));
            assert_eq!(refs.len(), 3);
        }
    }

    #[test]
    fn test_collect_refs_empty() {
        let spec = create_minimal_spec();
        let refs = collect_refs(&spec);
        assert_eq!(refs.len(), 0);
    }

    #[test]
    fn test_prune_components_schemas() {
        let mut spec = create_minimal_spec();
        let mut components = Components::default();
        
        let schema = Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(Type::String(Default::default())),
        };
        
        components.schemas.insert("User".to_string(), ReferenceOr::Item(schema.clone()));
        components.schemas.insert("Post".to_string(), ReferenceOr::Item(schema.clone()));
        components.schemas.insert("Comment".to_string(), ReferenceOr::Item(schema));
        
        spec.components = Some(components);

        let mut refs = HashSet::new();
        refs.insert("#/components/schemas/User".to_string());
        refs.insert("#/components/schemas/Post".to_string());

        prune_components(&mut spec, &refs);

        if let Some(components) = &spec.components {
            assert_eq!(components.schemas.len(), 2);
            assert!(components.schemas.contains_key("User"));
            assert!(components.schemas.contains_key("Post"));
            assert!(!components.schemas.contains_key("Comment"));
        }
    }

    #[test]
    fn test_prune_components_multiple_types() {
        let mut spec = create_minimal_spec();
        let mut components = Components::default();
        
        let schema = Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(Type::String(Default::default())),
        };
        
        components.schemas.insert("User".to_string(), ReferenceOr::Item(schema.clone()));
        components.schemas.insert("Unused".to_string(), ReferenceOr::Item(schema));
        
        let param = openapiv3::Parameter::Query {
            parameter_data: openapiv3::ParameterData {
                name: "page".to_string(),
                description: None,
                required: false,
                deprecated: None,
                format: openapiv3::ParameterSchemaOrContent::Schema(ReferenceOr::Item(Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(Type::Integer(Default::default())),
                })),
                example: None,
                examples: Default::default(),
                explode: None,
                extensions: Default::default(),
            },
            allow_reserved: false,
            style: openapiv3::QueryStyle::Form,
            allow_empty_value: None,
        };
        
        components.parameters.insert("PageParam".to_string(), ReferenceOr::Item(param.clone()));
        components.parameters.insert("UnusedParam".to_string(), ReferenceOr::Item(param));
        
        spec.components = Some(components);

        let mut refs = HashSet::new();
        refs.insert("#/components/schemas/User".to_string());
        refs.insert("#/components/parameters/PageParam".to_string());

        prune_components(&mut spec, &refs);

        if let Some(components) = &spec.components {
            assert_eq!(components.schemas.len(), 1);
            assert!(components.schemas.contains_key("User"));
            assert_eq!(components.parameters.len(), 1);
            assert!(components.parameters.contains_key("PageParam"));
        }
    }

    #[test]
    fn test_prune_components_all_removed() {
        let mut spec = create_minimal_spec();
        let mut components = Components::default();
        
        let schema = Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(Type::String(Default::default())),
        };
        
        components.schemas.insert("Unused1".to_string(), ReferenceOr::Item(schema.clone()));
        components.schemas.insert("Unused2".to_string(), ReferenceOr::Item(schema));
        
        spec.components = Some(components);

        let refs = HashSet::new();
        prune_components(&mut spec, &refs);

        if let Some(components) = &spec.components {
            assert_eq!(components.schemas.len(), 0);
        }
    }

    #[test]
    fn test_filter_refs_extracts_names() {
        let mut refs = HashSet::new();
        refs.insert("#/components/schemas/User".to_string());
        refs.insert("#/components/schemas/Post".to_string());
        refs.insert("#/components/parameters/PageParam".to_string());

        let schema_names = filter_refs(&refs, "#/components/schemas/");
        
        assert_eq!(schema_names.len(), 2);
        assert!(schema_names.contains("User"));
        assert!(schema_names.contains("Post"));
        assert!(!schema_names.contains("PageParam"));
    }

    #[test]
    fn test_prune_map_retains_used() {
        let mut map = IndexMap::new();
        map.insert("keep1".to_string(), 1);
        map.insert("remove1".to_string(), 2);
        map.insert("keep2".to_string(), 3);
        map.insert("remove2".to_string(), 4);

        let mut keep = HashSet::new();
        keep.insert("keep1".to_string());
        keep.insert("keep2".to_string());

        prune_map(&mut map, &keep);

        assert_eq!(map.len(), 2);
        assert!(map.contains_key("keep1"));
        assert!(map.contains_key("keep2"));
        assert!(!map.contains_key("remove1"));
        assert!(!map.contains_key("remove2"));
    }
}
