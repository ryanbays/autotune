use crate::audio::AudioClip;
use egui::{Color32, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2};

#[derive(Debug)]
pub struct Track {
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub soloed: bool,
    pub height: f32,
    pub color: Color32,
    pub clips: Vec<AudioClip>,
}

const TRACK_SPACING: f32 = 3.0;
const LEFT_PADDING: f32 = 150.0;
const RIGHT_PADDING: f32 = 3.0;
const TOP_PADDING: f32 = 50.0;

impl Track {
    pub fn new(name: String) -> Self {
        Self {
            name,
            volume: 0.0,
            pan: 0.0,
            muted: false,
            soloed: false,
            height: 80.0,
            color: Color32::from_rgb(60, 60, 60),
            clips: Vec::new(),
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        timeline_width: f32,
        pixels_per_second: f32,
        index: i32,
    ) -> Response {
        // Track container
        let desired_size = Vec2::new(timeline_width - (LEFT_PADDING + RIGHT_PADDING), self.height);
        let desired_position = Pos2::new(
            LEFT_PADDING,
            index as f32 * (self.height + TRACK_SPACING) + TOP_PADDING,
        );
        let desired_rect = Rect::from_min_size(desired_position, desired_size);
        let response = ui.allocate_rect(desired_rect, Sense::click_and_drag());

        // Draw track background
        ui.painter().rect_filled(desired_rect, 0.0, self.color);

        // Draw track header (left side)
        let header_width = 200.0;
        let header_rect =
            Rect::from_min_size(desired_rect.min, Vec2::new(header_width, self.height));

        ui.painter()
            .rect_filled(header_rect, 0.0, Color32::from_rgb(40, 40, 40));

        // Draw track name
        ui.put(
            header_rect.shrink(4.0),
            egui::Label::new(&self.name).selectable(false),
        );

        // Draw track controls
        let controls_rect = header_rect.shrink(4.0);
        let button_size = Vec2::new(20.0, 20.0);

        // Mute button
        let mute_rect = Rect::from_min_size(
            Pos2::new(controls_rect.right() - 50.0, controls_rect.top() + 4.0),
            button_size,
        );
        if ui
            .put(mute_rect, egui::Button::new("M").selected(self.muted))
            .clicked()
        {
            self.muted = !self.muted;
        }

        // Solo button
        let solo_rect = Rect::from_min_size(
            Pos2::new(controls_rect.right() - 25.0, controls_rect.top() + 4.0),
            button_size,
        );
        if ui
            .put(solo_rect, egui::Button::new("S").selected(self.soloed))
            .clicked()
        {
            self.soloed = !self.soloed;
        }

        // Draw clips
        for clip in &self.clips {
            let clip_x = header_width + (pixels_per_second);
            let clip_width = pixels_per_second;
            let clip_rect = Rect::from_min_size(
                Pos2::new(clip_x, desired_rect.min.y + 4.0),
                Vec2::new(clip_width, self.height - 8.0),
            );

            // Draw clip background
            ui.painter()
                .rect_filled(clip_rect, 4.0, Color32::from_rgb(80, 80, 80));

            // Draw waveform if available
            if !clip.waveform.is_empty() {
                let points: Vec<Pos2> = clip
                    .waveform
                    .iter()
                    .enumerate()
                    .map(|(i, &value)| {
                        let x =
                            clip_rect.min.x + (i as f32 * clip_width / clip.waveform.len() as f32);
                        let y = clip_rect.center().y + (value * clip_rect.height() / 2.0);
                        Pos2::new(x, y)
                    })
                    .collect();

                ui.painter()
                    .add(Shape::line(points, Stroke::new(1.0, Color32::WHITE)));
            }

            // Draw clip name
            ui.put(clip_rect.shrink(4.0), egui::Label::new(&clip.name).wrap());
        }

        response
    }
}
