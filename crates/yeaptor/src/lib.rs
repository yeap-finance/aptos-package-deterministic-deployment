pub mod config;
pub mod deployment;
pub mod version;

use clap::Parser;

pub type CliResult = Result<String, String>;

#[derive(Parser)]
#[clap(name = "yeaptor", author, version, propagate_version = true, styles = aptos_cli_common::aptos_cli_style())]
pub enum YeaptorTool {
    #[clap(subcommand)]
    Deployment(deployment::DeploymentTool),
    /// Print build and git version information
    Version(version::VersionTool),
}

impl YeaptorTool {
    pub async fn execute(self) -> CliResult {
        match self {
            YeaptorTool::Deployment(tool) => tool.execute().await,
            YeaptorTool::Version(tool) => tool.execute().await,
        }
    }
}
