use chrono::Utc;
use std::env;
use std::process::Command;

fn main() {
    // Try to get a descriptive git version
    let git_describe = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty", "--broken"])
        .output()
        .ok()
        .and_then(|o| if o.status.success() { Some(String::from_utf8_lossy(&o.stdout).trim().to_string()) } else { None });

    if let Some(desc) = git_describe {
        println!("cargo:rustc-env=GIT_DESCRIBE={}", desc);
    }

    // Build date and target
    let now = Utc::now().to_rfc3339();
    println!("cargo:rustc-env=BUILD_DATE={}", now);
    if let Ok(target) = env::var("TARGET") {
        println!("cargo:rustc-env=BUILD_TARGET={}", target);
    }

    // Invalidate on HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
}
