pub mod app;
mod components;

use tracing::{debug, info};

pub fn run() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_decorations(false),
        ..Default::default()
    };
    eframe::run_native(
        "My App",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::default()))),
    )?;
    Ok(())
}
