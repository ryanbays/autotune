use tracing::{error, info};
fn main() {
    info!("Building docs...");
    let mut cmd = std::process::Command::new("cargo");
    cmd.args(&["doc-md", "--no-deps", "--include_private", "-o", "docs/"]).status().unwrap();
    let result = cmd.spawn();
    if let Err(e) = result {
        error!("Failed to spawn cargo doc-md process: {}", e);
    }
}