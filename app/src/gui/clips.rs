use crate::audio::clip_manager::ClipManager;
use eframe::egui::{CursorIcon, Id, Sense, Ui};

pub struct ClipPanel {}

struct ClipUI {
    name: String,
    drag_delta: egui::Pos2,
}

impl Default for ClipUI {
    fn default() -> Self {
        ClipUI {
            name: String::new(),
            drag_delta: egui::Pos2::ZERO,
        }
    }
}
impl ClipUI {
    pub fn new(name: String) -> Self {
        ClipUI {
            name,
            drag_delta: egui::Pos2::ZERO,
        }
    }
    fn show(&mut self, ui: &mut Ui, id: Id) {
        ui.horizontal(|ui| {
            ui.label(&self.name);
            self.drag(ui, id, |ui| {
                ui.label("Drag me");
            });
        });
    }
    fn drag(&mut self, ui: &mut Ui, id: Id, body: impl FnOnce(&mut Ui)) {
        let response = ui.scope(body).response;
        let response = ui.interact(response.rect, id, Sense::drag());

        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::Grab);
        }

        if response.dragged() {
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            self.drag_delta += response.drag_delta();
        }
    }
}

impl ClipPanel {
    pub fn new() -> Self {
        ClipPanel {}
    }

    pub fn show(&self, clip_manager: &ClipManager, ui: &mut egui::Ui, width: f32) {
        egui::ScrollArea::vertical()
            .max_width(width)
            .max_height(width * 2.0)
            .show(ui, |ui| {
                ui.set_width(width);
                ui.label("Clips");
                for clip in &clip_manager.clips {
                    ui.label(&clip.name);
                }
            });
    }
}
