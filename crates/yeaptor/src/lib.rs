pub mod config;
pub mod env;
pub mod processor_config;
pub mod processor_config_generator;
pub mod version;

use crate::tools::{deployment, event, indexer};
use clap::Parser;

pub mod db_schema;
pub mod event_definition;
pub mod event_table_mapping;
pub mod tools;
pub type CliResult = Result<String, String>;

#[derive(Parser)]
#[clap(name = "yeaptor", author, version, propagate_version = true, styles = aptos_cli_common::aptos_cli_style())]
pub enum YeaptorTool {
    /// Build publish payloads and optional event files from yeaptor.toml deployments
    #[clap(subcommand)]
    Deployment(deployment::DeploymentTool),
    /// Generate event definition JSON from compiled Move packages
    #[clap(subcommand)]
    Event(event::EventTool),
    /// Run the processor/indexer using the configured schema and mappings
    #[clap(subcommand)]
    Processor(indexer::ProcessorTool),
    /// Print build and git version information
    Version(version::VersionTool),
}

impl YeaptorTool {
    pub async fn execute(self) -> CliResult {
        match self {
            YeaptorTool::Deployment(tool) => tool.execute().await,
            YeaptorTool::Version(tool) => tool.execute().await,
            YeaptorTool::Event(tool) => tool.execute().await,
            YeaptorTool::Processor(tool) => tool.execute().await,
        }
    }
}
