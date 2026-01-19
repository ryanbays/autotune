use crate::audio::file;
use crate::gui::components::track;
use eframe::egui::{self, Color32, Sense, Stroke};
use egui::TopBottomPanel;
use std::sync::mpsc;
use tracing::{debug, error, info, warn};

pub struct TitleBar {
    title: String,
    dragging: bool,
}

impl TitleBar {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            dragging: false,
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        track_manager_sender: mpsc::Sender<track::TrackManagerCommand>,
    ) {
        TopBottomPanel::top("title_bar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(&self.title);
                ui.menu_button("File", |ui| {
                    if ui.button("Load audio clip").clicked() {
                        tokio::task::spawn_blocking(move || {
                            let result = rfd::FileDialog::new()
                                .add_filter("WAV Audio", &["wav"])
                                .set_title("Select an audio file")
                                .pick_file();
                            if let Some(path) = result {
                                match file::AudioFileData::load(&path) {
                                    Ok(audio_data) => {
                                        info!("Loaded audio file: {:?}", path);
                                        if let Err(e) = track_manager_sender.send(
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
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let close_response = ui
                        .add(egui::Button::new("❌").frame(false))
                        .on_hover_text("Close");

                    let minimize_response = ui
                        .add(egui::Button::new("🗕").frame(false))
                        .on_hover_text("Minimize");

                    // Title bar response for dragging
                    let title_bar_response =
                        ui.add(egui::Label::new("").sense(Sense::click_and_drag()));

                    // Handle dragging
                    if title_bar_response.is_pointer_button_down_on() {
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
            });
            ui.add_space(4.0);
        });
    }
}
