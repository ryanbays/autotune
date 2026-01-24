**autotune > gui > app**

# Module: gui::app

## Contents

**Structs**

- [`App`](#app)

---

## autotune::gui::app::App

*Struct*

**Fields:**
- `titlebar: components::titlebar::TitleBar`
- `toolbar: components::toolbar::Toolbar`
- `clip_manager: components::clips::ClipManager`
- `track_manager: components::track::TrackManager`
- `track_manager_sender: mpsc::Sender<components::track::TrackManagerCommand>`
- `audio_controller_sender: mpsc::Sender<crate::audio::audio_controller::AudioCommand>`

**Methods:**

- `fn new() -> Self`

**Trait Implementations:**

- **App**
  - `fn update(self: & mut Self, ctx: &egui::Context, _frame: & mut eframe::Frame)`
  - `fn on_exit(self: & mut Self, _gl: Option<&eframe::glow::Context>)`



