use crate::audio::audio_controller::AudioCommand;
use tokio::sync::mpsc;
use tracing::{debug, error};

pub struct Toolbar {
    zoom_level: f32,
    volume_level: u32, // Volume level from 0 to 200
    audio_controller_sender: mpsc::Sender<AudioCommand>,
}

impl Toolbar {
    pub fn new(audio_controller_sender: mpsc::Sender<AudioCommand>) -> Self {
        Toolbar {
            zoom_level: 1.0,
            volume_level: 100,
            audio_controller_sender,
        }
    }
    pub fn get_zoom_level(&self) -> f32 {
        self.zoom_level
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar")
            .resizable(false)
            .default_height(40.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("▶").clicked() {
                        debug!("Play button clicked");
                        let result = self.audio_controller_sender.try_send(AudioCommand::Play);
                        if let Err(e) = result {
                            error!("Failed to send Stop command: {}", e);
                        }
                    }
                    if ui.button("⏸").clicked() {
                        let result = self.audio_controller_sender.try_send(AudioCommand::Stop);
                        if let Err(e) = result {
                            error!("Failed to send Stop command: {}", e);
                        }
                    }
                    if ui.button("⏹").clicked() {
                        let result = self.audio_controller_sender.try_send(AudioCommand::Stop);
                        if let Err(e) = result {
                            error!("Failed to send Stop command: {}", e);
                        }
                        let result = self
                            .audio_controller_sender
                            .try_send(AudioCommand::SetReadPosition(0));
                        if let Err(e) = result {
                            error!("Failed to send SetReadPosition command: {}", e);
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Zoom:");
                    ui.add(
                        egui::Slider::new(&mut self.zoom_level, 0.01..=10.0)
                            .text("x")
                            .logarithmic(true),
                    )
                });
                ui.horizontal(|ui| {
                    ui.label("Volume:");
                    ui.add(egui::Slider::new(&mut self.volume_level, 0..=200).text("%"));
                });
            });
        self.audio_controller_sender
            .try_send(AudioCommand::SetVolume(self.volume_level as f32 / 100.0))
            .unwrap_or_else(|e| {
                error!("Failed to send SetVolume command: {}", e);
            });
    }
}
