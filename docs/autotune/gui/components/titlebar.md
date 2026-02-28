**autotune > gui > components > titlebar**

# Module: gui::components::titlebar

## Contents

**Structs**

- [`TitleBar`](#titlebar) - Custom title bar component that includes the application title and a file menu for loading audio clips.

---

## autotune::gui::components::titlebar::TitleBar

*Struct*

Custom title bar component that includes the application title and a file menu for loading audio clips.

**Fields:**
- `title: String`
- `track_manager_sender: mpsc::Sender<track::TrackManagerCommand>`

**Methods:**

- `fn new<impl Into<String>>(title: impl Trait, track_manager_sender: mpsc::Sender<track::TrackManagerCommand>) -> Self`
- `fn show(self: & mut Self, ctx: &egui::Context)` - Displays the title bar at the top of the application window with buttons
- `fn handle_window_control(self: &Self, ui: & mut egui::Ui, ctx: &egui::Context)` - On UNIX systems, this function adds custom buttons and window controls



