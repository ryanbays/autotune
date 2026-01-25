use crate::audio::{self, Audio};
use crate::gui::components::track::calculate_pixels_per_second;
use egui::Sense;
use tracing::debug;

const LEFT_SIDE_PADDING: f32 = 40.0;
const VERTICAL_NOTE_SPACING: f32 = 15.0;

fn frame_to_screen(
    frame_idx: usize,
    rect: egui::Rect,
    pixels_per_second: f32,
    scroll_px: f32,
) -> f32 {
    let time_sec = frame_idx as f32 * 256.0 / 44100.0;
    LEFT_SIDE_PADDING + rect.left() + time_sec * pixels_per_second - scroll_px
}

/// Map a MIDI value to a y coordinate using fixed spacing per note, taking
/// vertical_scroll into account. Higher MIDI -> smaller y.
fn midi_to_y(
    midi: f32,
    rect: egui::Rect,
    min_midi: f32,
    max_midi: f32,
    vertical_scroll: f32,
) -> f32 {
    let clamped = midi.clamp(min_midi, max_midi);

    // Highest MIDI at the top, lowest at the bottom, using VERTICAL_NOTE_SPACING
    let top_midi = max_midi;

    // y position counted in "note steps" from the top
    let note_offset_from_top = top_midi - clamped; // 0 for top note, increasing downward
    let y = rect.top() + note_offset_from_top * VERTICAL_NOTE_SPACING;

    y + vertical_scroll
}

/// Height of the full note range based on fixed vertical spacing.
fn note_range_to_height(min_midi: f32, max_midi: f32, _rect: egui::Rect) -> f32 {
    let note_span = (max_midi - min_midi).max(1.0);
    note_span * VERTICAL_NOTE_SPACING
}

fn freq_to_y(
    freq: f32,
    rect: egui::Rect,
    min_midi: f32,
    max_midi: f32,
    vertical_scroll: f32,
) -> Option<f32> {
    if freq <= 0.0 {
        return None;
    }

    let note = audio::scales::frequency_to_midi_note(freq) as f32;
    Some(midi_to_y(note, rect, min_midi, max_midi, vertical_scroll))
}

fn y_to_freq(
    y: f32,
    rect: egui::Rect,
    min_midi: f32,
    max_midi: f32,
    vertical_scroll: f32,
) -> Option<f32> {
    // Invert the fixed-spacing mapping used in midi_to_y
    let note_span = (max_midi - min_midi).max(1.0);
    if note_span == 0.0 {
        return None;
    }

    let y_adj = y - vertical_scroll;

    // distance in pixels from the top
    let dy = y_adj - rect.top();

    // how many notes down from the top (0 at top note)
    let note_offset_from_top = dy / VERTICAL_NOTE_SPACING;

    let top_midi = max_midi;
    let midi = top_midi - note_offset_from_top;

    Some(audio::scales::midi_note_to_frequency(midi))
}

/// Track menu that appears to configure the autotune settings for a track
#[derive(Clone)]
pub struct TrackMenu {
    open: bool,
    horizontal_scroll: f32,
    vertical_scroll: f32,
    zoom_level: f32,
    cached_desired_f0: Option<Vec<f32>>,
    apply_autotune: bool,
    volume_level: u32, // Volume level from 0 to 200
}

