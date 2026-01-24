**rust_prototype_1 > audio > clip_manager**

# Module: audio::clip_manager

## Contents

**Structs**

- [`ClipManager`](#clipmanager)

---

## rust_prototype_1::audio::clip_manager::ClipManager

*Struct*

**Fields:**
- `clips: Vec<crate::audio::AudioClip>`
- `clip_receiver: tokio::sync::mpsc::UnboundedReceiver<crate::audio::AudioClip>`
- `clip_sender: tokio::sync::mpsc::UnboundedSender<crate::audio::AudioClip>`

**Methods:**

- `fn new() -> Self`
- `fn update(self: & mut Self)`
- `fn load_through_rfd(self: & mut Self)`



