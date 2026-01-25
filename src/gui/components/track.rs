use crate::{
    audio::{Audio, audio_controller::AudioCommand, file::AudioFileData},
    gui::components::{self, clips::ClipManager, track_menu::TrackMenu},
};
use egui::Sense;
use tokio::sync::mpsc;
use tracing::{debug, error};

const SAMPLES_PER_PIXEL: f32 = 441.0;
/// Constant that defines the amount of pixels to the left of the timeline ruler
/// and track
pub const LEFT_SIDE_PADDING: f32 = 50.0;

/// Helper function that calculates the number of pixels a second of audio takes up based on the sample rate
pub fn calculate_pixels_per_second(sample_rate: u32, zoom_level: f32) -> f32 {
    sample_rate as f32 / SAMPLES_PER_PIXEL * zoom_level
}

/// Enum for cross-thread communication between the TrackManager and the AudioController
pub enum TrackManagerCommand {
    AddAudioClip(AudioFileData),
    SetReadPosition(usize),
}

/// Struct that handles managing tracks and displaying in `egui`
pub struct TrackManager {
    tracks: Vec<Track>,
    horizontal_scroll: f32,
    receiver: mpsc::Receiver<TrackManagerCommand>,
    read_position: usize, // This is in samples
    audio_controller_sender: mpsc::Sender<crate::audio::audio_controller::AudioCommand>,
}

