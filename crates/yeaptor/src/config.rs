use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use aptos_types::account_address::AccountAddress;

#[derive(Deserialize, Debug,Clone)]
pub struct YeaptorConfig {
    pub format_version: u64,
    pub yeaptor_address: AccountAddress,
    #[serde(default)]
    pub publishers: BTreeMap<String, AccountAddress>,
    #[serde(rename = "named-addresses", default)]
    pub named_addresses: BTreeMap<String, AccountAddress>,
    #[serde(default)]
    pub deployments: Vec<Deployment>,
}

#[derive(Deserialize, Debug,Clone)]
pub struct Deployment {
    pub publisher: String,
    pub seed: String,
    pub packages: Vec<PackageSpec>,
}

#[derive(Deserialize, Debug,Clone)]
pub struct PackageSpec {
    pub address_name: String,
    pub path: String,
}

pub fn load_config(path: &Path) -> Result<YeaptorConfig> {
    let s = fs::read_to_string(path)?;
    let cfg: YeaptorConfig = toml::from_str(&s)?;
    Ok(cfg)
}
