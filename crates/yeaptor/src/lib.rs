pub mod deployment;
pub mod config;

use clap::Parser;

pub type CliResult = Result<String, String>;

#[derive(Parser)]
#[clap(name = "yeaptor", author, version, propagate_version = true, styles = aptos_cli_common::aptos_cli_style())]
pub enum YeaptorTool {
    #[clap(subcommand)]
    Deployment(deployment::DeploymentTool),
}

impl YeaptorTool {
    pub async fn execute(self) -> CliResult {
        match self {
            YeaptorTool::Deployment(tool) => tool.execute().await,
        }
    }
}
