// Copyright 2025 Yeap Labs
// SPDX-License-Identifier: Apache-2.0

/// Package Deployer
///
/// This module exposes a simple on-chain interface to publish and upgrade Move packages
/// to deterministic object addresses, backed by the
/// `object_code_deterministic_deployment::deployment` module.
///
/// Deployment model
/// - This module itself should be published using `aptos move create-resource-account-and-publish-package`.
/// - The resource account (the module address) acts as the deployer authority.
/// - Public entry functions expect to be called by the deployer (the resource account signer).
///
/// Functions
/// - init_module: called on first publish. Creates the resource account when using the CLI flow.
/// - deterministic_publish: publishes a package at a deterministic object address.
/// - upgrade: upgrades an existing package at a given object address.
/// - freeze: freezes a code object to make it immutable.
module package_deployer::deployer {
    use aptos_framework::account;
    use aptos_framework::account::SignerCapability;
    use aptos_framework::object::{Object};
    use aptos_framework::code::PackageRegistry;
    use aptos_framework::resource_account;
    use aptos_framework::transaction_context;
    use aptos_extensions::manageable;
    use object_code_deterministic_deployment::deployment as ocd;

    struct DeployCap has key {
        /// The address of the deployer resource account.
        cap: SignerCapability
    }

    /// Initializes the module during first publish.
    /// When using `aptos move create-resource-account-and-publish-package`, the framework
    /// invokes the designated init function to allow any bootstrapping logic. This function
    /// is a no-op placeholder to make that flow clear and future-proof.
    fun init_module(resource_account: &signer) {
        let sender= transaction_context::sender();
        let resource_account_signer_cap = resource_account::retrieve_resource_account_cap(resource_account, sender);
        manageable::new(resource_account, sender);
    }

    /// Ungovern the deployer module, removing its admin role and destroying the DeployCap resource.
    public entry fun ungov(admin: &signer) acquires DeployCap {
        let deployer = deployer_signer(admin);
        manageable::destroy(&deployer);
        let DeployCap {cap: _} = move_from<DeployCap>(@package_deployer);
    }

    /// Publish a package deterministically to an object address derived from the deployer and a seed.
    /// metadata_serialized: package-metadata.bcs bytes.
    /// code: vector of compiled module bytecode blobs.
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
    public entry fun freeze_code_object(
        admin: &signer,
        code_object: Object<PackageRegistry>,
    ) acquires DeployCap {
        let deployer = deployer_signer(admin);
        ocd::freeze_code_object(&deployer, code_object);
    }

    /// View: compute the deterministic object address for a given seed.
    #[view]
    public fun code_object_address(seed: vector<u8>): address {
        ocd::create_code_object_address(@package_deployer, seed)
    }

    inline fun deployer_signer(admin: &signer): signer {
        manageable::assert_is_admin(admin, @package_deployer);
        let deployer = account::create_signer_with_capability(&borrow_global<DeployCap>(@package_deployer).cap);
        deployer
    }

}
