#![allow(dead_code, unused)]
use crate::audio::{audio_controller, audio_controller::AudioCommand, autotune};
use clap::Parser;
use std::time::Duration;
use tokio::{sync::mpsc, time::sleep};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{EnvFilter, fmt};

mod audio;
mod gui;

fn init_logger(level: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();
}
#[derive(Parser, Debug)]
#[command(name = "Autotune App")]
struct Args {
    /// Log level filter (e.g., error, warn, info, debug)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    init_logger(&args.log_level);

    gui::run().map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(())
}
