**rust_prototype_2 > gui > components > titlebar**

# Module: gui::components::titlebar

## Contents

**Structs**

- [`TitleBar`](#titlebar)

---

## rust_prototype_2::gui::components::titlebar::TitleBar

*Struct*

**Fields:**
- `title: String`
- `track_manager_sender: mpsc::Sender<track::TrackManagerCommand>`

**Methods:**

- `fn new<impl Into<String>>(title: impl Trait, track_manager_sender: mpsc::Sender<track::TrackManagerCommand>) -> Self`
- `fn show(self: & mut Self, ctx: &egui::Context)`
- `fn handle_window_control(self: &Self, ui: & mut egui::Ui, ctx: &egui::Context)`



