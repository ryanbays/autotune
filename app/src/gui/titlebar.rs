use crate::audio::clip_manager::ClipManager;
use eframe::egui::{self, Color32, Sense, Stroke};
use egui::TopBottomPanel;
use rfd::FileDialog;

pub struct CustomTitleBar {
    title: String,
    dragging: bool,
}

impl CustomTitleBar {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            dragging: false,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, clip_manager: &mut ClipManager) {
        TopBottomPanel::top("title_bar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(&self.title);
                ui.menu_button("File", |ui| {
                    if ui.button("Load audio clip").clicked() {
                        clip_manager.load_through_rfd();
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

            // Add a separator line
            ui.painter().line_segment(
                [
                    ui.min_rect().left_bottom() + egui::vec2(-2.0, 0.0),
                    ui.min_rect().right_bottom() + egui::vec2(2.0, 0.0),
                ],
                Stroke::new(1.0, Color32::from_gray(80)),
            );
        });
    }
}
