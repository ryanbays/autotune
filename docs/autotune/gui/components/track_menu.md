**autotune > gui > components > track_menu**

# Module: gui::components::track_menu

## Contents

**Structs**

- [`TrackMenu`](#trackmenu) - Track menu that appears to configure the autotune settings for a track

**Functions**

- [`frame_to_screen`](#frame_to_screen)
- [`freq_to_y`](#freq_to_y)
- [`midi_to_y`](#midi_to_y) - Map a MIDI value to a y coordinate using fixed spacing per note, taking
- [`note_range_to_height`](#note_range_to_height) - Height of the full note range based on fixed vertical spacing.
- [`y_to_freq`](#y_to_freq)

**Constants**

- [`LEFT_SIDE_PADDING`](#left_side_padding)
- [`VERTICAL_NOTE_SPACING`](#vertical_note_spacing)

---

## autotune::gui::components::track_menu::LEFT_SIDE_PADDING

*Constant*: `f32`



## autotune::gui::components::track_menu::TrackMenu

*Struct*

Track menu that appears to configure the autotune settings for a track

**Fields:**
- `open: bool`
- `horizontal_scroll: f32`
- `vertical_scroll: f32`
- `zoom_level: f32`
- `cached_desired_f0: Option<Vec<f32>>`
- `apply_autotune: bool`
- `volume_level: u32`

**Methods:**

- `fn new() -> Self`
- `fn open(self: & mut Self)`
- `fn is_open(self: &Self) -> bool`
- `fn show_menu(self: & mut Self, id: u32, audio: & mut Audio, _ui: & mut egui::Ui, ctx: &egui::Context) -> bool` - Shows a floating window where the autotune can be configured for a track

**Trait Implementations:**

- **Clone**
  - `fn clone(self: &Self) -> TrackMenu`



## autotune::gui::components::track_menu::VERTICAL_NOTE_SPACING

*Constant*: `f32`



## autotune::gui::components::track_menu::frame_to_screen

*Function*

```rust
fn frame_to_screen(frame_idx: usize, rect: egui::Rect, pixels_per_second: f32, scroll_px: f32) -> f32
```



## autotune::gui::components::track_menu::freq_to_y

*Function*

```rust
fn freq_to_y(freq: f32, rect: egui::Rect, min_midi: f32, max_midi: f32, vertical_scroll: f32) -> Option<f32>
```



## autotune::gui::components::track_menu::midi_to_y

*Function*

Map a MIDI value to a y coordinate using fixed spacing per note, taking
vertical_scroll into account. Higher MIDI -> smaller y.

```rust
fn midi_to_y(midi: f32, rect: egui::Rect, min_midi: f32, max_midi: f32, vertical_scroll: f32) -> f32
```



## autotune::gui::components::track_menu::note_range_to_height

*Function*

Height of the full note range based on fixed vertical spacing.

```rust
fn note_range_to_height(min_midi: f32, max_midi: f32, _rect: egui::Rect) -> f32
```



## autotune::gui::components::track_menu::y_to_freq

*Function*

```rust
fn y_to_freq(y: f32, rect: egui::Rect, min_midi: f32, max_midi: f32, vertical_scroll: f32) -> Option<f32>
```



