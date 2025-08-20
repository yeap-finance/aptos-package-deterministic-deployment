use crate::config::YeaptorConfig;
use anyhow::anyhow;

use aptos::common::types::{CliError, CliTypedResult, MovePackageOptions};
use aptos::move_tool::{IncludedArtifacts, IncludedArtifactsArgs};
use aptos_framework::BuiltPackage;
use aptos_types::account_address::{AccountAddress, create_resource_address};
use std::collections::BTreeMap;

use std::path::Path;

#[derive(Debug, Clone)]
pub struct YeaptorEnv {
    config: YeaptorConfig,
    named_addresses: BTreeMap<String, AccountAddress>,
}
pub struct BuiltDeployment {
    #[allow(unused)]
    pub publisher: AccountAddress,
    pub seed: String,

    pub pack: BuiltPackage, // (package_name, metadata_serialized, modules)
}

impl YeaptorEnv {
    pub fn new(config: YeaptorConfig) -> Self {
        let mut named_addresses: BTreeMap<_, _> = config.named_addresses.clone();
        let package_addresses = config
            .deployments
            .iter()
            .flat_map(|de| {
                let deployment_address = create_resource_address(
                    config
                        .publishers
                        .get(de.publisher.as_str())
                        .unwrap()
                        .clone(),
                    de.seed.as_bytes(),
                );
                de.packages
                    .iter()
                    .map(move |package| (package.address_name.clone(), deployment_address.clone()))
            })
            .collect::<BTreeMap<String, AccountAddress>>();
        named_addresses.extend(package_addresses);

        Self {
            config,
            named_addresses,
        }
    }
    pub fn config(&self) -> &YeaptorConfig {
        &self.config
    }

    pub fn deploy_order(&self, package_path: &Path) -> CliTypedResult<Option<u64>> {
        let package_path = package_path.canonicalize().map_err(|e| {
            CliError::IO(
                format!(
                    "Failed to canonicalize package path {}",
                    package_path.display()
                ),
                e,
            )
        })?;
        let mut i = 0;
        for d in &self.config.deployments {
            for p in &d.packages {
                let path = p.path.canonicalize().map_err(|e| {
                    CliError::IO(
                        format!("Failed to canonicalize package path {}", p.path.display()),
                        e,
                    )
                })?;
                if path == package_path {
                    return Ok(Some(i));
                }
                i += 1;
            }
        }
        Ok(None)
    }
    #[allow(unused)]
    pub fn named_addresses(&self) -> &BTreeMap<String, AccountAddress> {
        &self.named_addresses
    }

    pub fn build_all(
        &self,
        included_args: &IncludedArtifactsArgs,
        move_options: &MovePackageOptions,
    ) -> CliTypedResult<Vec<BuiltDeployment>> {
        let mut deployments = Vec::new();
        for deployment in &self.config.deployments {
            let publisher = self
                .config
                .publishers
                .get(&deployment.publisher)
                .expect(&format!(
                    "Publisher address not found: {}",
                    deployment.publisher
                ))
                .clone();
            let seed = deployment.seed.clone();
            for pkg in &deployment.packages {
                let pkg_path = Path::new(&pkg.path);
                let included_artifacts = pkg
                    .include_artifacts
                    .as_ref()
                    .unwrap_or(&included_args.included_artifacts);
                let pack = self
                    .build_package(pkg_path, included_artifacts, move_options)
                    .expect("Failed to build package");

                let d = BuiltDeployment {
                    publisher: publisher.clone(),
                    seed: seed.clone(),
                    pack,
                };
                deployments.push(d);
            }
        }
        Ok(deployments)
    }

    pub fn build_package(
        &self,
        package_dir: &Path,
        included_args: &IncludedArtifacts,
        move_options: &MovePackageOptions,
    ) -> CliTypedResult<BuiltPackage> {
        let mut build_options = included_args.build_options(move_options)?;
        build_options.install_dir = move_options.output_dir.clone();
        let mut named_addresses = self.named_addresses.clone();
        named_addresses.extend(build_options.named_addresses.clone());
        build_options.named_addresses = named_addresses;
        let pack = BuiltPackage::build(package_dir.to_path_buf(), build_options)
            .map_err(|e| anyhow!("Move compilation error: {:#}", e))?;
        Ok(pack)
    }

    pub fn build_deployment_package(
        &self,
        package_dir: &Path,
        included_args: &IncludedArtifactsArgs,
        move_options: &MovePackageOptions,
    ) -> CliTypedResult<(usize, BuiltDeployment)> {
        // Canonicalize the input package directory for proper comparison
        let canonical_package_dir = package_dir.canonicalize().map_err(|e| {
            CliError::IO(
                format!(
                    "Failed to canonicalize package directory {}",
                    package_dir.display(),
                ),
                e,
            )
        })?;
        let mut i = 0;
        for deployment in &self.config.deployments {
            for pkg in &deployment.packages {
                // Canonicalize the config package path for comparison
                let canonical_pkg_path = Path::new(&pkg.path).canonicalize().map_err(|e| {
                    CliError::IO(
                        format!(
                            "Failed to canonicalize package directory {}",
                            package_dir.display(),
                        ),
                        e,
                    )
                })?;
                if canonical_pkg_path == canonical_package_dir {
                    let built_package = self.build_package(
                        canonical_pkg_path.as_path(),
                        &included_args.included_artifacts,
                        move_options,
                    )?;
                    let deployment = BuiltDeployment {
                        publisher: self
                            .config
                            .publishers
                            .get(&deployment.publisher)
                            .expect(&format!(
                                "Publisher address not found: {}",
                                deployment.publisher
                            ))
                            .clone(),
                        seed: deployment.seed.clone(),
                        pack: built_package,
                    };
                    return Ok((i, deployment));
                };
                i += 1;
            }
        }

        Err(CliError::UnexpectedError(format!(
            "No deployment found for package directory: {}",
            package_dir.display()
        )))
    }
}
