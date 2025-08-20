# Yeaptor CLI

Yeaptor is a small CLI that turns a declarative deployment plan (yeaptor.toml) into ready-to-run Aptos entry-function JSON payloads. Use these JSON files with the Aptos CLI to publish Move packages to deterministic resource accounts.

What it does
- Reads a single configuration file: yeaptor.toml
- Computes a resource account address for each deployment: create_resource_address(publisher, seed)
- Builds each listed Move package with the Aptos builder, injecting named addresses so that every package’s `address_name` resolves to the computed resource account address
- Emits one JSON file per package that calls `<yeaptor_address>::ra_code_deployment::deploy(seed, metadata, modules)`
- Optionally generates per‑package event definition JSON files from compiled Move packages (via `yeaptor event definition` or `deployment build --with-event`)
- Generates (does not run) a processor configuration YAML from event definitions and CSV inputs (`db_schema.csv`, `event_mappings.csv`) via `yeaptor processor generate`

Prerequisites
- Rust toolchain (stable)
- Aptos CLI installed and configured (profiles, network, etc.)

Install
- From repository root: `cargo install --path crates/yeaptor`
- Or run without installing: `cargo run -p yeaptor -- <args>`

Quick start
1) Author `yeaptor.toml` (see Configuration below).
2) Generate payloads:
   - `yeaptor deployment build --config ./yeaptor.toml --out-dir ./deployments`
   - Optional: include event definitions alongside payloads
     - `yeaptor deployment build --config ./yeaptor.toml --out-dir ./deployments --with-event`
3) Execute each payload with Aptos CLI in the generated order:
   - `aptos move run --profile <profile> --json-file ./deployments/0-<package>.package.json`

CLI commands

### yeaptor deployment build
Build publish payload JSON files for packages defined in `yeaptor.toml`. Optionally emit per‑package event definition JSON files.

- Flags
  - `--config <PATH>`: Path to `yeaptor.toml` (default: `./yeaptor.toml`)
  - `--out-dir <PATH>`: Output directory (default: `./deployments`)
  - `--with-event`: Also write event definition JSON files to `<out-dir>/events/`
  - Standard Aptos Move build flags via the underlying builder (e.g. `--package-dir` to build a single package)
- Examples
  - All deployments: `yeaptor deployment build --config ./yeaptor.toml --out-dir ./deployments`
  - Single package: `yeaptor deployment build --config ./yeaptor.toml --out-dir ./deployments --package-dir ./packages/proxy-account`
  - With events: `yeaptor deployment build --config ./yeaptor.toml --out-dir ./deployments --with-event`
- Outputs
  - `<out-dir>/<index>-<package>.package.json` publish payloads
  - `<out-dir>/events/<package>.event.json` (when `--with-event`)
  - `<out-dir>/addresses.toml` resolved named addresses

### yeaptor event definition
Generate event definition JSON files from compiled Move packages.

- Behavior
  - If `--package-dir` is provided, generates for that package only; otherwise scans packages referenced in `yeaptor.toml`.
- Flags
  - `--config <PATH>`: Path to `yeaptor.toml` (default: `./yeaptor.toml`)
  - `--out-dir <PATH>`: Output directory for event JSON (default: `./events`)
  - Standard Aptos Move build flags (e.g. `--package-dir <PATH>`)
- Examples
  - All from config: `yeaptor event definition --config ./yeaptor.toml --out-dir ./events`
  - Single package: `yeaptor event definition --config ./yeaptor.toml --out-dir ./events --package-dir ./packages/proxy-account`

### yeaptor processor generate
Generate (not run) a processor configuration YAML from event definitions and a DB schema + event‑to‑table mapping.

- Inputs
  - Event definitions directory (JSON files): `--events-dir` (default: `./events`)
  - Database schema CSV: `--db_schema` (default: `./db_schema.csv`)
  - Event‑to‑table mapping CSV: `--event_mapping` (default: `./event_mapping.csv`)
- Required flags
  - `--starting-version <u64>`: Starting version to use in the generated config
- Optional flags
  - `--network <testnet|mainnet|devnet|...>`: Target network (default: `testnet`)
  - `--output-file <PATH>`: Output YAML path (default: `./processor_config.yaml`)
- Example
  - `yeaptor processor generate --starting-version 123456 --events-dir ./events --db_schema ./db_schema.csv --event_mapping ./event_mapping.csv --output-file ./processor_config.yaml`

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

Generated publish payload shape
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
  - `aptos move run --profile <profile> --json-file ./deployments/<n>-<pkg>.package.json`

Notes
- Order matters: deployments and the packages within them are processed sequentially
- `seed` must be UTF-8 text (not hex) to ensure a consistent resource address derivation
- `address_name` must match the named address used in the package’s Move.toml
- `yeaptor_address` must be the on-chain address hosting the `ra_code_deployment` module
- The processor subcommand only generates the YAML; it does not run an indexer. You can consume the YAML in your own processor.

## CSV formats

### db_schema.csv
- Purpose: declare all tables/columns and their types/flags to build the processor schema.
- File: CSV with a header row; cells are trimmed (spaces around commas are OK).
- Required header columns (exact names):
  - table: string; table name
  - column: string; column name
  - column_type: string; semantic type name
    - For data: u8, u64, u128, address, bool, object, etc.
    - For transaction metadata: block_height, epoch, timestamp, version
    - For event metadata: account_address, creation_number, event_index, event_type, sequence_number
  - type: string; one of
    - move_type: regular event field types (u64, address, bool, object, ...)
    - transaction_metadata: metadata columns listed above
    - event_metadata: metadata columns listed above
  - default_value: optional; parsed as YAML and typed based on type/column_type (numbers as numbers, bools, strings; arrays/JSON if provided)
  - is_index: bool; true/false (accepts t/1/yes/y)
  - is_nullable: bool
  - is_option: bool
  - is_primary_key: bool
  - is_vec: bool
- Semantics:
  - Each row defines a single column. Rows with the same table accumulate into that table schema.
  - Transaction and event metadata columns are auto-mapped by the generator from their type/column_type; no event mapping entry is required.
  - Event field mapping will auto-target columns with the same name in mapped tables; use event_mappings.csv to override.
- Example rows:
  - adaptive_irm_activities, event_index, event_index, event_metadata, , False, False, False, True, False
  - adaptive_irm_activities, timestamp, timestamp, transaction_metadata, , True, True, False, False, False
  - fixed_price_oracle_current_config, price, u128, move_type, , False, True, False, False, False

### event_mappings.csv
- Purpose: map events and/or specific event fields to destination tables/columns.
- File: CSV with a header row (first row is ignored by the loader); two columns: event, table.
- Event column forms:
  - package::module::EventName — full-event mapping
  - package::module::EventName::field_name — field-specific mapping
- Table column forms:
  - For full events: table_name — enables auto field mapping by name for all fields into that table
  - For field mappings: table_name::column_name — explicit column target when names differ
- Semantics:
  - Multiple rows per event are allowed to map into multiple tables or columns; duplicates are deduplicated; order is normalized.
  - Transaction/event metadata do not require mappings here; they’re auto-mapped via db_schema.csv.
- Examples:
  - yeap-irm::fixed_rate_irm::ConfigChangedEvent, fixed_rate_irm_activities
  - yeap-irm::fixed_rate_irm::ConfigChangedEvent, fixed_rate_irm_current_config
  - yeap-vault::fee_setting::FeeSettingCreatedEvent::interest_fee, vault_settings::interest_fee_rate
  - yeap-borrow-protocol-common::market::MarketCreatedEvent::collateral_asset, borrow_market::collateral
