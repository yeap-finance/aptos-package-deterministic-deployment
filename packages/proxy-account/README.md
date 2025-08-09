# Proxy Account

A secure and manageable proxy account system for Aptos blockchain, providing deterministic resource account creation with two-step admin role management.

## Overview

This package implements a proxy account system that allows users to create deterministic resource accounts with secure admin role management. The system is designed for scenarios where you need predictable account addresses and secure privilege management.

## Features

- **Deterministic Proxy Creation**: Create resource accounts with predictable addresses using a seed-based approach
- **Two-Step Admin Transfer**: Secure admin role transfer mechanism inspired by OpenZeppelin's Ownable2Step
- **Resource Account Management**: Efficient management of Aptos resource accounts with signer capabilities
- **Event-Driven**: Comprehensive event emission for admin role changes and lifecycle management

## Modules

### `deterministic_proxy`

The core module for creating and managing deterministic proxy accounts.

**Key Functions:**
- `create(creator: &signer, proxy_name: vector<u8>): address` - Creates a new proxy account
- `proxy_address(creator: address, proxy_name: vector<u8>): address` - Calculates the deterministic address
- `generate_proxy_signer(admin: &signer, proxy: address): signer` - Generates a signer for the proxy account (admin only)

**Features:**
- Uses a deterministic seed (`proxy:deterministic_proxy` + custom name) for address generation
- Automatically sets up admin role management for the created proxy
- Returns the proxy account address for further interactions

### `manageable`

Provides secure admin role management with two-step transfer process.

**Key Functions:**
- `new(caller: &signer, admin: address)` - Initialize admin role for a resource
- `change_admin(caller: &signer, resource_address: address, new_admin: address)` - Start admin transfer
- `accept_admin(caller: &signer, resource_address: address)` - Complete admin transfer
- `admin(resource_address: address): address` - View current admin
- `pending_admin(resource_address: address): Option<address>` - View pending admin

**Security Features:**
- Two-step transfer process prevents accidental role transfer to inaccessible addresses
- Comprehensive access control with proper error handling
- Event emission for all role changes

## Usage

### Creating a Proxy Account

```move
use proxy::deterministic_proxy;

// Create a new proxy account
let proxy_addr = deterministic_proxy::create(&creator_signer, b"my_proxy");

// Calculate address without creating (for verification)
let expected_addr = deterministic_proxy::proxy_address(creator_address, b"my_proxy");

// Generate a signer for the proxy (only admin can do this)
let proxy_signer = deterministic_proxy::generate_proxy_signer(&admin_signer, proxy_addr);
```

### Managing Admin Roles

```move
use proxy::manageable;

// Check current admin
let current_admin = manageable::admin(proxy_address);

// Start admin transfer
manageable::change_admin(&current_admin_signer, proxy_address, new_admin_address);

// Accept admin role (must be called by new_admin)
manageable::accept_admin(&new_admin_signer, proxy_address);
```

## Events

The system emits the following events for transparency and monitoring:

- `AdminChangeStarted` - Emitted when admin transfer is initiated
- `AdminChanged` - Emitted when admin transfer is completed
- `AdminRoleDestroyed` - Emitted when admin role is destroyed

## Security Considerations

1. **Two-Step Transfer**: Admin role changes require two transactions, preventing accidental transfers
2. **Access Control**: All administrative functions verify caller permissions
3. **Deterministic Addresses**: Proxy addresses are predictable but collision-resistant
4. **Resource Safety**: Proper resource management with cleanup functions

## Dependencies

- **Aptos Framework**: Uses `account`, `event`, and standard library modules
- **Move Standard Library**: Utilizes `option`, `signer`, and `vector` modules

## Building and Testing

### Prerequisites

- Aptos CLI installed
- Move compiler

### Build

```bash
aptos move compile
```

### Deploy

```bash
# Update Move.toml with your address
aptos move publish --named-addresses proxy=<your_address>
```

## Configuration

Update `Move.toml` with your deployment address:

```toml
[addresses]
proxy = "YOUR_DEPLOYMENT_ADDRESS"
```

For development:

```toml
[dev-addresses]
proxy = "0x10"  # or your preferred dev address
```

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Contributing

This package is developed by Yeap Labs. Contributions following the established code style and security practices are welcome.

## Version History

- **1.0.0**: Initial release with deterministic proxy creation and manageable admin roles

---

**Author**: oneke@yeap.finance
**Organization**: Yeap Labs
