use crate::audio::{Audio, audio_controller::AudioCommand, file::AudioFileData};
use egui::Sense;
use tokio::sync::mpsc;
use tracing::{debug, error};

const SAMPLES_PER_PIXEL: f32 = 441.0;

pub fn calculate_pixels_per_second(sample_rate: u32, zoom_level: f32) -> f32 {
    sample_rate as f32 / SAMPLES_PER_PIXEL * zoom_level
}

pub enum TrackManagerCommand {
    AddAudioClip(AudioFileData),
    SetReadPosition(usize),
}
pub struct TrackManager {
    tracks: Vec<Track>,
    horizontal_scroll: f32,
    audio_files: Vec<AudioFileData>,
    receiver: mpsc::Receiver<TrackManagerCommand>,
    read_position: usize, // This is in samples
    zoom_level: f32,
    audio_controller_sender: mpsc::Sender<crate::audio::audio_controller::AudioCommand>,
}

impl TrackManager {
    pub fn new(
        audio_controller_sender: mpsc::Sender<crate::audio::audio_controller::AudioCommand>,
    ) -> Self {
        TrackManager {
            horizontal_scroll: 0.0,
            tracks: Vec::new(),
            audio_files: Vec::new(),
            receiver: mpsc::channel(1).1,
            read_position: 0,
            zoom_level: 1.0,
            audio_controller_sender,
        }
    }
    pub fn set_receiver(&mut self, receiver: mpsc::Receiver<TrackManagerCommand>) {
        self.receiver = receiver;
    }

    pub fn add_track(&mut self) -> u32 {
        let track_id = self.tracks.len() as u32;
        let track = Track::new(track_id, self.audio_controller_sender.clone());
        track.send_update();
        self.tracks.push(track);
        track_id
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        self.audio_controller_sender
            .try_send(AudioCommand::BroadcastPosition)
            .unwrap_or_else(|e| {
                error!("Failed to send BroadcastPosition command: {}", e);
            });
        while let Ok(command) = self.receiver.try_recv() {
            match command {
                TrackManagerCommand::AddAudioClip(audio_file) => {
                    self.push_audio_file(audio_file);
                }
                TrackManagerCommand::SetReadPosition(position) => {
                    self.read_position = position;
                }
            }
        }
        egui::SidePanel::left("audio_list")
            .resizable(true)
            .default_width(200.0)
            .max_width(250.0)
            .show(ctx, |ui| {
                ui.heading("Audio Clips");
                for (i, clip) in self.audio_files.iter().enumerate() {
                    let id = egui::Id::new(format!("audio_clip_{}", i));
                    let label = egui::Button::selectable(false, (i + 1).to_string());
                    let payload = clip.clone();
                    ui.dnd_drag_source(id, payload, |ui| {
                        ui.add(label);
                    });
                }
            });
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
                    ui.add(egui::Slider::new(&mut self.zoom_level, 0.01..=10.0).text("x"))
                });
            });
        let response = egui::CentralPanel::default().show(ctx, |ui| {
            let left_padding = 27.0;
            // Show timeline ruler
            ui.horizontal(|ui| {
                let ruler_width = ui.available_width();
                let ruler_height = 20.0;
                let (ruler_rect, _ruler_response) =
                    ui.allocate_exact_size(egui::vec2(ruler_width, ruler_height), Sense::hover());
                let painter = ui.painter_at(ruler_rect);
                let pixels_per_second = calculate_pixels_per_second(44100, self.zoom_level);
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

                    let x =
                        left_padding + ruler_rect.left() + time_sec * pixels_per_second - scroll_px;

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
            ui.separator();
            // Show tracks
            let mut i = 0;
            while i < self.tracks.len() {
                let track = &mut self.tracks[i];
                if track.show(i, self.zoom_level, self.horizontal_scroll, ui) {
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
            // Show read position line
            let rect = ui.max_rect();
            let x = left_padding
                + rect.left()
                + ((self.read_position as f32) * self.zoom_level / SAMPLES_PER_PIXEL)
                - self.horizontal_scroll;

            debug!(
                read_position = self.read_position,
                x_position = x,
                "Drawing read position line"
            );

            let painter = ui.painter_at(rect);
            painter.line_segment(
                [
                    egui::pos2(x, rect.top() + 30.0),
                    egui::pos2(x, rect.top() + 30.0 + self.tracks.len() as f32 * 80.0),
                ],
                egui::Stroke::new(1.0, egui::Color32::RED),
            );
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
    pub fn push_audio_file(&mut self, audio_file: AudioFileData) {
        self.audio_files.push(audio_file);
    }
}

#[derive(Clone)]
pub struct Track {
    id: u32,
    audio: Audio,
    muted: bool,
    soloed: bool,
    audio_controller_sender: mpsc::Sender<AudioCommand>,
}

impl Track {
    pub fn new(id: u32, audio_controller_sender: mpsc::Sender<AudioCommand>) -> Self {
        Track {
            id,
            audio: Audio::new(44100, Vec::new(), Vec::new()),
            muted: false,
            soloed: false,
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

    pub fn show(&mut self, index: usize, zoom: f32, scroll: f32, ui: &mut egui::Ui) -> bool {
        let mut wants_delete = false;
        let track_height = 60.0;
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), track_height),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                // Left control area
                ui.vertical(|ui| {
                    ui.label((index + 1).to_string());
                    ui.toggle_value(&mut self.muted, "M");
                    ui.toggle_value(&mut self.soloed, "S");
                    if ui.small_button("X").on_hover_text("Delete Track").clicked() {
                        wants_delete = true;
                    }
                });
                let (drop_zone_rsp, payload) = ui.dnd_drop_zone::<AudioFileData, egui::Response>(
                    egui::Frame::default(),
                    |ui| {
                        let desired_size = egui::vec2(ui.available_width(), ui.available_height());
                        let (rect, response) =
                            ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());
                        let painter = ui.painter_at(rect);

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
