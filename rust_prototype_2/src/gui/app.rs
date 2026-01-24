use crate::{
    audio::audio_controller,
    gui::components::{self, track},
};
use eframe::egui;
use tokio::sync::mpsc;
use tracing::debug;

pub struct App {
    titlebar: components::titlebar::TitleBar,
    track_manager: components::track::TrackManager,
    track_manager_sender: mpsc::Sender<components::track::TrackManagerCommand>,
    audio_controller_sender: mpsc::Sender<crate::audio::audio_controller::AudioCommand>,
}

impl App {
    pub fn new() -> Self {
        let (audio_controller_sender, audio_controller_recv) =
            mpsc::channel::<audio_controller::AudioCommand>(100);
        let (track_manager_sender, track_manager_recv) =
            mpsc::channel::<track::TrackManagerCommand>(100);
        let result = crate::audio::audio_controller::AudioController::new(
            audio_controller_recv,
            track_manager_sender.clone(),
        );
        let mut audio_controller = match result {
            Ok(controller) => controller,
            Err(e) => {
                panic!("Failed to initialize AudioController: {}", e);
            }
        };
        tokio::spawn(async move {
            audio_controller.run().await;
        });
        let mut track_manager = track::TrackManager::new(audio_controller_sender.clone());
        track_manager.add_track(); // Add an initial track

        track_manager.set_receiver(track_manager_recv);
        Self {
            titlebar: components::titlebar::TitleBar::new("Autotune"),
            track_manager,
            track_manager_sender,
            audio_controller_sender,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(16));
        ctx.set_zoom_factor(1.5);
        let panel_frame = egui::Frame {
            fill: ctx.style().visuals.window_fill(),
            corner_radius: 25.0.into(),
            stroke: ctx.style().visuals.widgets.noninteractive.fg_stroke,
            outer_margin: 0.5.into(),
            inner_margin: 7.5.into(),
            ..Default::default()
        };
        self.titlebar.show(ctx, self.track_manager_sender.clone());
        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| {
                ui.style_mut().interaction.selectable_labels = false;
                self.track_manager.show(ctx);
            });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        debug!("Shutting down AudioController...");
        self.audio_controller_sender
            .try_send(audio_controller::AudioCommand::Shutdown)
            .ok();
    }
}
