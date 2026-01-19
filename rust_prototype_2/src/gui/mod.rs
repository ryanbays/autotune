pub mod app;
mod components;

use tracing::{debug, info};

pub fn run() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "My App",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::default()))),
    )?;
    Ok(())
}
