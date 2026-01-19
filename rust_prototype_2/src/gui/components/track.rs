use crate::audio::{Audio, file::AudioFileData};
use egui::Sense;
use std::sync::mpsc;
use tracing::{debug, info};

pub enum TrackManagerCommand {
    AddAudioClip(AudioFileData),
}
pub struct TrackManager {
    tracks: Vec<Track>,
    next_id: u32,
    audio_files: Vec<AudioFileData>,
    receiver: mpsc::Receiver<TrackManagerCommand>,
    zoom_level: f32,
}

impl TrackManager {
    pub fn new() -> Self {
        TrackManager {
            tracks: Vec::new(),
            next_id: 1,
            audio_files: Vec::new(),
            receiver: mpsc::channel().1,
            zoom_level: 1.0,
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
        egui::CentralPanel::default().show(ctx, |ui| {
            for (i, mut track) in self.tracks.clone().into_iter().enumerate() {
                if track.show(i, self.zoom_level, ui, ctx, self) {
                    self.tracks.remove(i);
                }
            }
            if ui.button("Add Track").clicked() {
                self.add_track();
            }
        });
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
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        manager: &TrackManager,
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
                        let samples_per_pixel = 250.0;
                        let viewport_samples = (width as f32 / (zoom)) as usize;

                        for x in 0..viewport_samples {
                            let sample_x = x * width / viewport_samples;
                            let sample_idx = (x as f32 * samples_per_pixel / zoom) as usize;
                            if sample_idx >= samples.len() {
                                break;
                            }
                            let v = samples[sample_idx]; // -1.0 .. 1.0

                            let mid_y = rect.center().y;
                            let amp = v * rect.height() * 0.45;

                            painter.line_segment(
                                [
                                    egui::pos2(rect.left() + sample_x as f32, mid_y - amp),
                                    egui::pos2(rect.left() + sample_x as f32, mid_y + amp),
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
                        }
                    }
                }
            },
        );
        wants_delete
    }
}
