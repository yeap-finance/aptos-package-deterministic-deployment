/// Module providing deterministic deployment of Move packages to resource accounts.
///
/// This module exposes helper functions to:
/// - Initialize a deterministic resource account for a publisher and seed (init_resource_account)
/// - Publish or upgrade a package under that resource account (publish)
/// - Idempotently ensure the account exists and publish in one call (deploy)
module ra_code_deployment::ra_code_deployment {
    use std::signer::address_of;
    use aptos_framework::account;
    use aptos_framework::account::{SignerCapability, create_resource_address};
    use aptos_framework::code;
    use aptos_extensions::manageable;

    /// Capability to create a signer for the resource account in order to upgrade code.
    struct PublishPackageCap has key {
        cap: SignerCapability
    }

    /// Initialize the resource account for the given `publisher` and `seed` if it does not exist.
    ///
    /// - Creates the resource account derived from `publisher` and `seed`.
    /// - Stores a `PublishPackageCap` under the resource account for future publishes/upgrades.
    /// - Initializes a manageable admin resource with `publisher` as admin.
    ///
    /// This function is idempotent and will no-op if the resource account already exists.
    public fun init_resource_account(publisher: &signer, seed: vector<u8>) {
        // Create a resource account signer capability for the publisher.
        let resource_address = create_resource_address(&address_of(publisher), seed);
        if (!account::exists_at(resource_address)) {
            let (resource, resource_signer_cap) = account::create_resource_account(publisher, seed);
            move_to(&resource, PublishPackageCap { cap: resource_signer_cap });
            manageable::new(&resource, address_of(publisher));
        }
    }

    /// Deploy a package to a deterministic resource account derived from `publisher` and `seed`.
    ///
    /// Ensures the resource account exists (via `init_resource_account`) and then publishes the
    /// package by calling `publish`.
    public fun deploy(publisher: &signer, seed: vector<u8>, metadata_serialized: vector<u8>, code: vector<vector<u8>>) acquires PublishPackageCap {
        let resource_address = create_resource_address(&address_of(publisher), seed);
        if (!account::exists_at(resource_address)) {
            init_resource_account(publisher, seed);
        };
        publish(publisher, metadata_serialized, code, resource_address);
    }

    /// Publish a package to `resource_address`.
    ///
    /// - Requires `admin` to be an admin of the manageable resource at `resource_address`.
    /// - Uses the stored `PublishPackageCap` to create a signer for the resource account and publish
    ///   the package with `metadata_serialized` and `code`. Calling this again upgrades the package.
    public fun publish(admin: &signer, metadata_serialized: vector<u8>, code: vector<vector<u8>>, resource_address: address) acquires PublishPackageCap {
        manageable::assert_is_admin(admin, resource_address);
        let deploy_cap = borrow_global<PublishPackageCap>(resource_address);
        let resource_signer = account::create_signer_with_capability(&deploy_cap.cap);
        code::publish_package_txn(&resource_signer, metadata_serialized, code);
    }

}
