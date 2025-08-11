use anyhow::{Context, Result, anyhow};
use aptos::common::transactions::source_package::layout::SourcePackageLayout;
use aptos::common::transactions::source_package::manifest_parser;
use aptos::common::transactions::source_package::parsed_manifest::{NamedAddress, SourceManifest};
use clap::{Parser, Subcommand};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use aptos::common::types::{ArgWithTypeJSON, CliCommand, CliError, CliResult, CliTypedResult, EntryFunctionArgumentsJSON, MovePackageOptions, PromptOptions, SaveFile};
use aptos::move_tool::IncludedArtifactsArgs;
use aptos_framework::{BuildOptions, BuiltPackage, extended_checks};
use aptos_types::account_address::{AccountAddress, create_resource_address};
use clap::builder::Str;
use crate::config::{YeaptorConfig, load_config, Deployment};

#[derive(Subcommand)]
pub enum DeploymentTool {
    BuildPublishPayload(BuildPublishPayload),
}
impl DeploymentTool {
    pub async fn execute(self) -> CliResult {
        match self {
            DeploymentTool::BuildPublishPayload(tool) => tool.execute_serialized().await,
        }
    }
}

#[derive(Parser)]
pub struct BuildPublishPayload {
    #[clap(flatten)]
    pub(crate) included_artifacts_args: IncludedArtifactsArgs,
    #[clap(flatten)]
    pub(crate) move_options: MovePackageOptions,
    #[clap(flatten)]
    pub(crate) prompt_options: PromptOptions,
    /// Path to yeaptor config (TOML)
    #[clap(long, default_value = "./yeaptor.toml", value_parser)]
    pub(crate) config: PathBuf,

    /// Directory to write JSON payloads into (one file per package)
    #[clap(long, value_parser, default_value = "./deployments")]
    pub(crate) out_dir: PathBuf,
}

#[derive(Debug, Clone)]
struct YeaptorEnv {
    config: YeaptorConfig,
    named_addresses: BTreeMap<String, AccountAddress>,
    package_addresses: BTreeMap<String, AccountAddress>,
    package_manifests: BTreeMap<String, SourceManifest>,
}

#[derive(Debug, Clone)]
struct BuiltDeployment {
    publisher: AccountAddress,
    seed: String,
    packages: Vec<(String, Vec<u8>, Vec<Vec<u8>>)>, // (package_name, metadata_serialized, modules)
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
                        .unwrap().clone(),
                    de.seed.as_bytes(),
                );
                de.packages
                    .iter()
                    .map(move |package| (package.address_name.clone(), deployment_address.clone()))
            })
            .collect::<BTreeMap<String, AccountAddress>>();
        named_addresses.append(&mut (package_addresses.clone()));

        let package_manifests = config
            .deployments
            .iter()
            .flat_map(|de| {
                de.packages.iter().map(|pkg| {
                    let pkg_path = Path::new(&pkg.path);
                    let manifest = read_package_manifest(&pkg_path)
                        .expect("Failed to parse package manifest");
                    (pkg.address_name.clone(), manifest)
                })
            })
            .collect::<BTreeMap<String, SourceManifest>>();

        Self {
            config,
            named_addresses,
            package_addresses,
            package_manifests,
        }
    }
    pub fn config(&self) -> &YeaptorConfig {
        &self.config
    }
    pub fn named_addresses(&self) -> &BTreeMap<String, AccountAddress> {
        &self.named_addresses
    }
    pub fn build_all(&self, included_args: &IncludedArtifactsArgs,
                     move_options: &MovePackageOptions,)->CliTypedResult<Vec<BuiltDeployment>> {
        let deployments = self.config.deployments.iter()
            .map(|deployment| {
                self.build_deployment(deployment, included_args, move_options)
            })
            .collect::<CliTypedResult<Vec<_>>>()?;
        Ok(deployments)
    }
    pub fn build_deployment(&self, deployment: &Deployment,
        included_args: &IncludedArtifactsArgs,
        move_options: &MovePackageOptions,
    )->CliTypedResult<BuiltDeployment> {
        let packages = deployment
            .packages
            .iter()
            .map(|pkg| {
                let pkg_path = Path::new(&pkg.path);
                let (pkg_name, metadata_serialized, modules) = self.build_package(
                    pkg_path,
                    included_args,
                    move_options,
                )
                .expect("Failed to build package");

                let package_address = self.package_addresses.get(&pkg.address_name)
                    .expect("Package address not found");

                (
                    pkg_name,
                    metadata_serialized,
                    modules,
                )
            }).collect::<Vec<_>>();
        Ok(BuiltDeployment {
            publisher: self.config.publishers.get(&deployment.publisher)
                .expect("Publisher address not found")
                .clone(),
            seed: deployment.seed.clone(),
            packages,
        })
    }

    fn build_package(
        &self,
        package_dir: &Path,
        included_args: &IncludedArtifactsArgs,
        move_options: &MovePackageOptions,
    ) -> CliTypedResult<(String, Vec<u8>, Vec<Vec<u8>>)> {
        let mut build_options = included_args
            .included_artifacts
            .build_options(move_options)?;
        build_options.install_dir = move_options.output_dir.clone();
        let mut named_addresses = self.named_addresses.clone();
        named_addresses.append(&mut (build_options.named_addresses.clone()));
        build_options.named_addresses = named_addresses;
        let pack = BuiltPackage::build(package_dir.to_path_buf(), build_options)
            .map_err(|e| anyhow!("Move compilation error: {:#}", e))?;

        let metadata = pack.extract_metadata()?;
        let metadata_serialized =
            bcs::to_bytes(&metadata).expect("PackageMetadata should be serializable to BCS");
        let modules = pack.extract_code();

        let pkg_name = read_package_name(package_dir).unwrap_or_else(|_| {
            package_dir
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        });

        Ok((pkg_name, metadata_serialized, modules))
    }
}

