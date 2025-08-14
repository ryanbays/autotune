use crate::audio::clip_manager::ClipManager;
use eframe::egui::{CursorIcon, Id, Sense, Ui};
use std::collections::hash_map::HashMap;

pub struct ClipPanel {
    clips: HashMap<Id, ClipUI>,
}

struct ClipUI {
    name: String,
    uuid: Id,
    drag_delta: egui::Pos2,
}

impl Default for ClipUI {
    fn default() -> Self {
        ClipUI {
            name: String::new(),
            uuid: Id::new("clip_ui"),
            drag_delta: egui::Pos2::ZERO,
        }
    }
}

impl ClipUI {
    pub fn new(name: String, uuid: Id) -> Self {
        ClipUI {
            name,
            uuid,
            drag_delta: egui::Pos2::ZERO,
        }
    }
    fn show(&mut self, ui: &mut Ui, width: f32) {
        let (id, rect) = ui.allocate_space(egui::Vec2::new(width, 30.0));
        println!("Clip rect: {:?}", rect);
        let response = ui.interact(rect, id, Sense::click_and_drag());

        if response.hovered() {
            println!("Clip hovered: {}", self.name);
            ui.ctx().set_cursor_icon(CursorIcon::Grab);
        }
        if response.dragged() {
            println!("Clip dragged: {}", self.name);
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            self.drag_delta += response.drag_delta();
        } else {
            self.drag_delta = egui::Pos2::ZERO;
        }

        let new_min = rect.min + egui::Vec2::new(self.drag_delta.x, self.drag_delta.y);

        ui.painter().rect_stroke(
            egui::Rect::from_min_size(new_min, rect.size()),
            2,
            egui::Stroke::new(1.0, egui::Color32::WHITE),
            egui::StrokeKind::Middle,
        );
        ui.put(
            egui::Rect::from_min_size(new_min, rect.size()),
            egui::Label::new(&self.name).selectable(false),
        );
    }
}

impl Default for ClipPanel {
    fn default() -> Self {
        ClipPanel {
            clips: HashMap::new(),
        }
    }
}

impl ClipPanel {
    pub fn new() -> Self {
        ClipPanel {
            clips: HashMap::new(),
        }
    }

    pub fn show(&mut self, clip_manager: &ClipManager, ui: &mut egui::Ui, width: f32) {
        egui::ScrollArea::vertical()
            .max_width(width)
            .max_height(width * 2.0)
            .show(ui, |ui| {
                ui.label("Clips");
                for clip in clip_manager.clips.iter() {
                    if self.clips.contains_key(&clip.uuid) {
                        if let Some(clip_ui) = self.clips.get_mut(&clip.uuid) {
                            clip_ui.show(ui, width - 20.0);
                        }
                    } else {
                        let mut clip_ui = ClipUI::new(clip.name.clone(), clip.uuid);
                        clip_ui.show(ui, width - 20.0);
                        self.clips.insert(clip.uuid, clip_ui);
                    }
                }
            });
    }
}
