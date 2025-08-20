use crate::config::load_config;
use crate::env::{BuiltDeployment, YeaptorEnv};
use crate::tools::event::build_event_definition;
use anyhow::{Context, Result};
use aptos::common::transactions::source_package::manifest_parser;
use aptos::common::transactions::source_package::parsed_manifest::SourceManifest;
use aptos::common::types::{
    CliCommand, CliError, CliResult, CliTypedResult, MovePackageOptions, PromptOptions, SaveFile,
};
use aptos::move_tool::IncludedArtifactsArgs;
use aptos_types::account_address::AccountAddress;
use clap::{Parser, Subcommand};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub enum DeploymentTool {
    Build(Build),
}
impl DeploymentTool {
    pub async fn execute(self) -> CliResult {
        match self {
            DeploymentTool::Build(tool) => tool.execute_serialized().await,
        }
    }
}

#[derive(Parser)]
pub struct Build {
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

    /// If true, will include events in the build process
    #[clap(long, default_value = "false")]
    pub(crate) with_event: bool,
}

#[async_trait::async_trait]
impl CliCommand<String> for Build {
    fn command_name(&self) -> &'static str {
        "Build"
    }
    async fn execute(self) -> CliTypedResult<String> {
        let cfg = load_config(&self.config)
            .with_context(|| format!("failed to load config at {}", self.config.display()))?;

        fs::create_dir_all(&self.out_dir)
            .with_context(|| format!("failed to create output dir {}", self.out_dir.display()))?;

        let mut package_written = 0usize;
        let mut event_written = 0usize;
        let env = YeaptorEnv::new(cfg);

        // Check if a specific package directory is specified
        let built_deployments = if let Some(ref package_dir) = self.move_options.package_dir {
            // Build only the specific package
            let built_deployment = env
                .build_deployment_package(
                    package_dir,
                    &self.included_artifacts_args,
                    &self.move_options,
                )
                .with_context(|| format!("failed to build package at {}", package_dir.display()))?;
            vec![built_deployment]
        } else {
            // Build all deployments as before
            env.build_all(&self.included_artifacts_args, &self.move_options)
                .with_context(|| "failed to build all deployments")?
                .into_iter()
                .enumerate()
                .collect::<Vec<_>>()
        };

        fs::create_dir_all(&self.out_dir).with_context(|| {
            format!(
                "failed to create output directory {}",
                self.out_dir.display()
            )
        })?;
        if self.with_event {
            // Ensure the events subdirectory exists
            let events_dir = self.out_dir.join("events");
            fs::create_dir_all(&events_dir).with_context(|| {
                format!("failed to create events directory {}", events_dir.display())
            })?;
        }
        for (i, deployment) in built_deployments {
            let BuiltDeployment {
                publisher: _,
                seed,
                pack,
            } = deployment;

            let (pkg_name, metadata_serialized, modules) = {
                let metadata = pack
                    .extract_metadata()
                    .expect("Package metadata should be present");
                let metadata_serialized = bcs::to_bytes(&metadata)
                    .expect("PackageMetadata should be serializable to BCS");
                let modules = pack.extract_code();
                (pack.name().to_string(), metadata_serialized, modules)
            };
            if self.with_event {
                let all_events = build_event_definition(&pack);
                if !all_events.is_empty() {
                    // Ensure the events subdirectory exists
                    let events_dir = self.out_dir.join("events");
                    // write the events as json to the output directory
                    let save_file = SaveFile {
                        output_file: events_dir.join(format!("{}.event.json", pack.name())),
                        prompt_options: self.prompt_options.clone(),
                    };
                    save_file.check_file()?;
                    save_file.save_to_file(
                        "Event definitions",
                        serde_json::to_string_pretty(&all_events)
                            .map_err(|err| CliError::UnexpectedError(format!("{}", err)))?
                            .as_bytes(),
                    )?;
                    event_written += 1;
                }
            }

            let json = make_publish_payload_json(
                env.config().yeaptor_address,
                seed.as_str(),
                &metadata_serialized,
                &modules,
            );
            let out_path = self
                .out_dir
                .join(format!("{}-{}.package.json", i, pkg_name));
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
            package_written += 1;
        }

        // Write resolved named addresses to a TOML file at the end
        let addresses_path = self.out_dir.join("addresses.toml");
        let mut addresses_toml = String::from("[addresses]\n");
        for (name, addr) in env.named_addresses().iter() {
            addresses_toml.push_str(&format!("{} = \"{}\"\n", name, addr.to_standard_string()));
        }
        fs::write(&addresses_path, addresses_toml).with_context(|| {
            format!(
                "failed to write addresses file {}",
                addresses_path.display()
            )
        })?;

        let mut output = format!(
            "Wrote {} publish payload JSON files to {}",
            package_written,
            self.out_dir.display()
        );
        if event_written > 0 {
            output.push_str(&format!(
                ", Wrote {} event definition files to {}",
                event_written,
                self.out_dir.join("events").display()
            ));
        }
        Ok(output)
    }
}

fn read_package_manifest(package_dir: &Path) -> Result<SourceManifest> {
    Ok(
        manifest_parser::parse_move_manifest_from_file(package_dir).with_context(|| {
            format!(
                "failed to parse package manifest at {}",
                package_dir.display()
            )
        })?,
    )
}

#[inline]
fn read_package_name(package_dir: &Path) -> Result<String> {
    Ok(read_package_manifest(package_dir)?
        .package
        .name
        .as_str()
        .to_string())
}

fn make_publish_payload_json(
    ra_code_deployment_address: AccountAddress,
    seed: &str,
    metadata: &[u8],
    modules: &[Vec<u8>],
) -> serde_json::Value {
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
