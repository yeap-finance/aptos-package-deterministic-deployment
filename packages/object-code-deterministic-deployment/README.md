# Object Code Deterministic Deployment

A Move package for deploying, upgrading, and freezing smart contract modules to objects on the Aptos blockchain with deterministic addresses.

## Overview

This package provides an alternative method to publish code on-chain, where modules are deployed to objects rather than directly to accounts. This approach offers several advantages:

- **Deterministic addresses**: Generate predictable object addresses for code deployment
- **Abstracted resource management**: Simplifies the resources needed for deploying modules
- **Granular access control**: Enhanced authorization for upgrades and freeze operations
- **Object-based architecture**: Leverages Aptos object model for better code organization

## Features

### 📦 Publishing Modules
- Deploy modules to newly created objects with deterministic addresses
- Generate predictable object addresses derived from publisher address and custom seed
- Automatic creation of management references for future upgrades
- No additional permission checks required beyond basic object code deployment feature

### 🔄 Upgrading Modules
- Upgrade existing modules deployed to objects
- Owner-only access control for upgrades
- Seamless integration with existing object ownership model

### 🔒 Freezing Modules
- Make modules immutable to prevent future upgrades
- One-way operation - frozen modules cannot be unfrozen
- Enhanced security for production deployments

## Installation

Add this package to your `Move.toml` dependencies:

```toml
[dependencies.ObjectCodeDeterministicDeployment]
git = "https://github.com/your-org/aptos-package-deterministic-deployment.git"
rev = "main"
```

## Usage

### Publishing Code

Deploy modules to a new object with a deterministic address:

```move
use object_code_deterministic_deployment::deployment;

public entry fun deploy_my_modules(publisher: &signer) {
    let seed = b"my_unique_seed_v1";
    let metadata = /* serialized package metadata */;
    let code = /* vector of compiled modules */;

    deployment::deterministic_publish(
        publisher,
        seed,
        metadata,
        code
    );
}
```

### Predicting Object Address

Calculate the object address before deployment:

```move
use object_code_deterministic_deployment::deployment;

public fun get_deployment_address(publisher_addr: address): address {
    let seed = b"my_unique_seed_v1";
    deployment::create_code_object_address(publisher_addr, seed)
}
```

### Upgrading Code

Upgrade existing modules (requires ownership):

```move
use object_code_deterministic_deployment::deployment;

public entry fun upgrade_my_modules(
    publisher: &signer,
    code_object: Object<PackageRegistry>
) {
    let metadata = /* new serialized package metadata */;
    let code = /* new vector of compiled modules */;

    deployment::upgrade(
        publisher,
        metadata,
        code,
        code_object
    );
}
```

### Freezing Code

Make modules immutable (irreversible):

```move
use object_code_deterministic_deployment::deployment;

public entry fun freeze_my_modules(
    publisher: &signer,
    code_object: Object<PackageRegistry>
) {
    deployment::freeze_code_object(publisher, code_object);
}
```

## API Reference

### Public Functions

#### `deterministic_publish`
```move
public entry fun deterministic_publish(
    publisher: &signer,
    deterministic_object_seed: vector<u8>,
    metadata_serialized: vector<u8>,
    code: vector<vector<u8>>,
)
```
Deploys modules to a new object with deterministic address.

**Parameters:**
- `publisher`: The signer deploying the code
- `deterministic_object_seed`: Custom seed for address generation
- `metadata_serialized`: Serialized package metadata
- `code`: Vector of compiled module bytecode

#### `create_code_object_address`
```move
public fun create_code_object_address(
    publisher: address,
    seed: vector<u8>
): address
```
Calculates the deterministic object address for given publisher and seed.

#### `upgrade`
```move
public entry fun upgrade(
    publisher: &signer,
    metadata_serialized: vector<u8>,
    code: vector<vector<u8>>,
    code_object: Object<PackageRegistry>,
)
```
Upgrades modules in an existing code object (owner only).

#### `freeze_code_object`
```move
public entry fun freeze_code_object(
    publisher: &signer,
    code_object: Object<PackageRegistry>
)
```
Makes all modules in the object immutable (owner only, irreversible).

### Events

- **`Publish`**: Emitted when code is published to a new object
- **`Upgrade`**: Emitted when existing code is upgraded
- **`Freeze`**: Emitted when code is made immutable

## Error Codes

- `EOBJECT_CODE_DEPLOYMENT_NOT_SUPPORTED (1)`: Object code deployment feature not enabled
- `ENOT_CODE_OBJECT_OWNER (2)`: Caller is not the owner of the code object
- `ECODE_OBJECT_DOES_NOT_EXIST (3)`: The specified code object does not exist

## Prerequisites

- Aptos CLI installed
- Move compiler
- Object code deployment feature must be enabled on the network

## Development

### Building

```bash
aptos move compile
```

### Testing

```bash
aptos move test
```

### Publishing

```bash
aptos move publish
```

## Use Cases

- **Multi-version deployments**: Deploy different versions of contracts to separate objects
- **Modular architecture**: Organize related modules in separate objects
- **Predictable addresses**: Generate known addresses for contract interaction
- **Upgrade governance**: Implement custom upgrade authorization logic
- **Production deployments**: Freeze critical modules for security

## Security Considerations

- Only object owners can upgrade or freeze code
- Frozen modules cannot be unfrozen - this is permanent
- Deterministic addressing allows prediction of deployment addresses
- Ensure proper access control for upgrade operations

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

---

**Note**: This package requires the object code deployment feature to be enabled on the Aptos network you're deploying to.
