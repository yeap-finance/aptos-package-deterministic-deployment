# Package Deployer

`package-deployer` is an on-chain resource-account package that lets you deterministically publish, upgrade, and freeze Move packages using Aptos Objects. It wraps the lower-level `object_code_deterministic_deployment::deployment` API and adds admin-gated operations through a managed resource account.

## Key features
- Deterministic object address deployment using a seed
- Admin-gated deploy/upgrade/freeze via a resource account
- Two-step admin transfer (manageable module semantics)
- View to pre-compute code object address for a seed

## How it works
- Publish this package with the CLI command:
  aptos move create-resource-account-and-publish-package --dev
- The framework calls `init_module(resource_account)` after publish. This function (not an entry):
  - Retrieves and stores the resource account SignerCapability in `DeployCap`
  - Initializes admin management so only the initial sender can operate the deployer
- Subsequent calls to `publish`, `upgrade`, and `freeze_code_object` require the admin
  and internally derive the deployer signer from the stored capability.

## Public entry functions
- publish(admin: &signer, deterministic_object_seed: vector<u8>, metadata_serialized: vector<u8>, code: vector<vector<u8>>)
  Publish a package to a deterministic object address.
- upgrade(admin: &signer, metadata_serialized: vector<u8>, code: vector<vector<u8>>, code_object: Object<PackageRegistry>)
  Upgrade an existing upgradable package.
- freeze_code_object(admin: &signer, code_object: Object<PackageRegistry>)
  Make a package immutable.
- code_object_address(seed: vector<u8>) -> address [view]
  Pre-compute the object address for a given seed (publisher is the deployer resource account).
- ungov(admin: &signer)
  Remove admin state and destroy the stored capability to decommission the deployer.

## CLI tips
- Use --dev for local builds/tests so dev-addresses resolve:
  aptos move compile --dev
  aptos move test --dev
- After building a package you want to deploy, collect its package-metadata.bcs and bytecode_modules/*.mv bytes and pass them as arguments to `publish` (typically via a client/tool that encodes vector<vector<u8>>).

## Security notes
- Only the admin can operate the deployer; transfer uses the two-step manageable flow.
- Freezing is irreversible.

## Dependencies
- AptosFramework (mainnet rev)
- object-code-deterministic-deployment (local dependency)