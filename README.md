# oapi-cli

OpenAPI spec filter - keeps only paths with a given prefix and removes unused components.

## Usage

```bash
cargo run -- input.yaml output.json --prefix /api/v1/examples
```

## Build

```bash
cargo build --release
```

