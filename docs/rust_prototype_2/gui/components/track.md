**rust_prototype_2 > gui > components > track**

# Module: gui::components::track

## Contents

**Structs**

- [`Track`](#track)
- [`TrackManager`](#trackmanager) - Struct that handles managing tracks and displaying in `egui`
- [`TrackMenu`](#trackmenu) - Track menu that appears to configure the autotune settings for a track

**Enums**

- [`TrackManagerCommand`](#trackmanagercommand) - Enum for cross-thread communication between the TrackManager and the AudioController

**Functions**

- [`calculate_pixels_per_second`](#calculate_pixels_per_second) - Helper function that calculates the number of pixels a second of audio takes up based on the sample rate

**Constants**

- [`LEFT_SIDE_PADDING`](#left_side_padding) - Constant that defines the amount of pixels to the left of the timeline ruler
- [`SAMPLES_PER_PIXEL`](#samples_per_pixel)

---

## rust_prototype_2::gui::components::track::LEFT_SIDE_PADDING

*Constant*: `f32`

Constant that defines the amount of pixels to the left of the timeline ruler
and track



## rust_prototype_2::gui::components::track::SAMPLES_PER_PIXEL

*Constant*: `f32`



## rust_prototype_2::gui::components::track::Track

*Struct*

**Fields:**
- `id: u32`
- `audio: crate::audio::Audio`
- `muted: bool`
- `soloed: bool`
- `menu: TrackMenu`
- `audio_controller_sender: mpsc::Sender<crate::audio::audio_controller::AudioCommand>`

**Methods:**

- `fn new(id: u32, audio_controller_sender: mpsc::Sender<AudioCommand>) -> Self`
- `fn send_update(self: &Self)`
- `fn show(self: & mut Self, index: usize, zoom: f32, scroll: f32, ui: & mut egui::Ui, ctx: &egui::Context) -> bool`

**Trait Implementations:**

- **Clone**
  - `fn clone(self: &Self) -> Track`



## rust_prototype_2::gui::components::track::TrackManager

*Struct*

Struct that handles managing tracks and displaying in `egui`

**Fields:**
- `tracks: Vec<Track>`
- `horizontal_scroll: f32`
- `receiver: mpsc::Receiver<TrackManagerCommand>`
- `read_position: usize`
- `audio_controller_sender: mpsc::Sender<crate::audio::audio_controller::AudioCommand>`

**Methods:**

- `fn new(receiver: mpsc::Receiver<TrackManagerCommand>, audio_controller_sender: mpsc::Sender<crate::audio::audio_controller::AudioCommand>) -> Self` - Creates a new TrackManager with the given receiver and audio controller sender
- `fn add_track(self: & mut Self) -> u32` - Adds a new track to the TrackManager and returns its ID
- `fn audio_controller_communication(self: & mut Self, clip_manager: & mut ClipManager)` - Internal function to send commands to the AudioController from the TrackManager
- `fn show_timeline_ruler(self: &Self, zoom_level: f32, ui: & mut egui::Ui)` - Internal function to draw the timeline ruler above the tracks
- `fn show_read_pos_line(self: &Self, zoom_level: f32, ui: & mut egui::Ui)` - Internal function to draw a line indicating the current read position
- `fn show(self: & mut Self, clip_manager: & mut components::clips::ClipManager, toolbar: &components::toolbar::Toolbar, ctx: &egui::Context)` - Displays the UI:



## rust_prototype_2::gui::components::track::TrackManagerCommand

*Enum*

Enum for cross-thread communication between the TrackManager and the AudioController

**Variants:**
- `AddAudioClip(crate::audio::file::AudioFileData)`
- `SetReadPosition(usize)`



## rust_prototype_2::gui::components::track::TrackMenu

*Struct*

Track menu that appears to configure the autotune settings for a track

**Fields:**
- `open: bool`
- `horizontal_scroll: f32`
- `vertical_scroll: f32`
- `zoom_level: f32`
- `volume_level: u32`

**Methods:**

- `fn new() -> Self`
- `fn show_menu(self: & mut Self, id: u32, audio: & mut Audio, _ui: & mut egui::Ui, ctx: &egui::Context)` - Shows a floating window where the autotune can be configured for a track

**Trait Implementations:**

- **Clone**
  - `fn clone(self: &Self) -> TrackMenu`



## rust_prototype_2::gui::components::track::calculate_pixels_per_second

*Function*

Helper function that calculates the number of pixels a second of audio takes up based on the sample rate

```rust
fn calculate_pixels_per_second(sample_rate: u32, zoom_level: f32) -> f32
```



