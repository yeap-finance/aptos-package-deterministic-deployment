# Repository Instructions for Copilot

## Project overview
- Yeaptor provides a Rust CLI and Move packages to deterministically deploy and upgrade Aptos Move code to resource accounts (and related patterns) with admin gating and optional freeze.
- Primary focus: `crates/yeaptor` (CLI) and `packages/resource-account-code-deployment` (Move module `ra_code_deployment::ra_code_deployment`).

## Folder structure
- `crates/yeaptor/` — Rust CLI.
  - `src/lib.rs` (CLI wiring), `src/main.rs` (runtime), `src/deployment.rs` (payload generation), `src/config.rs` (TOML schema), `src/version.rs` (version subcommand).
  - `tests/` — config parsing tests.
  - `build.rs` — injects GIT_DESCRIBE/BUILD_DATE/BUILD_TARGET env vars.
- `packages/resource-account-code-deployment/` — Move package with entry fns: `create_resource_account`, `deploy`, `batch_deploy`, `publish`, `batch_publish`, `freeze_resource_account`.
  - `Move.toml`, `sources/resource_account_deployment.move`.
- `packages/proxy-account/` — Move package (proxy/resource account patterns).
- `packages/object-code-deterministic-deployment/` — Move package (object-based deterministic deployment).
- `yeaptor.toml.example` — example deployment config used by the CLI.
- `.github/workflows/` — CI (`yeaptor-ci.yml`) and release (`release.yml`).

## Libraries and tools
- Rust (edition 2024). Key crates: `tokio=1.43` (full), `clap=4.5.31` (derive), `anyhow`, `serde`/`serde_json`, `toml`, `hex`, `async-trait`, `jemallocator` (unix).
- Aptos deps via git (branch `mainnet`): `aptos`, `aptos-framework`, `aptos-cli-common`, `aptos-types`.
- BCS: `bcs` from aptos-labs (pinned rev).
- Move deps in packages use `AptosFramework` and `AptosExtensions` (see each `Move.toml`).

## Coding standards
- Forbid unsafe code in Rust (`#![forbid(unsafe_code)]`); prefer `anyhow::Context` for rich errors.
- Keep CLI args with `clap` derive; follow existing style from `aptos_cli_common::aptos_cli_style()`.
- Use `BTreeMap` for deterministic ordering of addresses and config maps.
- Serialize addresses with `to_standard_string()`; seeds are UTF‑8 text (not hex) for `create_resource_address`.
- When changing config schema, update `src/config.rs`, tests in `tests/`, and `yeaptor.toml.example` together.
- Keep JSON payload shape aligned with Move entry functions (`deploy`/`batch_deploy` arguments).

## Build, test, and CI
- Build/test the CLI from `crates/yeaptor` using `cargo build` / `cargo test`.
- CI (`yeaptor-ci.yml`): runs on PR and manual dispatch; matrix includes `macos-14`; caches Cargo; builds and tests the `yeaptor` crate.
- Release (`release.yml`): triggers on `v*` tags or manual; builds release binaries for macOS and Linux, uploads artifacts, and creates a GitHub Release.
- The `version` subcommand prints package version and build metadata from `build.rs` env vars.

## Contributions
- Keep the root `README.md` aligned with the CLI behavior and the `resource-account-code-deployment` API.
- If adding new Move entry functions or changing parameters, update the CLI payload generator accordingly and extend tests.
- Prefer small, focused changes with tests; run `cargo fmt` and `cargo clippy` locally where applicable.
