use anyhow::Result;
use aptos_types::account_address::AccountAddress;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

// Import IncludedArtifacts from the aptos framework
pub use aptos::move_tool::IncludedArtifacts;
use serde_with::serde_as;

#[derive(Deserialize, Debug, Clone)]
pub struct YeaptorConfig {
    pub format_version: u64,
    pub yeaptor_address: AccountAddress,
    #[serde(default)]
    pub publishers: BTreeMap<String, AccountAddress>,
    #[serde(default, rename = "named-addresses")]
    pub named_addresses: BTreeMap<String, AccountAddress>,
    #[serde(default)]
    pub deployments: Vec<Deployment>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Deployment {
    pub publisher: String,
    pub seed: String,
    #[serde(default)]
    pub packages: Vec<PackageSpec>,
}

#[serde_as]
#[derive(Deserialize, Debug, Clone)]
pub struct PackageSpec {
    pub address_name: String,
    pub path: String,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    #[serde(default)]
    pub include_artifacts: Option<IncludedArtifacts>,
}

pub fn load_config(path: &Path) -> Result<YeaptorConfig> {
    let s = fs::read_to_string(path)?;
    let cfg: YeaptorConfig = toml::from_str(&s)?;
    Ok(cfg)
}
