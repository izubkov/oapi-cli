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
