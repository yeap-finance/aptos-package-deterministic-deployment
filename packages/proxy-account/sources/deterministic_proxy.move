// Copyright 2025 Yeap Labs. All rights reserved.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// # Deterministic Proxy Module
///
/// This module provides functionality for creating and managing deterministic proxy accounts
/// on the Aptos blockchain. Proxy accounts are resource accounts with predictable addresses
/// that can be controlled by designated administrators.
///
/// ## Key Features
/// - **Deterministic Address Generation**: Create proxy accounts with predictable addresses
/// - **Admin-Controlled Access**: Only designated admins can generate signers for proxy accounts
/// - **Secure Resource Management**: Stores signer capabilities securely within proxy accounts
/// - **Integration with Manageable**: Leverages the manageable module for two-step admin transfers
///
/// ## Use Cases
/// This module is ideal for scenarios requiring:
/// - Protocol-owned accounts with predictable addresses
/// - Cross-chain address synchronization
/// - Delegated signing capabilities
/// - Secure multi-step operations
///
/// ## Security Considerations
/// - Only the current admin can generate proxy signers
/// - Signer capabilities are stored securely within the proxy account
/// - Admin role transfers follow a secure two-step process
///

module proxy::deterministic_proxy {

    use std::signer::address_of;

    use aptos_framework::account;
    use aptos_framework::account::{SignerCapability};
    use aptos_extensions::manageable;

    // ===== Constants =====

    /// Seed prefix used for deterministic proxy account creation.
    /// This ensures all proxy accounts created by this module have a consistent
    /// and collision-resistant address generation mechanism.
    const PROXY_SEED: vector<u8> = b"proxy:deterministic_proxy";

    // ===== Structs =====

    /// Capability holder for proxy account signer capabilities.
    ///
    /// This struct stores the `SignerCapability` that allows the generation
    /// of signers for the proxy account. It's stored at the proxy account's
    /// address and can only be accessed by the admin.
    ///
    /// # Fields
    /// * `cap` - The signer capability for the proxy account
    struct ProxyCap has key {
        cap: SignerCapability
    }

    // ===== Public Functions =====

    /// Creates a new deterministic proxy account.
    ///
    /// This function creates a resource account using a deterministic seed, stores
    /// the signer capability in a `ProxyCap` resource, and initializes admin management
    /// for the proxy account.
    ///
    /// # Parameters
    /// * `creator` - The signer creating the proxy account (becomes the initial admin)
    /// * `proxy_name` - A unique name/identifier for the proxy account
    ///
    /// # Returns
    /// * `address` - The address of the newly created proxy account
    ///
    /// # Examples
    /// ```move
    /// let proxy_addr = deterministic_proxy::create(&creator, b"my_proxy");
    /// ```
    ///
    /// # Aborts
    /// * If a proxy with the same creator and name already exists
    /// * If the creator doesn't have sufficient resources for account creation
    public fun create(creator: &signer, proxy_name: vector<u8>): address {
        let (proxy, proxy_signer_cap) = account::create_resource_account(creator, proxy_seed(proxy_name));
        move_to<ProxyCap>(&proxy, ProxyCap { cap: proxy_signer_cap });
        manageable::new(&proxy, address_of(creator));
        address_of(&proxy)
    }

    #[view]
    /// Calculates the deterministic address for a proxy account without creating it.
    ///
    /// This function allows you to compute what the address of a proxy account would be
    /// for a given creator and proxy name, without actually creating the account.
    /// Useful for verification and pre-computation scenarios.
    ///
    /// # Parameters
    /// * `creator` - The address that would create the proxy account
    /// * `proxy_name` - The name/identifier for the proxy account
    ///
    /// # Returns
    /// * `address` - The deterministic address that would be assigned to the proxy
    ///
    /// # Examples
    /// ```move
    /// let expected_addr = deterministic_proxy::proxy_address(@0x123, b"my_proxy");
    /// let actual_addr = deterministic_proxy::create(&signer_for_0x123, b"my_proxy");
    /// assert!(expected_addr == actual_addr, 0);
    /// ```
    public fun proxy_address(creator: address, proxy_name: vector<u8>): address {
        account::create_resource_address(&creator, proxy_seed(proxy_name))
    }

    /// Generates a signer for the proxy account (admin access only).
    ///
    /// This function allows the current admin of a proxy account to generate a signer
    /// that can perform operations on behalf of the proxy account. This is the core
    /// functionality that enables proxy account control.
    ///
    /// # Parameters
    /// * `admin` - The current admin signer for the proxy account
    /// * `proxy` - The address of the proxy account
    ///
    /// # Returns
    /// * `signer` - A signer capability for the proxy account
    ///
    /// # Examples
    /// ```move
    /// let proxy_signer = deterministic_proxy::generate_proxy_signer(&admin, proxy_addr);
    /// // Use proxy_signer to perform operations on behalf of the proxy
    /// ```
    ///
    /// # Aborts
    /// * `ENOT_ADMIN` - If the caller is not the current admin of the proxy account
    /// * If the `ProxyCap` resource doesn't exist at the proxy address
    public fun generate_proxy_signer(admin: &signer, proxy: address): signer acquires ProxyCap {
        manageable::assert_is_admin(admin, proxy);
        let cap = &ProxyCap[proxy];
        account::create_signer_with_capability(&cap.cap)
    }

    // ===== Helper Functions =====

    /// Constructs the seed for deterministic proxy account creation.
    ///
    /// This internal function combines the module's base seed with the provided
    /// custom seed to create a unique identifier for proxy account generation.
    ///
    /// # Parameters
    /// * `seed` - The custom seed (typically the proxy name)
    ///
    /// # Returns
    /// * `vector<u8>` - The combined seed for account creation
    inline fun proxy_seed(seed: vector<u8>): vector<u8> {
        let seeds = PROXY_SEED;
        seeds.append(seed);
        seeds
    }
}

