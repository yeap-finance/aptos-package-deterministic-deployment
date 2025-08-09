// Copyright 2025 Yeap Labs
// SPDX-License-Identifier: Apache-2.0

/// Package Deployer
///
/// Admin-gated resource-account deployer for publishing, upgrading, and freezing
/// Move packages to deterministic Aptos Object addresses. This wraps
/// `object_code_deterministic_deployment::deployment` and centralizes deployment authority
/// under a managed resource account.
///
/// Deployment model
/// - Publish this package using `aptos move create-resource-account-and-publish-package`.
/// - The resource account (this module address) acts as the deployer authority.
/// - The framework calls `init_module(resource_account)` after publish. It is NOT an entry;
///   it stores the resource account SignerCapability in `DeployCap` and sets the initial
///   admin (the transaction sender) via `manageable`'s two-step admin model.
/// - Public entry functions must be called by the current admin; a deployer signer is
///   derived internally from the stored capability.
///
/// Functions
/// - init_module(resource_account: &signer)  [framework-invoked initializer, not entry]
/// - publish(admin, seed, metadata_serialized, code)
/// - upgrade(admin, metadata_serialized, code, code_object)
/// - freeze_code_object(admin, code_object)
/// - code_object_address(seed) [view]
/// - retire(admin)  [permanently decommissions the deployer]
///
/// Access control
/// - Only the active admin may call publish/upgrade/freeze_code_object/retire.
/// - Admin changes follow the two-step `manageable` flow (change -> accept).
///
/// Events
/// - This module surfaces the underlying events emitted by
///   `object_code_deterministic_deployment::deployment`: Publish, Upgrade, Freeze.
///
/// Notes
/// - `metadata_serialized` must be the package-metadata.bcs bytes.
/// - `code` must be the vector of compiled `.mv` bytecode blobs.
module package_deployer::deployer {
    use aptos_framework::account;
    use aptos_framework::account::SignerCapability;
    use aptos_framework::object::{Object};
    use aptos_framework::code::PackageRegistry;
    use aptos_framework::resource_account;
    use aptos_framework::transaction_context;
    use aptos_extensions::manageable;
    use object_code_deterministic_deployment::deployment as ocd;

    /// Capability container for the deployer resource account.
    ///
    /// The DeployCap resource is stored at the resource account address (@package_deployer)
    /// during `init_module` and enables deriving a signer for subsequent deployments/upgrades.
    struct DeployCap has key {
        /// The signer capability of the deployer resource account.
        cap: SignerCapability
    }

    /// Module initializer for the resource-account publish flow.
    ///
    /// This is invoked automatically by the Aptos CLI command
    /// `aptos move create-resource-account-and-publish-package` after the package code is
    /// published under the resource account. It is NOT an entry function; it is invoked by
    /// the framework during the special publish flow and accepts the resource account signer.
    ///
    /// Steps:
    /// 1) Retrieve the resource account's SignerCapability from the framework.
    /// 2) Store it in `DeployCap` under the resource account address.
    /// 3) Initialize admin management so that only the chosen admin (the transaction sender)
    ///    can operate the deployer.
    ///
    /// Security: Only the framework calls this during the special publish flow. The `sender`
    /// becomes the initial admin; transfer requires the `manageable` two-step flow.
    fun init_module(resource_account: &signer) {
        let sender = transaction_context::sender();
        let resource_account_signer_cap = resource_account::retrieve_resource_account_cap(resource_account, sender);
        move_to<DeployCap>(resource_account, DeployCap { cap: resource_account_signer_cap });
        manageable::new(resource_account, sender);
    }

    /// Retire the deployer module, removing its admin role and destroying the DeployCap resource.
    ///
    /// Only callable by the current admin. After this, no further deploy/upgrade operations
    /// will be possible since the signer capability is destroyed alongside the admin state.
    public entry fun retire(admin: &signer) acquires DeployCap {
        let deployer = deployer_signer(admin);
        manageable::destroy(&deployer);
        let DeployCap { cap: _ } = move_from<DeployCap>(@package_deployer);
    }

    /// Publish a package deterministically to an object address derived from the deployer and a seed.
    ///
    /// Parameters:
    /// - deterministic_object_seed: domain-separated seed bytes for address derivation
    /// - metadata_serialized: bytes of package-metadata.bcs
    /// - code: vector of compiled .mv module bytecode blobs
    ///
    /// Access control: admin-only (through deployer_signer).
    /// Emits: underlying `Publish` event from the object deployment module.
    public entry fun publish(
        admin: &signer,
        deterministic_object_seed: vector<u8>,
        metadata_serialized: vector<u8>,
        code: vector<vector<u8>>,
    ) acquires DeployCap {
        let deployer = deployer_signer(admin);
        ocd::deterministic_publish(&deployer, deterministic_object_seed, metadata_serialized, code);
    }

    /// Upgrade an existing upgradable package at the provided code_object.
    ///
    /// Parameters:
    /// - metadata_serialized: new package-metadata.bcs
    /// - code: new module bytecode blobs (matching upgrade constraints)
    /// - code_object: the object containing the package registry to be upgraded
    ///
    /// Access control: admin-only. Emits the underlying `Upgrade` event.
    public entry fun upgrade(
        admin: &signer,
        metadata_serialized: vector<u8>,
        code: vector<vector<u8>>,
        code_object: Object<PackageRegistry>,
    ) acquires DeployCap {
        let deployer = deployer_signer(admin);
        ocd::upgrade(&deployer, metadata_serialized, code, code_object);
    }

    /// Freeze an existing package, making it immutable.
    ///
    /// After freezing, the code object can no longer be upgraded.
    /// Access control: admin-only. Emits the underlying `Freeze` event.
    public entry fun freeze_code_object(
        admin: &signer,
        code_object: Object<PackageRegistry>,
    ) acquires DeployCap {
        let deployer = deployer_signer(admin);
        ocd::freeze_code_object(&deployer, code_object);
    }

    /// View: compute the deterministic object address for a given seed.
    ///
    /// This uses the resource account's address (@package_deployer) as the publisher.
    #[view]
    public fun code_object_address(seed: vector<u8>): address {
        ocd::create_code_object_address(@package_deployer, seed)
    }

    /// Internal helper to derive a signer for the deployer resource account.
    ///
    /// Verifies the caller is the admin via `manageable`, then creates a signer
    /// using the stored `SignerCapability` in `DeployCap`.
    inline fun deployer_signer(admin: &signer): signer acquires DeployCap {
        manageable::assert_is_admin(admin, @package_deployer);
        let deployer = account::create_signer_with_capability(&borrow_global<DeployCap>(@package_deployer).cap);
        deployer
    }

}
