use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tempfile::NamedTempFile;
use yeaptor::config::{load_config, YeaptorConfig, Deployment, PackageSpec};
use aptos_types::account_address::AccountAddress;

#[test]
fn test_load_valid_config() {
    let config_content = r#"
format_version = 1
yeaptor_address = "0x1"

[publishers]
test-publisher = "0x10"
another-publisher = "0x20"

[named-addresses]
test-address = "0x30"

[[deployments]]
publisher = "test-publisher"
seed = "test-seed"
packages = [
    { address_name = "test_package", path = "packages/test" },
    { address_name = "another_package", path = "packages/another" },
]

[[deployments]]
publisher = "another-publisher"
seed = "another-seed"
packages = [
    { address_name = "third_package", path = "packages/third" },
]
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), config_content).unwrap();

    let config = load_config(temp_file.path()).unwrap();

    assert_eq!(config.format_version, 1);
    assert_eq!(config.yeaptor_address, AccountAddress::from_hex_literal("0x1").unwrap());

    // Test publishers
    assert_eq!(config.publishers.len(), 2);
    assert_eq!(
        config.publishers.get("test-publisher").unwrap(),
        &AccountAddress::from_hex_literal("0x10").unwrap()
    );
    assert_eq!(
        config.publishers.get("another-publisher").unwrap(),
        &AccountAddress::from_hex_literal("0x20").unwrap()
    );

    // Test named addresses
    assert_eq!(config.named_addresses.len(), 1);
    assert_eq!(
        config.named_addresses.get("test-address").unwrap(),
        &AccountAddress::from_hex_literal("0x30").unwrap()
    );

    // Test deployments
    assert_eq!(config.deployments.len(), 2);

    let first_deployment = &config.deployments[0];
    assert_eq!(first_deployment.publisher, "test-publisher");
    assert_eq!(first_deployment.seed, "test-seed");
    assert_eq!(first_deployment.packages.len(), 2);
    assert_eq!(first_deployment.packages[0].address_name, "test_package");
    assert_eq!(first_deployment.packages[0].path, "packages/test");
    assert_eq!(first_deployment.packages[1].address_name, "another_package");
    assert_eq!(first_deployment.packages[1].path, "packages/another");

    let second_deployment = &config.deployments[1];
    assert_eq!(second_deployment.publisher, "another-publisher");
    assert_eq!(second_deployment.seed, "another-seed");
    assert_eq!(second_deployment.packages.len(), 1);
    assert_eq!(second_deployment.packages[0].address_name, "third_package");
    assert_eq!(second_deployment.packages[0].path, "packages/third");
}

#[test]
fn test_load_minimal_config() {
    let config_content = r#"
format_version = 1
yeaptor_address = "0x1"
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), config_content).unwrap();

    let config = load_config(temp_file.path()).unwrap();

    assert_eq!(config.format_version, 1);
    assert_eq!(config.yeaptor_address, AccountAddress::from_hex_literal("0x1").unwrap());
    assert_eq!(config.publishers.len(), 0);
    assert_eq!(config.named_addresses.len(), 0);
    assert_eq!(config.deployments.len(), 0);
}

#[test]
fn test_load_config_missing_optional_sections() {
    // Test config with only required fields
    let config_content = r#"
format_version = 2
yeaptor_address = "0x42"
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), config_content).unwrap();

    let config = load_config(temp_file.path()).unwrap();

    assert_eq!(config.format_version, 2);
    assert_eq!(config.yeaptor_address, AccountAddress::from_hex_literal("0x42").unwrap());
    // Default empty collections should be created
    assert!(config.publishers.is_empty());
    assert!(config.named_addresses.is_empty());
    assert!(config.deployments.is_empty());
}

#[test]
fn test_load_config_partial_sections() {
    // Test config with only some optional sections present
    let config_content = r#"
format_version = 1
yeaptor_address = "0x1"

[publishers]
test-publisher = "0x10"
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), config_content).unwrap();

    let config = load_config(temp_file.path()).unwrap();

    assert_eq!(config.format_version, 1);
    assert_eq!(config.yeaptor_address, AccountAddress::from_hex_literal("0x1").unwrap());

    // Publishers section is present
    assert_eq!(config.publishers.len(), 1);
    assert_eq!(
        config.publishers.get("test-publisher").unwrap(),
        &AccountAddress::from_hex_literal("0x10").unwrap()
    );

    // Other sections should default to empty
    assert!(config.named_addresses.is_empty());
    assert!(config.deployments.is_empty());
}

