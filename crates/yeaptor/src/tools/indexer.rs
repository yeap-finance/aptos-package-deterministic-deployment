use crate::db_schema::load_db_schema_from_csv;
use crate::event_table_mapping::load_event_table_mappings_from_csv;
use crate::processor_config::save_processor_config_yaml;
use crate::processor_config_generator::{
    generate_processor_config, load_event_definitions_from_dir,
};
use aptos::common::init::Network;
use aptos::common::types::{CliCommand, CliError, CliTypedResult};
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum IndexerTool {
    Generate(Generate),
}

impl IndexerTool {
    pub async fn execute(self) -> crate::CliResult {
        match self {
            IndexerTool::Generate(tool) => tool.execute_serialized().await,
        }
    }
}

#[derive(clap::Parser)]
pub struct Generate {
    #[clap(short, long, value_parser, default_value = "testnet")]
    pub(crate) network: Network,
    #[clap(short, long, value_parser)]
    pub(crate) starting_version: u64,

    /// Path to yeaptor config (TOML)
    #[clap(long, default_value = "./events", value_parser)]
    pub(crate) events_dir: PathBuf,
    #[clap(long, value_parser, default_value = "./db_schema.csv")]
    pub(crate) db_schema: PathBuf,
    #[clap(long, value_parser, default_value = "./event_mapping.csv")]
    pub(crate) event_mapping: PathBuf,
    #[clap(long, value_parser, default_value = "./processor_config.yaml")]
    pub(crate) output_file: PathBuf,
}
#[async_trait::async_trait]
impl CliCommand<String> for Generate {
    fn command_name(&self) -> &'static str {
        "definition"
    }
    async fn execute(self) -> CliTypedResult<String> {
        let db_schema = load_db_schema_from_csv(self.db_schema.as_path()).map_err(|e| {
            CliError::UnableToReadFile(self.db_schema.display().to_string(), e.to_string())
        })?;
        let event_definitions = load_event_definitions_from_dir(self.events_dir.as_path())
            .map_err(|e| {
                CliError::UnableToReadFile(self.events_dir.display().to_string(), e.to_string())
            })?;
        let event_mapping = load_event_table_mappings_from_csv(self.event_mapping.as_path())
            .map_err(|e| {
                CliError::UnableToReadFile(self.event_mapping.display().to_string(), e.to_string())
            })?;

        let (config, unmapped_events, unmapped_table_columns) = generate_processor_config(
            self.network,
            self.starting_version, // Use the provided starting version
            &event_definitions,
            &db_schema,
            &event_mapping,
        )?;
        save_processor_config_yaml(self.output_file.as_path(), &config)?;

        let mut error_message = String::new();
        if !unmapped_events.is_empty() {
            error_message.push_str("Unmapped events:\n");
            for event in unmapped_events {
                error_message.push_str(&format!("  - {}\n", event));
            }
        }
        if !unmapped_table_columns.is_empty() {
            error_message.push_str("Unmapped table columns:\n");
            for (table, column) in unmapped_table_columns {
                error_message.push_str(&format!("  - {},{}\n", table, column));
            }
        }
        // If there are unmapped events or columns, return them as part of the error
        if !error_message.is_empty() {
            error_message = format!(
                "Processor config generated with warnings:\n{}",
                error_message
            );
            println!("{}", error_message);
        }

        Ok(format!(
            "Processor config generated successfully at {}",
            self.output_file.display()
        ))
    }
}
