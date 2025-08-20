use crate::processor_config::{ColumnSpec, ColumnTypeSpec, CustomConfig, TableSchema};
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_yaml::Value as YamlValue;
use std::collections::BTreeMap;
use std::path::Path;

// ===================== CSV Loader for db_schema =====================
fn parse_bool_cell(s: &str) -> bool {
    matches!(
        s.trim().to_ascii_lowercase().as_str(),
        "true" | "t" | "1" | "yes" | "y"
    )
}

fn parse_default_value_cell(s: Option<&str>, type_spec: &ColumnTypeSpec) -> Option<YamlValue> {
    match s.map(|v| v.trim()) {
        Some(v) if !v.is_empty() => {
            match (type_spec.r#type.as_str(), type_spec.column_type.as_str()) {
                // Numeric types
                ("move_type", "u8" | "u16" | "u32" | "u64") => {
                    v.parse::<u64>().ok().map(YamlValue::from).or_else(|| {
                        // If parsing fails, keep as string
                        Some(YamlValue::String(v.to_string()))
                    })
                }
                // Boolean type
                ("move_type", "bool") => {
                    // need to be string "true" or "false" to be parsed by geomi
                    Some(YamlValue::String(
                        if parse_bool_cell(v) { "true" } else { "false" }.to_string(),
                    ))
                }
                // Address type - keep as string
                ("move_type", "address") => Some(YamlValue::String(v.to_string())),
                ("transaction_metadata", _) => {
                    v.parse::<u64>().ok().map(YamlValue::from).or_else(|| {
                        // If parsing fails, keep as string
                        Some(YamlValue::String(v.to_string()))
                    })
                }
                // Timestamp and version types - treat as numeric if possible
                ("event_metadata", "creation_number" | "sequence_number" | "event_index") => v
                    .parse::<u64>()
                    .ok()
                    .map(YamlValue::from)
                    .or_else(|| Some(YamlValue::String(v.to_string()))),
                // Default case - keep as string
                _ => Some(YamlValue::String(v.to_string())),
            }
        }
        _ => None,
    }
}

// Serde helpers for CSV field decoding
fn de_bool_flex<'de, D>(deserializer: D) -> std::result::Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    Ok(parse_bool_cell(&raw))
}

fn de_opt_string<'de, D>(deserializer: D) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.and_then(|s| {
        let t = s.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    }))
}

#[derive(Debug, Deserialize)]
pub struct DBSchema {
    pub table: String,
    pub column: String,
    pub column_type: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default, deserialize_with = "de_opt_string")]
    pub default_value: Option<String>,
    #[serde(deserialize_with = "de_bool_flex")]
    pub is_index: bool,
    #[serde(deserialize_with = "de_bool_flex")]
    pub is_nullable: bool,
    #[serde(deserialize_with = "de_bool_flex")]
    pub is_option: bool,
    #[serde(deserialize_with = "de_bool_flex")]
    pub is_primary_key: bool,
    #[serde(deserialize_with = "de_bool_flex")]
    pub is_vec: bool,
}

pub fn load_db_schema_from_csv(path: &Path) -> Result<BTreeMap<String, TableSchema>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_path(path)
        .with_context(|| format!("failed to open CSV: {}", path.display()))?;

    let mut tables: BTreeMap<String, TableSchema> = BTreeMap::new();
    for row in rdr.deserialize::<DBSchema>() {
        let row = row.with_context(|| format!("failed to parse CSV row in {}", path.display()))?;
        let column_type_spec = ColumnTypeSpec {
            column_type: row.column_type,
            r#type: row.r#type,
        };
        let col_spec = ColumnSpec {
            default_value: parse_default_value_cell(
                row.default_value.as_deref(),
                &column_type_spec,
            ),
            column_type: column_type_spec,
            is_index: row.is_index,
            is_nullable: row.is_nullable,
            is_option: row.is_option,
            is_primary_key: row.is_primary_key,
            is_vec: row.is_vec,
        };
        tables
            .entry(row.table)
            .or_insert_with(BTreeMap::new)
            .insert(row.column, col_spec);
    }

    Ok(tables)
}

pub fn load_db_schema_into_custom(custom: &mut CustomConfig, path: &Path) -> Result<()> {
    custom.db_schema = load_db_schema_from_csv(path)?;
    Ok(())
}
