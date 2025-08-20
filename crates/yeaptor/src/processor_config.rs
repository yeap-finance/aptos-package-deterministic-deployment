use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_yaml::Value as YamlValue;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessorConfig {
    pub spec_identifier: SpecIdentifier,
    pub common_config: CommonConfig,
    pub custom_config: CustomConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpecIdentifier {
    pub spec_creator: String,
    pub spec_name: String,
    pub spec_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommonConfig {
    pub network: String,
    pub starting_version: u64,
    pub starting_version_override: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomConfig {
    #[serde(default)]
    pub db_schema: BTreeMap<String, TableSchema>,
    #[serde(default)]
    pub events: BTreeMap<String, EventMapping>,
    #[serde(default)]
    pub transaction_metadata: BTreeMap<String, Vec<ColumnTarget>>,
    #[serde(default)]
    pub payload: BTreeMap<String, YamlValue>,
    #[serde(default)]
    pub event_metadata: BTreeMap<String, Vec<ColumnTarget>>,
}

// A table schema is a mapping from column name to its specification.
pub type TableSchema = BTreeMap<String, ColumnSpec>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColumnSpec {
    pub column_type: ColumnTypeSpec,
    #[serde(default)]
    pub default_value: Option<YamlValue>,
    pub is_index: bool,
    pub is_nullable: bool,
    pub is_option: bool,
    pub is_primary_key: bool,
    pub is_vec: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColumnTypeSpec {
    pub column_type: String,
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventMapping {
    #[serde(default)]
    pub constant_values: Vec<YamlValue>,
    #[serde(default)]
    pub event_fields: BTreeMap<String, Vec<ColumnTarget>>,
    #[serde(default)]
    pub event_metadata: BTreeMap<String, Vec<ColumnTarget>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColumnTarget {
    pub column: String,
    pub table: String,
}

// Helpers for YAML I/O
pub fn load_processor_config_yaml(path: &Path) -> Result<ProcessorConfig> {
    let s = fs::read_to_string(path)
        .with_context(|| format!("failed to read YAML config: {}", path.display()))?;
    let cfg: ProcessorConfig = serde_yaml::from_str(&s)
        .with_context(|| format!("failed to parse YAML config: {}", path.display()))?;
    Ok(cfg)
}

pub fn save_processor_config_yaml(path: &Path, cfg: &ProcessorConfig) -> Result<()> {
    let serialized = serde_yaml::to_string(cfg).context("failed to serialize YAML config")?;
    fs::write(path, serialized)
        .with_context(|| format!("failed to write YAML config: {}", path.display()))?;
    Ok(())
}
