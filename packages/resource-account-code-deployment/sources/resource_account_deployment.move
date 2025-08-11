// Copyright (c) 2025 yeap-finance
// SPDX-License-Identifier: Apache-2.0

/// Module providing deterministic deployment of Move packages to resource accounts.
///
/// This module exposes helper functions to:
/// - Create a deterministic resource account for a publisher and seed (create_resource_account)
/// - Publish or upgrade a package under that resource account (publish)
/// - Idempotently ensure the account exists and publish in one call (deploy)
module ra_code_deployment::ra_code_deployment {
    use std::error;
    use std::signer::address_of;
    use aptos_framework::account;
    use aptos_framework::account::{SignerCapability, create_resource_address};
    use aptos_framework::code;
    use aptos_extensions::manageable;

    const LENGTH_MISMATCH: u64 = 1;

    /// Capability to create a signer for the resource account in order to upgrade code.
    struct PublishPackageCap has key {
        cap: SignerCapability
    }

    /// Create the resource account for the given `publisher` and `seed`.
    ///
    /// - Creates the resource account derived from `publisher` and `seed`.
    /// - Stores a `PublishPackageCap` under the resource account for future publishes/upgrades.
    /// - Initializes a manageable admin resource with `publisher` as admin.
    ///
    /// Note: This function does not check for prior existence and will abort if the account
    /// already exists.
    public entry fun create_resource_account(publisher: &signer, seed: vector<u8>) {
        let (resource, resource_signer_cap) = account::create_resource_account(publisher, seed);
        move_to(&resource, PublishPackageCap { cap: resource_signer_cap });
        manageable::new(&resource, address_of(publisher));
    }

    /// Freeze a resource account by revoking management and removing the publish capability.
    ///
    /// - Requires `admin` to be an admin of the manageable resource at `resource_address`.
    /// - Moves out the `PublishPackageCap` from the resource account, preventing future publishes/upgrades.
    /// - Generates a signer from that capability and calls `manageable::destroy` to remove the
    ///   manageable admin resource from the resource account.
    ///
    /// Effects:
    /// - After execution, the resource account is no longer manageable via this module and cannot
    ///   publish or upgrade packages using the removed capability.
    public entry fun freeze_resource_account(admin: &signer, resource_address: address) acquires PublishPackageCap {
        manageable::assert_is_admin(admin, resource_address);
        let PublishPackageCap {cap} = move_from<PublishPackageCap>(resource_address);
        let resource_signer = account::create_signer_with_capability(&cap);
        manageable::destroy(&resource_signer);
    }

    /// Deploy a package to a deterministic resource account derived from `publisher` and `seed`.
    ///
    /// Ensures the resource account exists (via `create_resource_account`) and then publishes the
    /// package by calling `publish`.
    public entry fun deploy(publisher: &signer, seed: vector<u8>, metadata_serialized: vector<u8>, code: vector<vector<u8>>) acquires PublishPackageCap {
        let resource_address = create_resource_address(&address_of(publisher), seed);
        if (!exists<PublishPackageCap>(resource_address)) {
            create_resource_account(publisher, seed);
        };
        publish(publisher, metadata_serialized, code, resource_address);
    }

    /// Batch deploy multiple packages to the deterministic resource account derived from
    /// `publisher` and `seed`.
    ///
    /// Ensures the resource account exists (via `create_resource_account`) and then publishes all
    /// packages in order by delegating to `batch_publish`.
    public entry fun batch_deploy(
        publisher: &signer,
        seed: vector<u8>,
        metadatas: vector<vector<u8>>,
        packages: vector<vector<vector<u8>>>
    ) acquires PublishPackageCap {
        let resource_address = create_resource_address(&address_of(publisher), seed);
        if (!account::exists_at(resource_address)) {
            create_resource_account(publisher, seed);
        };
        batch_publish(publisher, resource_address, metadatas, packages);
    }

    /// Publish a package to `resource_address`.
    ///
    /// - Requires `admin` to be an admin of the manageable resource at `resource_address`.
    /// - Uses the stored `PublishPackageCap` to create a signer for the resource account and publish
    ///   the package with `metadata_serialized` and `code`. Calling this again upgrades the package.
    public entry fun publish(admin: &signer, metadata_serialized: vector<u8>, code: vector<vector<u8>>, resource_address: address) acquires PublishPackageCap {
        manageable::assert_is_admin(admin, resource_address);
        let deploy_cap = borrow_global<PublishPackageCap>(resource_address);
        let resource_signer = account::create_signer_with_capability(&deploy_cap.cap);
        code::publish_package_txn(&resource_signer, metadata_serialized, code);
    }

    /// Batch publish multiple packages to `resource_address` in the given order.
    ///
    /// - Requires `admin` to be an admin of the manageable resource at `resource_address`.
    /// - Publishes each package sequentially using the stored `PublishPackageCap` signer.
    /// - The order in the arrays is preserved; lengths must match.
    public entry fun batch_publish(
        admin: &signer,
        resource_address: address,
        metadatas: vector<vector<u8>>,
        packages: vector<vector<vector<u8>>>
    ) acquires PublishPackageCap {
        manageable::assert_is_admin(admin, resource_address);
        let deploy_cap = borrow_global<PublishPackageCap>(resource_address);
        let resource_signer = account::create_signer_with_capability(&deploy_cap.cap);

        let len_m = metadatas.length();
        let len_p = packages.length();
        assert!(len_m == len_p, error::invalid_argument(LENGTH_MISMATCH));

        // Reverse inputs so pop_back preserves original order
        metadatas.reverse();
        packages.reverse();

        while (!metadatas.is_empty()) {
            let m = metadatas.pop_back();
            let p = packages.pop_back();
            code::publish_package_txn(&resource_signer, m, p);
        };
    }
}
