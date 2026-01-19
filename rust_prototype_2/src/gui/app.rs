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
        let panel_frame = egui::Frame {
            fill: ctx.style().visuals.window_fill(),
            corner_radius: 5.0.into(),
            stroke: ctx.style().visuals.widgets.noninteractive.fg_stroke,
            outer_margin: 0.5.into(),
            inner_margin: 7.5.into(),
            ..Default::default()
        };
        self.titlebar.show(ctx, self.track_manager_sender.clone());
        egui::CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            ui.style_mut().interaction.selectable_labels = false;
            self.track_manager.show(ctx);
        });
    }
}