#[async_trait::async_trait]
impl CliCommand<String> for BuildPublishPayload {
    fn command_name(&self) -> &'static str {
        "BuildPublishPayload"
    }
    async fn execute(self) -> CliTypedResult<String> {
        let cfg = load_config(&self.config)
            .with_context(|| format!("failed to load config at {}", self.config.display()))?;

        fs::create_dir_all(&self.out_dir)
            .with_context(|| format!("failed to create output dir {}", self.out_dir.display()))?;

        let mut written = 0usize;
        let env = YeaptorEnv::new(cfg);
        let built_deployments = env.build_all(&self.included_artifacts_args, &self.move_options).with_context(|| "failed to build all deployments")?;
        for dep in built_deployments {
            for (pkg_name, metadata_serialized, modules) in &dep.packages {
                let json = make_publish_payload_json(env.config.yeaptor_address, dep.seed.as_str(), &metadata_serialized, &modules);
                let out_path = self.out_dir.join(format!("{}-{}-publish.json",written, pkg_name));
                let save_file = SaveFile {
                    output_file: out_path,
                    prompt_options: self.prompt_options.clone(),
                };
                save_file.check_file()?;
                save_file.save_to_file(
                    "Publication entry function JSON file",
                    serde_json::to_string_pretty(&json)
                        .map_err(|err| CliError::UnexpectedError(format!("{}", err)))?
                        .as_bytes(),
                )?;
                written += 1;
            }
        }

        Ok(format!(
            "Wrote {} publish payload JSON files to {}",
            written,
            self.out_dir.display()
        ))
    }
}

fn read_package_manifest(
    package_dir: &Path,
) -> Result<SourceManifest> {
    Ok(manifest_parser::parse_move_manifest_from_file(package_dir)
        .with_context(|| format!("failed to parse package manifest at {}", package_dir.display()))?)
}

#[inline]
fn read_package_name(package_dir: &Path) -> Result<String> {
    Ok(read_package_manifest(package_dir)?.package.name.as_str().to_string())
}

fn make_publish_payload_json(ra_code_deployment_address: AccountAddress, seed: &str, metadata: &[u8], modules: &[Vec<u8>]) -> serde_json::Value {
    let seed_hex = format!("0x{}", hex::encode(seed.as_bytes()));
    let meta_hex = format!("0x{}", hex::encode(metadata));
    let module_hex: Vec<String> = modules
        .iter()
        .map(|m| format!("0x{}", hex::encode(m)))
        .collect();
    json!({
        "function_id": format!("{}::{}::{}", ra_code_deployment_address.to_standard_string(), "ra_code_deployment", "deploy"),
        "type_args": [],
        "args": [
            { "type": "hex", "value": seed_hex },
            { "type": "hex", "value": meta_hex },
            { "type": "hex", "value": module_hex },
        ]
    })
}
