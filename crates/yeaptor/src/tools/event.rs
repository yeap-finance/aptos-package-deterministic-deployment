use crate::config::load_config;
use crate::env::YeaptorEnv;
use crate::event_definition::{EventDefinition, extract_event_definitions};
use anyhow::Context;
use aptos::common::types::{
    CliCommand, CliError, CliResult, CliTypedResult, MovePackageOptions, PromptOptions, SaveFile,
};
use aptos::move_tool::IncludedArtifacts;
use aptos_framework::BuiltPackage;
use clap::{Parser, Subcommand};
use move_binary_format::access::ModuleAccess;
use std::fs;
use std::path::PathBuf;

#[derive(Subcommand)]
/// Event utilities
pub enum EventTool {
    /// Generate event definition JSON files from compiled Move packages
    Generate(Generate),
}

impl EventTool {
    pub async fn execute(self) -> CliResult {
        match self {
            EventTool::Generate(tool) => tool.execute_serialized().await,
        }
    }
}

#[derive(Parser)]
/// Generate event definition JSON files for a package (via --package-dir) or all packages in yeaptor.toml
pub struct Generate {
    #[clap(flatten)]
    pub(crate) move_options: MovePackageOptions,
    #[clap(flatten)]
    pub(crate) prompt_options: PromptOptions,
    /// Path to yeaptor config (TOML)
    #[clap(long, default_value = "./yeaptor.toml", value_parser)]
    pub(crate) config: PathBuf,

    /// Directory to write JSON payloads into (one file per package)
    #[clap(long, value_parser, default_value = "./events")]
    pub(crate) out_dir: PathBuf,
}

#[async_trait::async_trait]
impl CliCommand<String> for Generate {
    fn command_name(&self) -> &'static str {
        "generate_event_definitions"
    }
    async fn execute(self) -> CliTypedResult<String> {
        let cfg = load_config(&self.config)
            .with_context(|| format!("failed to load config at {}", self.config.display()))?;

        fs::create_dir_all(&self.out_dir)
            .with_context(|| format!("failed to create output dir {}", self.out_dir.display()))?;

        let env = YeaptorEnv::new(cfg);
        let packages: Vec<PathBuf> = if self.move_options.package_dir.is_none() {
            env.config()
                .deployments
                .iter()
                .flat_map(|d| d.packages.iter().map(|p| p.path.clone()))
                .collect::<Vec<_>>()
        } else {
            vec![self.move_options.package_dir.clone().unwrap()]
        };
        let mut writen = 0;
        for package_dir in &packages {
            let pack =
                env.build_package(&package_dir, &IncludedArtifacts::None, &self.move_options)?;

            let all_events = build_event_definition(&pack);

            // write the events as json to the output directory
            let save_file = SaveFile {
                output_file: self.out_dir.join(format!("{}.event.json", pack.name())),
                prompt_options: self.prompt_options.clone(),
            };
            save_file.check_file()?;
            save_file.save_to_file(
                "Event definitions",
                serde_json::to_string_pretty(&all_events)
                    .map_err(|err| CliError::UnexpectedError(format!("{}", err)))?
                    .as_bytes(),
            )?;
            writen += 1;
        }

        Ok(format!(
            "wrote {} event definition files to {}",
            writen,
            self.out_dir.display()
        ))
    }
}

pub(crate) fn build_event_definition(pack: &BuiltPackage) -> Vec<EventDefinition> {
    let package_name = pack.name().to_string();
    let modules = pack.modules().collect::<Vec<_>>();
    let all_events = modules
        .iter()
        .flat_map(|m| {
            let events = extract_event_definitions(m);
            let module_name = m.name().to_string();
            let package_name = package_name.clone();
            events.into_iter().map(move |(event_name, fields)| {
                let event = EventDefinition {
                    package_name: package_name.clone(),
                    module_address: *m.address(),
                    module_name: module_name.clone(),
                    name: event_name.clone(),
                    fields,
                };
                event
            })
        })
        .collect::<Vec<_>>();
    all_events
}
