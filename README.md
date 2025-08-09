# Yeaptor

Yeaptor is a set of Aptos Move packages and lightweight tools to help developers build, deploy, and manage smart contracts on Aptos with predictable addresses and safer upgrade paths.

## What’s inside
- packages/object-code-deterministic-deployment
  Deterministically publish Move packages to Aptos Objects, with support for upgrades and freezing, plus a view to pre-compute the object address.
- packages/proxy-account
  Create deterministic proxy (resource) accounts with admin-controlled signer generation and a secure two-step admin transfer flow via the accompanying `manageable` module.

## Why Yeaptor
- Deterministic addresses: Predict where code or resource accounts will live before deployment (useful for integrations and off-chain references).
- Controlled upgrades: Upgrade packages deployed to objects, or freeze them to make them immutable.
- Operational safety: Two-step admin handover for proxies reduces risks from accidental key transfers.
- Composability: Patterns designed to compose with existing Aptos Framework primitives.

## Typical use cases
- Protocol-owned “factories” and modules at stable, pre-known addresses
- Managed proxy accounts for automation, custody, or delegated execution
- Cross-environment coordination where addresses must be known ahead of time
- Safer lifecycle: upgrade during rollout, freeze when finalized

## Getting started
- Requires Aptos CLI and an initialized profile
- Build and test each package separately in `packages/*`
- See each package’s Move sources and README for details and examples

## Project status
These modules target the Aptos mainnet framework revision defined in each `Move.toml`. Review, testing, and audits are recommended before production use.

## License
Apache-2.0. See `LICENSE`.
