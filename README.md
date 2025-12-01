# oapi-cli

CLI tool for modifying, filtering and pruning OpenAPI specs. Especially useful when working with large OpenAPI specs, `oapi-cli` is a command-line tool that filters, prunes, and extracts only the paths and components you need. Ideal for preparing compact specs for AI agents and LLM workflows.

## Usage

### Filtering

Keeps only paths with a given prefix and removes unused components.

```bash
# keep only exact path
oapi-cli filter -i openapi.yaml -o openapi-no-example.yaml --path /api/v1/example

# keep all paths starting with /api/v1/
oapi-cli filter -i openapi.yaml -o openapi-no-example.yaml --path "/api/v1/example/*"
```

## Build

```bash
cargo build --release
```
