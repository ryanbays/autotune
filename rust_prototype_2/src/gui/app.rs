use crate::gui::components;
use eframe::egui;

pub struct App {
    titlebar: components::titlebar::TitleBar,
    track_manager: components::track::TrackManager,
    track_manager_sender: std::sync::mpsc::Sender<components::track::TrackManagerCommand>,
}

impl Default for App {
    fn default() -> Self {
        let mut track_manager = components::track::TrackManager::new();
        track_manager.add_track(); // Add an initial track
        let (sender, receiver) =
            std::sync::mpsc::channel::<components::track::TrackManagerCommand>();
        track_manager.set_receiver(receiver);
        Self {
            titlebar: components::titlebar::TitleBar::new("Autotune"),
            track_manager,
            track_manager_sender: sender,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_zoom_factor(1.5);
        self.titlebar.show(ctx, self.track_manager_sender.clone());
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().interaction.selectable_labels = false;
            self.track_manager.show(ctx);
        });
    }
}