impl TrackManager {
    /// Creates a new TrackManager with the given receiver and audio controller sender
    pub fn new(
        receiver: mpsc::Receiver<TrackManagerCommand>,
        audio_controller_sender: mpsc::Sender<crate::audio::audio_controller::AudioCommand>,
    ) -> Self {
        TrackManager {
            horizontal_scroll: 0.0,
            tracks: Vec::new(),
            receiver,
            read_position: 0,
            audio_controller_sender,
        }
    }
    /// Adds a new track to the TrackManager and returns its ID
    pub fn add_track(&mut self) -> u32 {
        let track_id = self.tracks.len() as u32;
        let track = Track::new(track_id, self.audio_controller_sender.clone());
        track.send_update();
        self.tracks.push(track);
        track_id
    }
    /// Internal function to send commands to the AudioController from the TrackManager
    /// This is non-blocking so if there is nothing in the recv queue it moves on instantly
    /// this means that there may be slight inaccuracies at frame time
    fn audio_controller_communication(&mut self, clip_manager: &mut ClipManager) {
        self.audio_controller_sender
            .try_send(AudioCommand::BroadcastPosition)
            .unwrap_or_else(|e| {
                error!("Failed to send BroadcastPosition command: {}", e);
            });
        while let Ok(command) = self.receiver.try_recv() {
            match command {
                TrackManagerCommand::AddAudioClip(audio_file) => {
                    clip_manager.add_clip(audio_file);
                }
                TrackManagerCommand::SetReadPosition(position) => {
                    self.read_position = position;
                }
            }
        }
    }
    /// Internal function to draw the timeline ruler above the tracks
    fn show_timeline_ruler(&self, zoom_level: f32, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let ruler_width = ui.available_width();
            let ruler_height = 20.0;
            let (ruler_rect, _ruler_response) =
                ui.allocate_exact_size(egui::vec2(ruler_width, ruler_height), Sense::hover());
            let painter = ui.painter_at(ruler_rect);
            let pixels_per_second = calculate_pixels_per_second(44100, zoom_level);
            let scroll_px = self.horizontal_scroll;
            let start_time = (scroll_px / pixels_per_second).max(0.0);
            let first_mark_time = start_time.floor();
            let visible_duration = ruler_width / pixels_per_second;
            let last_mark_time = first_mark_time + visible_duration + 1.0;

            let min_mark_spacing_px = 50.0;
            let mut mark_interval = 1.0; // in seconds
            while mark_interval * pixels_per_second < min_mark_spacing_px {
                mark_interval *= 2.0;
            }

            let mut t = (first_mark_time / mark_interval) as i32;
            while (t as f32) <= last_mark_time / mark_interval {
                let time_sec = t as f32 * mark_interval;

                let x = LEFT_SIDE_PADDING + ruler_rect.left() + time_sec * pixels_per_second
                    - scroll_px;

                // Only draw if inside the ruler rect
                if x >= ruler_rect.left() && x <= ruler_rect.right() {
                    painter.line_segment(
                        [
                            egui::pos2(x, ruler_rect.top()),
                            egui::pos2(x, ruler_rect.bottom()),
                        ],
                        egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY),
                    );
                    painter.text(
                        egui::pos2(x + 2.0, ruler_rect.top() + 2.0),
                        egui::Align2::LEFT_TOP,
                        format!("{:.1}s", time_sec),
                        egui::FontId::default(),
                        egui::Color32::WHITE,
                    );
                }

                t += 1;
            }
        });
    }
    /// Internal function to draw a line indicating the current read position
    fn show_read_pos_line(&self, zoom_level: f32, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let x = LEFT_SIDE_PADDING
            + rect.left()
            + ((self.read_position as f32) * zoom_level / SAMPLES_PER_PIXEL)
            - self.horizontal_scroll;
        if x < LEFT_SIDE_PADDING + rect.left() || x > rect.right() {
            return;
        }
        let painter = ui.painter_at(rect);
        painter.line_segment(
            [
                egui::pos2(x, rect.top() + 30.0),
                egui::pos2(x, rect.top() + 30.0 + self.tracks.len() as f32 * 80.0),
            ],
            egui::Stroke::new(1.0, egui::Color32::RED),
        );
    }
    /// Displays the UI:
    /// * Timeline ruler
    /// * Read position
    /// * All the tracks
    pub fn show(
        &mut self,
        clip_manager: &mut components::clips::ClipManager,
        toolbar: &components::toolbar::Toolbar,
        ctx: &egui::Context,
    ) {
        self.audio_controller_communication(clip_manager);

        let response = egui::CentralPanel::default().show(ctx, |ui| {
            self.show_timeline_ruler(toolbar.get_zoom_level(), ui);

            ui.separator();

            // Show tracks
            let mut i = 0;
            while i < self.tracks.len() {
                let track = &mut self.tracks[i];
                if track.show(i, toolbar.get_zoom_level(), self.horizontal_scroll, ui, ctx) {
                    self.tracks.remove(i);
                    self.audio_controller_sender
                        .try_send(AudioCommand::RemoveTrack(i as u32))
                        .unwrap_or_else(|e| {
                            error!("Failed to send RemoveTrack command: {}", e);
                        });
                } else {
                    i += 1;
                }
            }

            self.show_read_pos_line(toolbar.get_zoom_level(), ui);

            if ui.button("Add Track").clicked() {
                self.add_track();
            }
        });
        if response.response.hovered() {
            if ctx.input(|i| i.raw_scroll_delta.y != 0.0) {
                let scroll_amount = ctx.input(|i| i.raw_scroll_delta.y);
                self.horizontal_scroll += scroll_amount * 0.5;
                self.horizontal_scroll = self.horizontal_scroll.max(0.0);
            }
        }
    }
}

#[derive(Clone)]
pub struct Track {
    id: u32,
    audio: Audio,
    muted: bool,
    soloed: bool,
    menu: TrackMenu,
    audio_controller_sender: mpsc::Sender<AudioCommand>,
}

impl Track {
    pub fn new(id: u32, audio_controller_sender: mpsc::Sender<AudioCommand>) -> Self {
        let mut audio = Audio::new(44100, Vec::new(), Vec::new());
        audio.perform_pyin_background();
        Track {
            id,
            audio,
            muted: false,
            soloed: false,
            menu: TrackMenu::new(),
            audio_controller_sender,
        }
    }
    pub fn send_update(&self) {
        debug!(track_id = self.id, "Sending UpdateTrackAudio command");
        let audio_data = self.audio.clone();
        let cmd = AudioCommand::SendTrack(audio_data, self.id);
        let sender = self.audio_controller_sender.clone();
        tokio::spawn(async move {
            if let Err(e) = sender.send(cmd).await {
                error!("Failed to send UpdateTrackAudio command: {}", e);
            }
        });
    }