#[test]
fn test_load_config_empty_sections() {
    // Test config with explicitly empty sections
    let config_content = r#"
format_version = 1
yeaptor_address = "0x1"

[publishers]

[named-addresses]

"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), config_content).unwrap();

    let config = load_config(temp_file.path()).unwrap();

    assert_eq!(config.format_version, 1);
    assert_eq!(config.yeaptor_address, AccountAddress::from_hex_literal("0x1").unwrap());
    assert_eq!(config.publishers.len(), 0);
    assert_eq!(config.named_addresses.len(), 0);
    assert_eq!(config.deployments.len(), 0);
}

#[test]
fn test_load_nonexistent_file() {
    let result = load_config(Path::new("nonexistent_file.toml"));
    assert!(result.is_err());
}

#[test]
fn test_load_invalid_toml() {
    let invalid_content = r#"
format_version = 1
yeaptor_address = "0x1"
[publishers
invalid toml syntax
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), invalid_content).unwrap();

    let result = load_config(temp_file.path());
    assert!(result.is_err());
}

#[test]
fn test_load_missing_required_fields() {
    let config_content = r#"
# Missing format_version and yeaptor_address
[publishers]
test-publisher = "0x10"
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), config_content).unwrap();

    let result = load_config(temp_file.path());
    assert!(result.is_err());
}

#[test]
fn test_load_invalid_address_format() {
    let config_content = r#"
format_version = 1
yeaptor_address = "invalid_address"

[publishers]

[named-addresses]

deployments = []
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), config_content).unwrap();

    let result = load_config(temp_file.path());
    assert!(result.is_err());
}

#[test]
fn test_load_config_with_hex_addresses() {
    let config_content = r#"
format_version = 1
yeaptor_address = "0x0000000000000000000000000000000000000000000000000000000000000001"
deployments = []
[publishers]
test-publisher = "0x0000000000000000000000000000000000000000000000000000000000000010"

[named-addresses]
test-address = "0x0000000000000000000000000000000000000000000000000000000000000030"


"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), config_content).unwrap();

    let config = load_config(temp_file.path()).unwrap();

    assert_eq!(config.format_version, 1);
    assert_eq!(
        config.yeaptor_address,
        AccountAddress::from_hex_literal("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap()
    );
    assert_eq!(
        config.publishers.get("test-publisher").unwrap(),
        &AccountAddress::from_hex_literal("0x0000000000000000000000000000000000000000000000000000000000000010").unwrap()
    );
    assert_eq!(
        config.named_addresses.get("test-address").unwrap(),
        &AccountAddress::from_hex_literal("0x0000000000000000000000000000000000000000000000000000000000000030").unwrap()
    );
}
#[test]
fn test_config_serialization_format() {
    // Test that the config structure matches expected TOML format
    let config_content = r#"
format_version = 2
yeaptor_address = "0x42"

[publishers]
publisher1 = "0x100"
publisher2 = "0x200"

[named-addresses]
addr1 = "0x300"
addr2 = "0x400"

[[deployments]]
publisher = "publisher1"
seed = "deployment1"
packages = [
    { address_name = "pkg1", path = "path/to/pkg1" },
]

[[deployments]]
publisher = "publisher2"
seed = "deployment2"
packages = [
    { address_name = "pkg2", path = "path/to/pkg2" },
    { address_name = "pkg3", path = "path/to/pkg3" },
]
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), config_content).unwrap();

    let config = load_config(temp_file.path()).unwrap();

    // Verify the structure is correctly parsed
    assert_eq!(config.format_version, 2);
    assert_eq!(config.deployments.len(), 2);
    assert_eq!(config.deployments[0].packages.len(), 1);
    assert_eq!(config.deployments[1].packages.len(), 2);

    // Verify BTreeMap ordering is preserved
    let publisher_keys: Vec<&String> = config.publishers.keys().collect();
    assert_eq!(publisher_keys, vec!["publisher1", "publisher2"]);

    let named_address_keys: Vec<&String> = config.named_addresses.keys().collect();
    assert_eq!(named_address_keys, vec!["addr1", "addr2"]);
}
