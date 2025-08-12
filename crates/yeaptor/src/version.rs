use clap::Parser;

pub type CliResult = Result<String, String>;

#[derive(Parser, Debug)]
pub struct VersionTool {}

impl VersionTool {
    pub async fn execute(self) -> CliResult {
        Ok(format!(
            "yeaptor {} (git: {}, build: {} {})",
            env!("CARGO_PKG_VERSION"),
            option_env!("GIT_DESCRIBE").unwrap_or("unknown"),
            option_env!("BUILD_DATE").unwrap_or("unknown"),
            option_env!("BUILD_TARGET").unwrap_or("unknown")
        ))
    }
}
