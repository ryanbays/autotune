//! Integration tests for `AudioController` behavior that are
//! more focused than the full end-to-end autotune pipeline.
//!
//! These tests check state transitions and error handling:
//! - Method order (load → compute_pyin → apply_autotune)
//! - Behavior when called in invalid order
//! - Changing autotune parameters / desired F0

use std::path::PathBuf;

use rust_prototype_2::audio::audio_controller::AudioController;

