# Yeaptor CLI

Yeaptor is a small CLI that turns a declarative deployment plan (yeaptor.toml) into ready-to-run Aptos entry-function JSON payloads. Use these JSON files with the Aptos CLI to publish Move packages to deterministic resource accounts.

What it does
- Reads a single configuration file: yeaptor.toml
- Computes a resource account address for each deployment: create_resource_address(publisher, seed)
- Builds each listed Move package with the Aptos builder, injecting named addresses so that every package’s `address_name` resolves to the computed resource account address
- Emits one JSON file per package that calls `<yeaptor_address>::ra_code_deployment::deploy(seed, metadata, modules)`

Prerequisites
- Rust toolchain (stable)
- Aptos CLI installed and configured (profiles, network, etc.)

Install
- From repository root: `cargo install --path crates/yeaptor`
- Or run without installing: `cargo run -p yeaptor -- <args>`

Quick start
1) Author `crates/yeaptor/yeaptor.toml.example` (see Configuration below).
2) Generate payloads:
   - `yeaptor deployment build --config ./yeaptor.toml --out-dir ./deployments`
3) Execute each payload with Aptos CLI in the generated order:
   - `aptos move run --profile <profile> --json-file ./deployments/0-<package>-publish.json`

Command synopsis
- Subcommand: `yeaptor deployment build`
  - `--config <PATH>`: Path to yeaptor.toml (default: ./yeaptor.toml)
  - `--out-dir <PATH>`: Directory for JSON output (default: ./deployments)
  - Accepts standard Aptos Move build flags via the underlying builder

Configuration (yeaptor.toml)
- format_version: Schema version. Use 1
- yeaptor_address: On-chain address where the module `ra_code_deployment` is published
- [publishers]: Map of alias -> on-chain address. Referenced by deployments.publisher
- [named-addresses] (optional): Extra Move named addresses shared across packages
- [[deployments]]: Ordered deployments. Each defines one resource account derived from (publisher + seed) and the ordered packages to publish into it
  - publisher: Alias from [publishers] or a literal on-chain address string
  - seed: UTF-8 text used to deterministically derive the resource account (hex not allowed)
  - packages: Array of objects { address_name, path }
    - address_name: The Move named address used by that package (will resolve to the derived resource account)
    - path: Filesystem path to the Move package (containing Move.toml)

Example
```
format_version = 1
yeaptor_address = "0x1"

[publishers]
yeap-multisig = "0x10"

[named-addresses]
# std = "0x1"

[[deployments]]
publisher = "yeap-multisig"
seed = "core-v1"            # UTF-8 only
packages = [
  { address_name = "ra_code_deployment", path = "packages/resource-account-code-deployment" },
  { address_name = "proxy_account",      path = "packages/proxy-account" },
]
```

Generated outputs
- Files are written to `--out-dir` in deployment order, one per package: `<index>-<package>-publish.json`
- Each file contains an Aptos entry-function JSON payload like:
```
{
  "function_id": "0x<yeaptor_address>::ra_code_deployment::deploy",
  "type_args": [],
  "args": [
    { "type": "hex", "value": "0x<seed-bytes>" },
    { "type": "hex", "value": "0x<metadata-bcs>" },
    { "type": "hex", "value": ["0x<module-1>", "0x<module-2>", ...] }
  ]
}
```
Run with Aptos CLI
- Execute each JSON in order:
  - `aptos move run --profile <profile> --json-file ./deployments/<n>-<pkg>-publish.json`

Notes
- Order matters: deployments and the packages within them are processed sequentially
- `seed` must be UTF-8 text (not hex) to ensure a consistent resource address derivation
- `address_name` must match the named address used in the package’s Move.toml
- `yeaptor_address` must be the on-chain address hosting the `ra_code_deployment` module