    pub fn show(
        &mut self,
        index: usize,
        zoom: f32,
        scroll: f32,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
    ) -> bool {
        if self.menu.is_open() {
            let staying_open = self.menu.show_menu(self.id, &mut self.audio, ui, ctx);
            if !staying_open {
                self.audio_controller_sender
                    .try_send(AudioCommand::SendTrack(self.audio.clone(), self.id))
                    .unwrap_or_else(|e| {
                        error!("Failed to send OpenTrackMenu command: {}", e);
                    });
            }
        }
        let mut wants_delete = false;
        let track_height = 60.0;
        let track_left = ui.max_rect().left() + LEFT_SIDE_PADDING;
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), track_height),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                // Left control area
                ui.vertical(|ui| {
                    ui.set_min_width(LEFT_SIDE_PADDING - 7.0);
                    ui.label(format!("Track {}", index + 1));
                    if ui.button("Tune").on_hover_text("Autotune Track").clicked() {
                        self.menu.open();

                    }

                    ui.horizontal(|ui| {
                        ui.style_mut().spacing.item_spacing.x = 2.0;

                        let solo_button = egui::Button::new("S").selected(self.soloed).fill(if self.soloed {
                            egui::Color32::from_rgb(46, 31, 255)
                        } else {
                            egui::Color32::from_rgb(50, 50, 50)
                        }).min_size(egui::vec2(20.0, 20.0));
                        let response = ui.add(solo_button).on_hover_text("Solo Track");
                        if response.clicked() {
                            self.soloed = !self.soloed;
                            self.send_update();
                        }

                        let mute_button = egui::Button::new("M").selected(self.muted).fill(if self.muted {
                            egui::Color32::from_rgb(200, 10, 10)
                        } else {
                            egui::Color32::from_rgb(50, 50, 50)
                        }).min_size(egui::vec2(20.0, 20.0));
                        let response = ui.add(mute_button).on_hover_text("Mute Track");
                        if response.clicked() {
                            self.muted = !self.muted;
                            self.send_update();
                        }
                    });
                    if ui.small_button("×").on_hover_text("Delete Track").clicked() {
                        wants_delete = true;
                    }
                });
                ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
                let (drop_zone_rsp, payload) = ui.dnd_drop_zone::<AudioFileData, egui::Response>(
                    egui::Frame::default().fill(egui::Color32::TRANSPARENT),
                    |ui| {
                        let desired_size = egui::vec2(ui.available_width(), ui.available_height());
                        let (mut rect, response) =
                            ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());
                        rect.set_left(track_left);
                        let painter = ui.painter_at(rect);
                        painter.rect_filled(rect, 5.0, egui::Color32::from_rgb(50, 50, 50));

                        // Draw waveform (min/max per pixel)
                        let samples = &self.audio.left();
                        let width = rect.width() as usize;

                        for x in 0..width{
                            let sample_idx = ((x as f32 + scroll) / zoom * SAMPLES_PER_PIXEL) as usize;
                            if sample_idx >= samples.len() {
                                break;
                            }
                            let v = samples[sample_idx]; // -1.0 .. 1.0

                            let mid_y = rect.center().y;
                            let amp = v * rect.height() * 0.45;

                            painter.line_segment(
                                [
                                egui::pos2(rect.left() + x as f32, mid_y - amp),
                                egui::pos2(rect.left() + x as f32, mid_y + amp),
                                ],
                                egui::Stroke::new(1.0, egui::Color32::BLUE),
                            );
                        }
                        response
                    },
                    );
                // Handling audio clip drag and drop
                if let Some(clip) = payload {
                    if drop_zone_rsp.inner.hovered() {
                        if let Some(pos) = ui.ctx().pointer_interact_pos() {
                            // Convert absolute position to time/sample index
                            let relative_x = pos.x - drop_zone_rsp.inner.rect.left();
                            let sample_index = ((relative_x / zoom) as usize) * 250;
                            debug!(?pos, ?relative_x, ?sample_index, "Dropped clip at position");
                            let audio_data = clip.to_audio();
                            let result = self.audio.insert_audio_at(sample_index, &audio_data);
                            if let Err(e) = result {
                                error!("Failed to insert audio clip: {}", e);
                                return;
                            }
                            debug!(audio = ?self.audio.length(), "Ending audio length after insertion");
                            self.audio.perform_pyin_background();
                            self.send_update();
                        }
                    }
                }
            },
            );
        wants_delete
    }
}
