use clap::Parser;
use indexmap::IndexMap;
use openapiv3::OpenAPI;
use serde_json::Value;
use std::{collections::HashSet, fs, path::PathBuf};

#[derive(Parser)]
struct Args {
    /// OpenAPI file (JSON or YAML)
    input: PathBuf,
    /// Output file (JSON)
    output: PathBuf,
    /// Only keep paths starting with this prefix
    #[arg(long)]
    prefix: String,
}

fn main() {
    let args = Args::parse();

    // Load JSON or YAML
    let raw = fs::read_to_string(&args.input).expect("Failed to read input file");
    let mut spec: OpenAPI = if args.input.extension().unwrap() == "json" {
        serde_json::from_str(&raw).expect("Invalid JSON")
    } else {
        serde_yaml::from_str(&raw).expect("Invalid YAML")
    };

    // 1. Filter paths
    spec.paths.paths = spec
        .paths
        .paths
        .into_iter()
        .filter(|(path, _)| path.starts_with(&args.prefix))
        .collect();

    // 2. Collect referenced components
    let mut refs = HashSet::new();
    collect_refs(&serde_json::to_value(&spec).unwrap(), &mut refs);

    // Partition refs by component type
    let used_schemas = filter_refs(&refs, "#/components/schemas/");
    let used_params = filter_refs(&refs, "#/components/parameters/");
    let used_responses = filter_refs(&refs, "#/components/responses/");
    let used_bodies = filter_refs(&refs, "#/components/requestBodies/");
    let used_headers = filter_refs(&refs, "#/components/headers/");
    let used_security = filter_refs(&refs, "#/components/securitySchemes/");
    let used_examples = filter_refs(&refs, "#/components/examples/");
    let used_links = filter_refs(&refs, "#/components/links/");
    let used_callbacks = filter_refs(&refs, "#/components/callbacks/");

    // 3. Prune unused components
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

    // Save output as JSON
    fs::write(
        args.output,
        serde_json::to_string_pretty(&spec).unwrap(),
    )
    .unwrap();
}

/// Recursively collect all `$ref` occurrences
fn collect_refs(value: &Value, refs: &mut HashSet<String>) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(r)) = map.get("$ref") {
                refs.insert(r.clone());
            }
            for v in map.values() {
                collect_refs(v, refs);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                collect_refs(v, refs);
            }
        }
        _ => {}
    }
}

/// Extract all refs for a specific component namespace
fn filter_refs(refs: &HashSet<String>, prefix: &str) -> HashSet<String> {
    refs.iter()
        .filter_map(|r| r.strip_prefix(prefix).map(|s| s.to_string()))
        .collect()
}

fn prune_map<T>(map: &mut IndexMap<String, T>, keep: &HashSet<String>) {
    map.retain(|k, _| keep.contains(k));
}

// /// Remove entries from OpenAPI component maps
// fn prune_map<T>(map: &mut std::collections::BTreeMap<String, T>, keep: &HashSet<String>) {
//     map.retain(|k, _| keep.contains(k));
// }
