use crate::audio::{Audio, audio_controller::{AudioController, AudioCommand}, file::AudioFileData};
use egui::Sense;
use tokio::sync::mpsc;
use tracing::{debug, info, error};

const SAMPLES_PER_PIXEL: f32 = 250.0;

pub fn calculate_pixels_per_second(sample_rate: u32, zoom_level: f32) -> f32 {
    sample_rate as f32 / SAMPLES_PER_PIXEL * zoom_level
}

pub enum TrackManagerCommand {
    AddAudioClip(AudioFileData),
}
pub struct TrackManager {
    tracks: Vec<Track>,
    horizontal_scroll: f32,
    next_id: u32,
    audio_files: Vec<AudioFileData>,
    receiver: mpsc::Receiver<TrackManagerCommand>,
    zoom_level: f32,
    audio_controller_sender: mpsc::Sender<crate::audio::audio_controller::AudioCommand>,
}

impl TrackManager {
    pub fn new(audio_controller_sender: mpsc::Sender<crate::audio::audio_controller::AudioCommand>) -> Self {
        TrackManager {
            horizontal_scroll: 0.0,
            tracks: Vec::new(),
            next_id: 1,
            audio_files: Vec::new(),
            receiver: mpsc::channel(1).1,
            zoom_level: 1.0,
            audio_controller_sender,
        }
    }
    pub fn set_receiver(&mut self, receiver: mpsc::Receiver<TrackManagerCommand>) {
        self.receiver = receiver;
    }

    pub fn add_track(&mut self) -> u32 {
        let track = Track::new(self.next_id);
        self.tracks.push(track);
        self.next_id += 1;
        self.next_id - 1
    }

    pub fn get_track(&self, id: u32) -> Option<&Track> {
        self.tracks.iter().find(|track| track.id() == id)
    }

    pub fn get_track_mut(&mut self, id: u32) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|track| track.id() == id)
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        while let Ok(command) = self.receiver.try_recv() {
            match command {
                TrackManagerCommand::AddAudioClip(audio_file) => {
                    self.push_audio_file(audio_file);
                }
            }
        }
        egui::SidePanel::left("audio_list")
            .resizable(true)
            .default_width(200.0)
            .max_width(250.0)
            .show(ctx, |ui| {
                ui.heading("Audio Clips");
                let height = 50.0;
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
                    if ui.button("▶︎").clicked() {
                        info!("Play button clicked");
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
                        let result = self.audio_controller_sender.try_send(AudioCommand::ClearBuffer);
                        if let Err(e) = result {
                            error!("Failed to send ClearBuffer command: {}", e);
                        }
                    }
                                    });
                ui.horizontal(|ui| {
                    ui.label("Zoom:");
                    if ui
                        .add(egui::Slider::new(&mut self.zoom_level, 0.1..=5.0).text("x"))
                        .changed()
                    {
                        debug!(?self.zoom_level, "Zoom level changed");
                    }
                });
            });
        let response = egui::CentralPanel::default().show(ctx, |ui| {
            // Show timeline ruler
            ui.horizontal(|ui| {
                let left_padding = 27.0;
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

                let mut t = first_mark_time as i32;
                while (t as f32) <= last_mark_time {
                    let time_sec = t as f32;

                    let x = left_padding
                        + ruler_rect.left()
                        + time_sec * pixels_per_second
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
            });            ui.separator();
            // Show tracks
            let mut i = 0;
            while i < self.tracks.len() {
                let track = &mut self.tracks[i];
                if track.show(i, self.zoom_level, self.horizontal_scroll, ui, ctx) {
                    self.tracks.remove(i);
                } else {
                    i += 1;
                }
            }
            if ui.button("Add Track").clicked() {
                self.add_track();
            }
        });
        if response.response.hovered() {
            if ctx.input(|i| i.raw_scroll_delta.y != 0.0) {
                let scroll_amount = ctx.input(|i| i.raw_scroll_delta.y);
                self.horizontal_scroll += scroll_amount * 0.5;
                self.horizontal_scroll = self.horizontal_scroll.max(0.0);
                debug!(?self.horizontal_scroll, "Adjusted horizontal scroll");
            }
        }
    }
    pub fn push_audio_file(&mut self, audio_file: AudioFileData) {
        self.audio_files.push(audio_file);
    } 
    pub fn get_audio_files(&self) -> &Vec<AudioFileData> {
        &self.audio_files
    }
    pub fn get_audio_file(&self, index: usize) -> Option<&AudioFileData> {
        self.audio_files.get(index)
    }
    pub fn get_audio_file_mut(&mut self, index: usize) -> Option<&mut AudioFileData> {
        self.audio_files.get_mut(index)
    }
}

#[derive(Clone)]
struct DraggedClip {
    index: usize,
}

#[derive(Clone)]
pub struct Track {
    id: u32,
    audio: Audio,
    muted: bool,
    soloed: bool,
}

impl Track {
    pub fn new(id: u32) -> Self {
        Track {
            id,
            audio: Audio::new(44100, vec![0.0; 44100 * 1], vec![0.0; 44100 * 1]), // 5 seconds of silence at 44.1kHz
            muted: false,
            soloed: false,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn show(
        &mut self,
        index: usize,
        zoom: f32,
        scroll: f32,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
    ) -> bool {
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
                        let pixels_per_second = calculate_pixels_per_second(self.audio.sample_rate(), zoom); 

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
                if let Some(clip) = payload {
                    if drop_zone_rsp.inner.hovered() {
                        if let Some(pos) = ui.ctx().pointer_interact_pos() {
                            // Convert absolute position to time/sample index
                            let relative_x = pos.x - drop_zone_rsp.inner.rect.left();
                            let sample_index = ((relative_x / zoom) as usize) * 250;
                            debug!(?pos, ?relative_x, ?sample_index, "Dropped clip at position");
                            let audio_data = clip.to_audio();
                            self.audio.insert_audio_at(sample_index, &audio_data);
                            debug!(audio = ?self.audio.length(), "Ending audio length after insertion");
                        }
                    }
                }
            },
        );
        wants_delete
    }
}
