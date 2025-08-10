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
/// - publish_package(admin, seed, metadata_serialized, code)
/// - upgrade_package(admin, seed, metadata_serialized, code)  [derives code object from seed]
/// - freeze_package(admin, seed)  [derives code object from seed]
/// - upgrade_code_object(admin, metadata_serialized, code, code_object)
/// - freeze_code_object(admin, code_object)
/// - code_object_address(seed) [view]
/// - retire(admin)  [permanently decommissions the deployer]
///
/// Access control
/// - Only the active admin may call publish/upgrade/freeze/retire functions.
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
    use aptos_framework::multisig_account;
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
    /// Invoked by the framework during the special resource-account publish flow
    /// (not an entry function).
    ///
    /// Effects
    /// - Stores the resource account SignerCapability in `DeployCap` at @package_deployer
    /// - Initializes admin management with the transaction sender as initial admin
    ///
    /// Aborts
    /// - If the framework-specific capability retrieval fails (should not occur in the
    ///   intended publish flow)
    fun init_module(resource_account: &signer) {
        let deployer = @deployer_admin_initial;
        let resource_account_signer_cap = resource_account::retrieve_resource_account_cap(resource_account, deployer);
        move_to<DeployCap>(resource_account, DeployCap { cap: resource_account_signer_cap });
        manageable::new(resource_account, deployer);
    }

    /// Retire the deployer module, removing its admin role and destroying the DeployCap resource.
    ///
    /// Access control
    /// - Admin only
    ///
    /// Effects
    /// - Destroys admin state via `manageable::destroy`
    /// - Destroys the stored signer capability (`DeployCap`)
    ///
    /// Irreversible
    /// - After retiring, no further publish/upgrade/freeze operations are possible
    ///
    /// Emits
    /// - Underlying manageable module may emit AdminRoleDestroyed
    public entry fun retire(admin: &signer) acquires DeployCap {
        let deployer = deployer_signer(admin);
        manageable::destroy(&deployer);
        let DeployCap { cap: _ } = move_from<DeployCap>(@package_deployer);
    }

    /// Publish a package to a deterministic object address derived from the deployer resource
    /// account and a custom seed.
    ///
    /// Parameters
    /// - deterministic_object_seed: custom seed used in deterministic address derivation
    /// - metadata_serialized: contents of package-metadata.bcs for the package to publish
    /// - code: vector of module bytecodes (.mv) for the package
    ///
    /// Access control
    /// - Admin only (validated internally via deployer_signer)
    ///
    /// Emits
    /// - `Publish` event from the underlying deployment module
    ///
    /// Notes
    /// - The resulting object address equals `code_object_address(seed)` for the same seed
    public entry fun publish_package(
        admin: &signer,
        deterministic_object_seed: vector<u8>,
        metadata_serialized: vector<u8>,
        code: vector<vector<u8>>,
    ) acquires DeployCap {
        let deployer = deployer_signer(admin);
        ocd::deterministic_publish(&deployer, deterministic_object_seed, metadata_serialized, code);
    }

    /// Upgrade a package by seed.
    ///
    /// This convenience wrapper derives the code object address from `seed` (using
    /// `code_object_address`) and upgrades the package in that object.
    ///
    /// Parameters
    /// - seed: the deterministic seed originally used during publish
    /// - metadata_serialized: new package-metadata.bcs
    /// - code: new module bytecodes (.mv)
    ///
    /// Access control
    /// - Admin only
    ///
    /// Aborts
    /// - If the target object does not exist or is not owned by the deployer
    /// - If the target package is immutable (frozen) or violates upgrade constraints
    public entry fun upgrade_package(
        admin: &signer,
        seed: vector<u8>,
        metadata_serialized: vector<u8>,
        code: vector<vector<u8>>,
    ) acquires DeployCap {
        let addr = ocd::create_code_object_address(@package_deployer, seed);
        let code_object = aptos_framework::object::address_to_object<PackageRegistry>(addr);
        upgrade_code_object(admin, metadata_serialized, code, code_object);
    }

    /// Freeze a package by seed.
    ///
    /// This convenience wrapper derives the code object address from `seed` (using
    /// `code_object_address`) and freezes the package in that object.
    ///
    /// Parameters
    /// - seed: the deterministic seed originally used during publish
    ///
    /// Access control
    /// - Admin only
    ///
    /// Aborts
    /// - If the target object does not exist or is not owned by the deployer
    ///
    /// Irreversible
    /// - Once frozen, the package cannot be upgraded
    public entry fun freeze_package(
        admin: &signer,
        seed: vector<u8>,
    ) acquires DeployCap {

        let addr = ocd::create_code_object_address(@package_deployer, seed);
        let code_object = aptos_framework::object::address_to_object<PackageRegistry>(addr);
        freeze_code_object(admin, code_object);
    }


    /// Upgrade an existing upgradable package by passing an explicit code object.
    ///
    /// Prefer `upgrade_package` for seed-based flows; use this variant when you already
    /// have an `Object<PackageRegistry>` reference.
    ///
    /// Parameters
    /// - metadata_serialized: new package-metadata.bcs
    /// - code: new module bytecodes (.mv)
    /// - code_object: the object holding the target package registry
    ///
    /// Access control
    /// - Admin only
    ///
    /// Aborts
    /// - If the object is not owned by the deployer or is missing
    /// - If the package is immutable (frozen) or violates upgrade constraints
    public entry fun upgrade_code_object(
        admin: &signer,
        metadata_serialized: vector<u8>,
        code: vector<vector<u8>>,
        code_object: Object<PackageRegistry>,
    ) acquires DeployCap {
        let deployer = deployer_signer(admin);
        ocd::upgrade(&deployer, metadata_serialized, code, code_object);
    }

    /// Freeze an existing package by passing an explicit code object.
    ///
    /// Prefer `freeze_package` for seed-based flows; use this variant when you already
    /// have an `Object<PackageRegistry>` reference.
    ///
    /// Parameters
    /// - code_object: the object holding the target package registry
    ///
    /// Access control
    /// - Admin only
    ///
    /// Irreversible
    /// - Once frozen, the package cannot be upgraded
    public entry fun freeze_code_object(
        admin: &signer,
        code_object: Object<PackageRegistry>,
    ) acquires DeployCap {
        let deployer = deployer_signer(admin);
        ocd::freeze_code_object(&deployer, code_object);
    }

    /// View: compute the deterministic object address for a given seed.
    ///
    /// Returns
    /// - Address of the object derived from (@package_deployer, seed)
    ///
    /// Notes
    /// - Must match the address used when calling `publish_package` with the same seed
    #[view]
    public fun code_object_address(seed: vector<u8>): address {
        ocd::create_code_object_address(@package_deployer, seed)
    }

    /// Internal helper to derive a signer for the deployer resource account.
    ///
    /// Access control
    /// - Verifies the caller is the current admin via `manageable::assert_is_admin`
    ///
    /// Returns
    /// - A signer created from the stored `SignerCapability` in `DeployCap`
    inline fun deployer_signer(admin: &signer): signer acquires DeployCap {
        manageable::assert_is_admin(admin, @package_deployer);
        let deployer = account::create_signer_with_capability(&borrow_global<DeployCap>(@package_deployer).cap);
        deployer
    }

}
