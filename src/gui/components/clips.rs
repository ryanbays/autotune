use crate::audio::file::AudioFileData;

/// Manages the list of audio clips and their display in the GUI
pub struct ClipManager {
    clips: Vec<AudioFileData>,
}

impl ClipManager {
    pub fn new() -> Self {
        ClipManager { clips: Vec::new() }
    }
    pub fn add_clip(&mut self, clip: AudioFileData) {
        self.clips.push(clip);
    }
    /// Displays the list of audio clips in a side panel with drag-and-drop support
    pub fn show(&self, ctx: &egui::Context) {
        egui::SidePanel::left("audio_list")
            .resizable(true)
            .default_width(200.0)
            .max_width(250.0)
            .show(ctx, |ui| {
                ui.heading("Audio Clips");
                for (i, clip) in self.clips.iter().enumerate() {
                    let id = egui::Id::new(format!("audio_clip_{}", i));
                    let label = egui::Button::selectable(false, clip.file_path.to_string_lossy());
                    let payload = clip.clone();
                    ui.dnd_drag_source(id, payload, |ui| {
                        ui.add(label);
                    });
                }
            });
    }
}
