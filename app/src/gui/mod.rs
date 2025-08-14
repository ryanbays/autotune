mod clips;
mod titlebar;
mod track;

use crate::audio::clip_manager::ClipManager;
use eframe::egui;
use track::Track;

pub struct AutotuneApp {
    tracks: Vec<Track>,
    title_bar: titlebar::CustomTitleBar,
    clip_panel: clips::ClipPanel,
    clip_manager: ClipManager,
}

impl Default for AutotuneApp {
    fn default() -> Self {
        Self {
            tracks: vec![
                Track::new("Track 1".to_string()),
                Track::new("Track 2".to_string()),
            ],
            title_bar: titlebar::CustomTitleBar::new("Autotune"),
            clip_panel: clips::ClipPanel::new(),
            clip_manager: ClipManager::new(),
        }
    }
}

impl eframe::App for AutotuneApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_zoom_factor(1.5);
        self.clip_manager.update();

        self.title_bar.show(ctx, &mut self.clip_manager);
        egui::CentralPanel::default().show(ctx, |ui| {
            self.clip_panel.show(&self.clip_manager, ui, 150.0);
            let mut i = 0;
            for track in &mut self.tracks {
                let timeline_width = ui.available_width();
                let pixels_per_second = 100.0; // Example value, adjust as needed
                let response = track.show(ui, timeline_width, pixels_per_second, i);

                // Handle interaction with the track
                if response.clicked() {
                    println!("Track clicked: {}", track.name);
                }
                i = i + 1;
            }
        });
    }
}
