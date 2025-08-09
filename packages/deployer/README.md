# Package Deployer

package-deployer is onchain resource account, which should be initialized by project developer publishing the module using the command `aptos move create-resource-account-and-publish-package`.
the resource account will be created in the `init_module` function of the module.

It has public functions to deploy/upgrade packages using the object_code_determinitic_deployment.