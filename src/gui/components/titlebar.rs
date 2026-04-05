use crate::audio::audio_controller;
use crate::audio::file;
use crate::gui::components::track;
use eframe::egui::{self, Sense};
use egui::TopBottomPanel;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// Custom title bar component that includes the application title and a file menu for loading audio clips.
pub struct TitleBar {
    title: String,
    track_manager_sender: mpsc::Sender<track::TrackManagerCommand>,
    audio_controller_sender: mpsc::Sender<audio_controller::AudioCommand>,
}

impl TitleBar {
    pub fn new(
        title: impl Into<String>,
        track_manager_sender: mpsc::Sender<track::TrackManagerCommand>,
        audio_controller_sender: mpsc::Sender<audio_controller::AudioCommand>,
    ) -> Self {
        Self {
            track_manager_sender,
            audio_controller_sender,
            title: title.into(),
        }
    }
    /// Displays the title bar at the top of the application window with buttons
    pub fn show(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("title_bar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                #[cfg(unix)]
                ui.label(&self.title);
                ui.menu_button("File", |ui| {
                    if ui.button("Load audio clip").clicked() {
                        let tx = self.track_manager_sender.clone();
                        tokio::task::spawn_blocking(move || {
                            let result = rfd::FileDialog::new()
                                .add_filter("WAV Audio", &["wav"])
                                .set_title("Select an audio file")
                                .pick_file();
                            if let Some(path) = result {
                                match file::AudioFileData::load(&path) {
                                    Ok(audio_data) => {
                                        info!("Loaded audio file: {:?}", path);
                                        if let Err(e) = tx.try_send(
                                            track::TrackManagerCommand::AddAudioClip(audio_data),
                                        ) {
                                            error!(
                                                "Failed to send audio clip to track manager: {}",
                                                e
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        error!(?path, "Failed to load audio file: {}", e);
                                    }
                                }
                            } else {
                                debug!("No file selected");
                            }
                        });
                    }
                    if ui.button("Export mixdown").clicked() {
                        debug!("Export mixdown clicked");
                        let tx = self.audio_controller_sender.clone();
                        tokio::task::spawn_blocking(move || {
                            let result = rfd::FileDialog::new()
                                .add_filter("WAV Audio", &["wav"])
                                .set_title("Export mixdown")
                                .save_file();
                            if let Some(path) = result {
                                if let Err(e) =
                                    tx.try_send(audio_controller::AudioCommand::ExportMixdown(path))
                                {
                                    error!("Failed to send export command to track manager: {}", e);
                                }
                            } else {
                                debug!("No file selected for export");
                            }
                        });
                    }
                });
                self.handle_window_control(ui, ctx);
            });
            ui.add_space(4.0);
        });
    }
    /// Windows handles this so this functions does nothing.
    #[cfg(windows)]
    fn handle_window_control(&self, _ui: &mut egui::Ui, _ctx: &egui::Context) {}
    /// On UNIX systems, this function adds custom buttons and window controls
    #[cfg(unix)]
    fn handle_window_control(&self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let close_response = ui
                .add(egui::Button::new("❌").frame(false))
                .on_hover_text("Close");

            let minimize_response = ui
                .add(egui::Button::new("🗕").frame(false))
                .on_hover_text("Minimize");

            // Title bar response for dragging
            let title_bar_response = ui.add(egui::Label::new("").sense(Sense::click_and_drag()));

            // Handle dragging
            if title_bar_response.clicked() {
                debug!("Dragging title bar");
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            // Handle close button
            if close_response.clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            // Handle minimize button
            if minimize_response.clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
        });
    }
}
