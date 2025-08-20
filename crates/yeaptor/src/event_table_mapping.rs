use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::Path;

use crate::processor_config::{CustomConfig, EventMapping};

// CSV Loader for event->table mappings
pub fn load_event_table_mappings_from_csv(path: &Path) -> Result<BTreeMap<String, Vec<String>>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_path(path)
        .with_context(|| format!("failed to open CSV: {}", path.display()))?;

    let mut records = rdr.records();
    // Skip header
    let _ = records.next();

    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for result in records {
        let rec =
            result.with_context(|| format!("failed to parse CSV row in {}", path.display()))?;
        if rec.len() < 2 {
            continue;
        }
        let event = rec.get(0).map(|s| s.trim()).unwrap_or("");
        let table = rec.get(1).map(|s| s.trim()).unwrap_or("");
        if event.is_empty() || table.is_empty() {
            continue;
        }
        let entry = map.entry(event.to_string()).or_default();
        if !entry.contains(&table.to_string()) {
            entry.push(table.to_string());
        }
    }

    // Sort table lists for deterministic order
    for v in map.values_mut() {
        v.sort();
    }
    Ok(map)
}

pub fn ensure_events_exist_from_mapping(
    custom: &mut CustomConfig,
    mapping: &BTreeMap<String, Vec<String>>,
) {
    for (event, _tables) in mapping.iter() {
        custom.events.entry(event.clone()).or_insert(EventMapping {
            constant_values: Vec::new(),
            event_fields: BTreeMap::new(),
            event_metadata: BTreeMap::new(),
        });
    }
}
