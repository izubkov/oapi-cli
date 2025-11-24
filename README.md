# oapi-cli

CLI tool for filtering and pruning OpenAPI specs.

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
