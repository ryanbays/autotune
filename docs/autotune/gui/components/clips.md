**autotune > gui > components > clips**

# Module: gui::components::clips

## Contents

**Structs**

- [`ClipManager`](#clipmanager)

---

## autotune::gui::components::clips::ClipManager

*Struct*

**Fields:**
- `clips: Vec<crate::audio::file::AudioFileData>`

**Methods:**

- `fn new() -> Self`
- `fn add_clip(self: & mut Self, clip: AudioFileData)`
- `fn get_clips(self: &Self) -> &Vec<AudioFileData>`
- `fn show(self: &Self, ctx: &egui::Context)`



