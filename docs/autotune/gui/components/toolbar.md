**autotune > gui > components > toolbar**

# Module: gui::components::toolbar

## Contents

**Structs**

- [`Toolbar`](#toolbar)

---

## autotune::gui::components::toolbar::Toolbar

*Struct*

**Fields:**
- `zoom_level: f32`
- `volume_level: u32`
- `audio_controller_sender: mpsc::Sender<crate::audio::audio_controller::AudioCommand>`

**Methods:**

- `fn new(audio_controller_sender: mpsc::Sender<AudioCommand>) -> Self`
- `fn get_zoom_level(self: &Self) -> f32`
- `fn show(self: & mut Self, ctx: &egui::Context)`



