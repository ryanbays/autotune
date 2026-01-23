pub mod app;
pub mod components;

use tracing::{debug, info};

#[cfg(unix)]
pub fn run() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_decorations(false),
        ..Default::default()
    };
    eframe::run_native(
        "Autotune",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::new()))),
    )?;
    Ok(())
}

#[cfg(windows)]
pub fn run() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        vsync: false,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_decorations(true),
        ..Default::default()
    };
    eframe::run_native(
        "Autotune",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::new()))),
    )?;
    Ok(())
}

