**rust_prototype_1 > gui > clips**

# Module: gui::clips

## Contents

**Structs**

- [`ClipPanel`](#clippanel)
- [`ClipUI`](#clipui)

---

## rust_prototype_1::gui::clips::ClipPanel

*Struct*

**Fields:**
- `clips: std::collections::hash_map::HashMap<eframe::egui::Id, ClipUI>`

**Methods:**

- `fn new() -> Self`
- `fn show(self: & mut Self, clip_manager: &ClipManager, ui: & mut egui::Ui, width: f32)`

**Trait Implementations:**

- **Default**
  - `fn default() -> Self`



## rust_prototype_1::gui::clips::ClipUI

*Struct*

**Fields:**
- `name: String`
- `uuid: eframe::egui::Id`
- `drag_delta: egui::Pos2`

**Methods:**

- `fn new(name: String, uuid: Id) -> Self`
- `fn show(self: & mut Self, ui: & mut Ui, width: f32)`

**Trait Implementations:**

- **Default**
  - `fn default() -> Self`



