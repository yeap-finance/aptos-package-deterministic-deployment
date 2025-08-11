use std::path::PathBuf;
use aptos::common::types::{ChunkedPublishOption, CliResult, MovePackageOptions, OverrideSizeCheckOption, TransactionOptions};
use aptos::move_tool::{IncludedArtifactsArgs, PublishPackage};
use aptos_types::transaction::TransactionPayload;
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct BuildPackage {

    #[clap(flatten)]
    pub(crate) included_artifacts_args: IncludedArtifactsArgs,
    #[clap(flatten)]
    pub(crate) move_options: MovePackageOptions,

}

    #[derive(Parser)]
    pub struct BuildPublishPayload {
        #[clap(flatten)]
        build_package: BuildPackage,
        /// JSON output file to write publication transaction to
        #[clap(long, value_parser)]
        pub(crate) json_output_file: PathBuf,

    }

    impl BuildPublishPayload {
        pub async fn execute_serialized(self) -> CliResult {
            // Implement the logic to build and publish the payload
            Ok(())
        }
    }

pub(crate) struct PackageCustomPublicationData {
    metadata_serialized: Vec<u8>,
    compiled_units: Vec<Vec<u8>>,
    payload: TransactionPayload,
}

impl TryFrom<&BuildPackage> for PackageCustomPublicationData {
    type Error = anyhow::Error;

    fn try_from(build_package: &BuildPackage) -> Result<Self, Self::Error> {
        let options = build_package.included_artifacts_args
            .included_artifacts
            .build_options(&build_package.move_options)?;
        BuiltPackage::build(move_options.get_package_path()?, options)


    }
}


#[derive(Debug, Subcommand)]
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