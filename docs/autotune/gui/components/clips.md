**autotune > gui > components > clips**

# Module: gui::components::clips

## Contents

**Structs**

- [`ClipManager`](#clipmanager) - Manages the list of audio clips and their display in the GUI

---

## autotune::gui::components::clips::ClipManager

*Struct*

Manages the list of audio clips and their display in the GUI

**Fields:**
- `clips: Vec<crate::audio::file::AudioFileData>`

**Methods:**

- `fn new() -> Self`
- `fn add_clip(self: & mut Self, clip: AudioFileData)`
- `fn show(self: &Self, ctx: &egui::Context)` - Displays the list of audio clips in a side panel with drag-and-drop support



