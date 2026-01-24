**rust_prototype_1 > gui > track**

# Module: gui::track

## Contents

**Structs**

- [`Track`](#track)

**Constants**

- [`LEFT_PADDING`](#left_padding)
- [`RIGHT_PADDING`](#right_padding)
- [`TOP_PADDING`](#top_padding)
- [`TRACK_SPACING`](#track_spacing)

---

## rust_prototype_1::gui::track::LEFT_PADDING

*Constant*: `f32`



## rust_prototype_1::gui::track::RIGHT_PADDING

*Constant*: `f32`



## rust_prototype_1::gui::track::TOP_PADDING

*Constant*: `f32`



## rust_prototype_1::gui::track::TRACK_SPACING

*Constant*: `f32`



## rust_prototype_1::gui::track::Track

*Struct*

**Fields:**
- `name: String`
- `volume: f32`
- `pan: f32`
- `muted: bool`
- `soloed: bool`
- `height: f32`
- `color: egui::Color32`
- `clips: Vec<crate::audio::AudioClip>`

**Methods:**

- `fn new(name: String) -> Self`
- `fn show(self: & mut Self, ui: & mut Ui, timeline_width: f32, pixels_per_second: f32, index: i32) -> Response`

**Trait Implementations:**

- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`