impl TrackMenu {
    pub fn new() -> Self {
        TrackMenu {
            open: false,
            horizontal_scroll: 0.0,
            vertical_scroll: 0.0,
            zoom_level: 1.0,
            cached_desired_f0: None,
            apply_autotune: false,
            volume_level: 100,
        }
    }
    pub fn open(&mut self) {
        self.open = true;
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
    /// Shows a floating window where the autotune can be configured for a track
    pub fn show_menu(
        &mut self,
        id: u32,
        audio: &mut Audio,
        _ui: &mut egui::Ui,
        ctx: &egui::Context,
    ) -> bool {
        egui::Window::new(format!("Track {} Autotune", id + 1))
            .min_size(egui::vec2(400.0, 300.0))
            .title_bar(false)
            .show(ctx, |ui| {
                // Use the window's context and a unique ID:
                egui::TopBottomPanel::top(format!("autotune_toolbar_track_{}", id)).show_inside(
                    ui,
                    |ui| {
                        ui.horizontal(|ui| {
                            ui.with_layout(
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.label("Autotune Settings");
                                    let close_response = ui
                                        .add(egui::Button::new("❌").frame(false))
                                        .on_hover_text("Close");
                                    if close_response.clicked() {
                                        self.open = false;
                                    }
                                },
                            );
                        });
                        let apply_response =
                            ui.checkbox(&mut self.apply_autotune, "Apply Autotune to this track");
                        if apply_response.changed() {
                            if !self.apply_autotune {
                                self.cached_desired_f0 = Some(
                                    audio
                                        .desired_f0
                                        .as_ref()
                                        .map(|v| v.clone())
                                        .unwrap_or_else(|| vec![]),
                                );
                                audio.desired_f0 = None;
                            } else if let Some(cached) = self.cached_desired_f0.clone() {
                                audio.desired_f0 = Some(cached);
                                self.cached_desired_f0 = None;
                            } else {
                                audio.desired_f0 = Some(
                                    audio
                                        .get_pyin()
                                        .map_or(vec![], |pyin| vec![0.0; pyin.f0().len()]),
                                );
                            }
                        }
                        ui.horizontal(|ui| {
                            ui.label("Zoom:");
                            ui.add(
                                egui::Slider::new(&mut self.zoom_level, 0.01..=3.0)
                                    .text("x")
                                    .logarithmic(true),
                            )
                        });
                        ui.horizontal(|ui| {
                            ui.label("Volume:");
                            ui.add(egui::Slider::new(&mut self.volume_level, 0..=200).text("%"));
                        });
                    },
                );
                // Show timeline ruler for pitch data
                let response = ui.horizontal(|ui| {
                    let ruler_width = ui.available_width();
                    let ruler_height = 20.0;
                    let (ruler_rect, _ruler_response) = ui
                        .allocate_exact_size(egui::vec2(ruler_width, ruler_height), Sense::hover());
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
                            LEFT_SIDE_PADDING + ruler_rect.left() + time_sec * pixels_per_second
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
                // Handle horizontal scrolling
                if response.response.hovered() {
                    if ctx.input(|i| i.raw_scroll_delta.y != 0.0) {
                        let scroll_amount = ctx.input(|i| i.raw_scroll_delta.y);
                        self.horizontal_scroll += scroll_amount * 0.5;
                        self.horizontal_scroll = self.horizontal_scroll.max(0.0);
                    }
                }
                ui.separator();

                let track_height = ui.available_height().min(1000.0);
                let response = ui.horizontal(|ui| {
                    let pitch_data = audio.get_pyin();

                    let mut rect = ui.max_rect();
                    rect.set_bottom(rect.top() + track_height);
                    ui.allocate_rect(rect, Sense::hover());

                    // Show note names on left using MIDI/freq helpers
                    let mut notes = audio::scales::Key::new(
                        audio::scales::Note::C,
                        audio::scales::Scale::Chromatic,
                    )
                    .get_scale_note_names(2, 6);
                    notes.reverse();

                    let painter = ui.painter_at(rect);
                    let mut font = egui::FontId::default();
                    font.size = 10.0;

                    // Determine MIDI range from note names
                    let min_midi = audio::scales::note_name_to_midi_note(
                        &notes.last().cloned().unwrap_or_default(),
                    )
                    .ok()
                    .unwrap_or(0.0) as f32;
                    let max_midi = audio::scales::note_name_to_midi_note(
                        &notes.first().cloned().unwrap_or_default(),
                    )
                    .ok()
                    .unwrap_or(127.0) as f32;

                    let total_note_height = note_range_to_height(min_midi, max_midi, rect);

                    for note_name in notes.iter() {
                        let midi = audio::scales::note_name_to_midi_note(note_name)
                            .ok()
                            .unwrap_or_else(|| {
                                debug!("Failed to get MIDI for note name {}", note_name);
                                0.0
                            }) as f32;

                        let y = midi_to_y(midi, rect, min_midi, max_midi, self.vertical_scroll);

                        if y < rect.top() || y > rect.bottom() {
                            continue;
                        }

                        painter.text(
                            egui::pos2(rect.left(), y),
                            egui::Align2::LEFT_CENTER,
                            note_name,
                            font.clone(),
                            egui::Color32::WHITE,
                        );

                        // Also draw horizontal grid lines if pitch data exists
                        if pitch_data.is_some() {
                            painter.line_segment(
                                [
                                    egui::pos2(rect.left() + LEFT_SIDE_PADDING, y),
                                    egui::pos2(rect.right(), y),
                                ],
                                egui::Stroke::new(0.5, egui::Color32::DARK_GRAY),
                            );
                        }
                    }
                    if let Some(pyin) = pitch_data {
                        // Draw vertical grid lines for time
                        let pixels_per_second = calculate_pixels_per_second(44100, self.zoom_level);
                        let scroll_px = self.horizontal_scroll;
                        let start_time = (scroll_px / pixels_per_second).max(0.0);
                        let first_mark_time = start_time.floor();
                        let visible_duration = rect.width() / pixels_per_second;
                        let last_mark_time = first_mark_time + visible_duration + 1.0;
                        let min_mark_spacing_px = 50.0;
                        let mut mark_interval = 1.0; // in seconds
                        while mark_interval * pixels_per_second < min_mark_spacing_px {
                            mark_interval *= 2.0;
                        }
                        let mut t = (first_mark_time / mark_interval) as i32;
                        while (t as f32) <= last_mark_time / mark_interval {
                            let time_sec = t as f32 * mark_interval;
                            let x = LEFT_SIDE_PADDING + rect.left() + time_sec * pixels_per_second
                                - scroll_px;
                            // Only draw if inside the grid rect
                            if x >= rect.left() + LEFT_SIDE_PADDING && x <= rect.right() {
                                painter.line_segment(
                                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                                    egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
                                );
                            }
                            t += 1;
                        }
                        if let Some(ref mut desired_f0) = audio.desired_f0 {
                            // Ensure same length as pyin
                            if desired_f0.len() < pyin.f0().len() {
                                desired_f0.resize(pyin.f0().len(), 0.0);
                            }
                        }

                        // Draw pitch data
                        let blue = egui::Color32::BLUE;
                        let green = egui::Color32::GREEN;

                        for i in 0..pyin.f0().len() {
                            // ----- original pitch (non-editable) -----
                            if pyin.voiced_prob()[i] >= 0.5 {
                                let x = frame_to_screen(i, rect, pixels_per_second, scroll_px);
                                if x < rect.left() || x > rect.right() {
                                    continue;
                                }
                                if let Some(y) = freq_to_y(
                                    pyin.f0()[i],
                                    rect,
                                    min_midi,
                                    max_midi,
                                    self.vertical_scroll,
                                ) {
                                    if y >= rect.top() && y <= rect.bottom() {
                                        painter.circle_filled(egui::pos2(x, y), 1.5, blue);
                                    }
                                }
                            }
                            if let Some(ref mut desired_f0) = audio.desired_f0 {
                                // ----- desired pitch (editable) -----
                                let desired_freq = desired_f0[i];
                                if desired_freq <= 0.0 {
                                    continue;
                                }

                                let x = frame_to_screen(i, rect, pixels_per_second, scroll_px);
                                if x < rect.left() || x > rect.right() {
                                    continue;
                                }

                                if let Some(y) = freq_to_y(
                                    desired_freq,
                                    rect,
                                    min_midi,
                                    max_midi,
                                    self.vertical_scroll,
                                ) {
                                    if y < rect.top() || y > rect.bottom() {
                                        continue;
                                    }

                                    let point_radius = 3.0;
                                    let point_rect = egui::Rect::from_center_size(
                                        egui::pos2(x, y),
                                        egui::vec2(point_radius * 2.0, point_radius * 2.0),
                                    );

                                    let id = ui.make_persistent_id(("desired_pitch_point", id, i));
                                    let response =
                                        ui.interact(point_rect, id, Sense::click_and_drag());

                                    // draw point
                                    painter.circle_filled(point_rect.center(), point_radius, green);

                                    // handle drag
                                    if response.dragged() {
                                        let drag_delta = response.drag_delta();
                                        let new_y = y + drag_delta.y;

                                        // clamp to rect
                                        let clamped_y = new_y.clamp(rect.top(), rect.bottom());

                                        // invert mapping to get new frequency from y
                                        if let Some(new_freq) = y_to_freq(
                                            clamped_y,
                                            rect,
                                            min_midi,
                                            max_midi,
                                            self.vertical_scroll,
                                        ) {
                                            desired_f0[i] = new_freq;
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        painter.text(
                            egui::pos2(rect.center().x, rect.center().y - 10.0),
                            egui::Align2::CENTER_CENTER,
                            "Pitch data being analazed...",
                            egui::FontId::default(),
                            egui::Color32::LIGHT_GRAY,
                        );
                    }

                    // Return rect and total_note_height so we can clamp outside
                    (rect, total_note_height)
                });
                let (rect, total_note_height) = response.inner;
                if response.response.hovered() {
                    if ctx.input(|i| i.raw_scroll_delta.y != 0.0) {
                        let scroll_amount = ctx.input(|i| i.raw_scroll_delta.y);
                        self.vertical_scroll += scroll_amount * 0.5;

                        // Clamp so we don't scroll past the octaves (allow some overscroll)
                        let max_scroll = 5.0;
                        let min_scroll =
                            (rect.height() - total_note_height - VERTICAL_NOTE_SPACING / 2.0)
                                .min(0.0);
                        self.vertical_scroll = self.vertical_scroll.clamp(min_scroll, max_scroll);
                    }
                }
            });
        self.open
    }
}
