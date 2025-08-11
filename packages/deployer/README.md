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
- The framework calls `init_module(resource_account)` after publish (not an entry):
  - Retrieves and stores the resource account SignerCapability in `DeployCap`
  - Initializes admin management so only the initial sender can operate the deployer
- Subsequent calls require the admin and derive the deployer signer internally.

## Public entry functions
- publish_package(admin, seed, metadata_serialized, code)
  Publish a package to a deterministic object address.
- upgrade_package(admin, seed, metadata_serialized, code)
  Upgrade a package; derives the code object from the seed.
- freeze_package(admin, seed)
  Freeze a package; derives the code object from the seed.
- upgrade_code_object(admin, metadata_serialized, code, code_object)
  Upgrade by passing an explicit `Object<PackageRegistry>`.
- freeze_code_object(admin, code_object)
  Freeze by passing an explicit `Object<PackageRegistry>`.
- code_object_address(seed) -> address [view]
  Pre-compute the object address for a given seed (publisher is the deployer resource account).
- retire(admin)
  Remove admin state and destroy the stored capability to decommission the deployer.

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

## CLI tips
- Use --dev for local builds/tests so dev-addresses resolve:
  aptos move compile --dev
  aptos move test --dev
- For publish/upgrade, pass package-metadata.bcs and module bytecode blobs; most users will call through an off-chain tool that encodes vector<vector<u8>>.

## Security notes
- Only the admin can operate the deployer; admin transfers use the two-step manageable flow.
- Freezing is irreversible.

## Dependencies
- AptosFramework (mainnet rev)
- object-code-deterministic-deployment (local dependency)