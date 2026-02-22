pub mod app;
pub mod components;

/// Run the GUI app with UNIX-specific options
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

/// Run the GUI app with Windows-specific options
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
