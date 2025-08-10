# Yeaptor

Yeaptor is a set of Aptos Move packages and lightweight tools to help developers build, deploy, and manage smart contracts on Aptos with predictable addresses and safer upgrade paths.

## Highlights
- Deterministic addresses for code and resource accounts
- Safer upgrade lifecycle with explicit freeze mechanics
- Admin-gated orchestration via a resource-account deployer
- Composable, framework-aligned Move patterns

## What’s inside
- [packages/object-code-deterministic-deployment](packages/object-code-deterministic-deployment/README.md)
  Deterministically publish Move packages to Aptos Objects, with support for upgrades and freezing, plus a view to pre-compute the object address.
- [packages/proxy-account](packages/proxy-account/README.md)
  Create deterministic proxy (resource) accounts with admin-controlled signer generation and a secure two-step admin transfer flow via the accompanying `manageable` module.
- [packages/deployer](packages/deployer/README.md)
  Resource-account deployer that admin-gates deterministic publish/upgrade/freeze to Aptos Objects. Provides seed-based convenience flows and explicit object variants.

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

## 5‑minute quickstart
1) Prerequisites: Aptos CLI installed and a profile initialized (devnet/testnet/mainnet)
2) Build the packages (use --dev for local dev addresses):
   - cd packages/object-code-deterministic-deployment && aptos move compile --dev
   - cd packages/proxy-account && aptos move compile --dev
   - cd packages/deployer && aptos move compile --dev
3) Explore package READMEs for usage and API details
   - Object deployment: deterministic publish/upgrade/freeze to objects
   - Proxy accounts: create/generate admin-gated resource account signers
   - Deployer: resource-account admin orchestrates deterministic deployments

## Account relationships (who owns/controls what)
```mermaid
graph LR
  %% Actors
  subgraph Actors
    Admin["Admin (EOA)"]
    Pending[Pending Admin]
  end

  %% Deployer resource account and on-chain resources
  subgraph "Deployer(Resource Account)"
    RA["@package_deployer"]
    DeployCap[(DeployCap)]
    AdminRole[(manageable::AdminRole)]
  end

  %% Code Objects produced by deterministic publish
  subgraph Code Objects
    O1{{Object O1}}:::obj
    O2{{Object O2}}:::obj
    PR1[(PackageRegistry)]:::res
    PR2[(PackageRegistry)]:::res
  end

  %% Admin flow
  Admin -- change_admin(new_admin) --> AdminRole
  Pending -- accept_admin() --> AdminRole
  AdminRole -- admin_of --> RA
  RA -- stores --> DeployCap

  %% Calls and signer derivation
  Admin -- calls publish/upgrade/freeze --> RA
  RA -. derive signer via DeployCap .-> RA

  %% Deterministic address derivation
  SeedA[(seed A)]
  SeedB[(seed B)]
  F["create_code_object_address(@RA, seed)"]
  SeedA --> F
  SeedB --> F
  RA --> F
  F -- address --> O1
  F -- address --> O2

  %% Ownership and contents
  RA -- owner_of --> O1
  RA -- owner_of --> O2
  O1 -- contains --> PR1
  O2 -- contains --> PR2

  classDef obj fill:#eef,stroke:#66f,stroke-width:1px;
  classDef res fill:#efe,stroke:#6a6,stroke-width:1px;
```

## Deployment diagrams

### 1) Create Deployer via Resource-account publish and init flow
```mermaid
sequenceDiagram
    participant Dev as Developer
    participant CLI as Aptos CLI
    participant FW as Aptos Framework
    participant Dep as package_deployer::deployer
    Dev->>CLI: aptos move create-resource-account-and-publish-package
    CLI->>FW: Publish package under resource account
    FW->>Dep: Invoke init_module(resource_account)
    Dep->>FW: resource_account::retrieve_resource_account_cap
    Dep->>Dep: move_to<DeployCap>(cap)
    Dep->>Dep: manageable::new(resource_account, sender)
    Note over Dep: sender becomes initial admin
```

### 2) Deterministic publish via deployer
```mermaid
sequenceDiagram
    participant Admin
    participant Dep as package_deployer::deployer
    participant OCD as object_code_deterministic_deployment::deployment
    participant Obj as Aptos Object
    Admin->>Dep: publish_package(seed, metadata, code)
    Dep->>Dep: deployer_signer(admin) (assert_is_admin + create_signer_with_capability)
    Dep->>OCD: deterministic_publish(deployer, seed, metadata, code)
    OCD->>OCD: create_named_object + generate_signer
    OCD->>Obj: code::publish_package_txn(metadata, code)
    OCD-->>Admin: Publish event(object_address)
```

### 3) Upgrade/freeze via seed wrappers
```mermaid
flowchart TD
    A[Admin] --> B["upgrade_package(seed, metadata, code)"]
    B --> C["deployer_signer(admin)"]
    C --> D["addr = code_object_address(seed)"]
    D --> E["address_to_object<PackageRegistry>(addr)"]
    E --> F["ocd::upgrade(..., code_object)"]
    F --> G[[Upgrade event]]

    A --> H["freeze_package(seed)"]
    H --> C
    C --> D2["addr = code_object_address(seed)"]
    D2 --> E2["address_to_object<PackageRegistry>(addr)"]
    E2 --> I["ocd::freeze_code_object(...)"]
    I --> J[[Freeze event]]
```


## Getting started
- Requires Aptos CLI and an initialized profile
- Build and test each package separately in `packages/*`
- See each package’s Move sources and README for details and examples

## Who is this for?
- Protocol teams needing predictable addresses and controlled rollouts
- Wallets, custodians, and automation systems managing delegated execution
- Developers standardizing on modern Aptos object and resource-account patterns

## Roadmap
- Turnkey scripts/SDK for packaging metadata/code and submitting transactions
- Example dapps and e2e test flows for testnet/mainnet
- Optional deployer-level events and richer views for observability

## Project status
These modules target the Aptos mainnet framework revision defined in each `Move.toml`. Review, testing, and audits are recommended before production use.

## Contributing
Issues and PRs are welcome. Please include clear repro steps and tests where possible.

## License
Apache-2.0. See `LICENSE`.
