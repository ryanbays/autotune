**rust_prototype_1 > gui**

# Module: gui

## Contents

**Modules**

- [`clips`](#clips)
- [`titlebar`](#titlebar)
- [`track`](#track)

**Structs**

- [`AutotuneApp`](#autotuneapp)

---

## rust_prototype_1::gui::AutotuneApp

*Struct*

**Fields:**
- `tracks: Vec<track::Track>`
- `title_bar: titlebar::CustomTitleBar`
- `clip_panel: clips::ClipPanel`
- `clip_manager: crate::audio::clip_manager::ClipManager`

**Trait Implementations:**

- **App**
  - `fn update(self: & mut Self, ctx: &egui::Context, _frame: & mut eframe::Frame)`
- **Default**
  - `fn default() -> Self`



## Module: clips



## Module: titlebar



## Module: track



