//! End-to-end integration test for the audio_controller:
//! 1. Load an input audio file.
//! 2. Run PYIN analysis and apply autotune (PSOLA).
//! 3. Save the processed result to disk.
//!
//! This assumes that:
//! - `audio_controller` exposes a constructor that does not depend on GUI,
//! - there are methods to load from a path, run analysis / autotune, and save,
//! - paths are `&str`/`Path`-like and use blocking I/O.
//!
//! Adjust the method names / signatures to match your actual API.

use std::fs;
use std::path::{Path, PathBuf};

use rust_prototype_2::audio::audio_controller::AudioController;

/// Helper: locate a test asset under `tests/assets`.
fn asset_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests");
    p.push("assets");
    p.push(name);
    p
}

/// Helper: create a temp output path under `tests/out`.
fn output_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests");
    p.push("out");
    // Ensure directory exists
    fs::create_dir_all(&p).expect("failed to create tests/out directory");
    p.push(name);
    p
}

#[test]
fn end_to_end_load_autotune_save() -> anyhow::Result<()> {
    // ----- 1. Arrange: pick input and output paths -----
    // You should add a small WAV (e.g. 1–2s mono) at tests/assets/simple_voice.wav
    let input = asset_path("simple_voice.wav");
    assert!(
        input.exists(),
        "Missing test asset: {:?}. Add a small WAV file for E2E tests.",
        input
    );

    let output = output_path("simple_voice_autotuned.wav");
    if output.exists() {
        fs::remove_file(&output)?;
    }

    // ----- 2. Act: construct controller and run pipeline -----
    // Adjust this to match your real constructor. Examples:
    // - AudioController::new()
    // - AudioController::default()
    // - AudioController::new_with_sample_rate(44100)
    let mut controller = AudioController::new();

    // Adjust API calls to what you actually have:
    //
    // Example expected flow:
    //  - load_file(path)
    //  - compute_pyin() / analyze_pitch()
    //  - set_desired_scale / desired_f0 / autotune parameters
    //  - apply_autotune()
    //  - save_to_file(path)
    //
    // If some of these steps are automatic, drop what you don't need.

    // 2.1 Load audio
    controller.load_file(input.to_str().unwrap())?;

    // Sanity check we actually have audio in memory
    {
        let audio = controller.current_audio().expect("audio should be loaded");
        assert!(audio.left().len() > 0);
        assert_eq!(audio.left().len(), audio.right().len());
    }

    // 2.2 Run pitch analysis (PYIN)
    controller.compute_pyin()?;

    {
        let audio = controller.current_audio().unwrap();
        let pyin = audio
            .get_pyin()
            .expect("PYIN data should be computed by compute_pyin()");
        assert!(!pyin.f0().is_empty(), "PYIN f0 should not be empty");
    }

    // 2.3 Configure autotune target.
    // This depends on your API – substitute with whatever you use in the GUI:
    // e.g. controller.set_desired_f0_from_scale("C_major");
    // or    controller.set_desired_f0_from_shift(semitones);
    //
    // For this example we assume a simple "identity" desired F0: keep detected F0
    {
        let audio = controller.current_audio().unwrap();
        let pyin = audio.get_pyin().unwrap();
        controller.set_desired_f0(pyin.f0().clone());
    }

    // 2.4 Apply autotune (PSOLA)
    controller.apply_autotune()?;

    {
        let audio = controller
            .current_audio()
            .expect("autotuned audio should still be present");
        assert!(
            audio.left().len() > 0,
            "autotuned audio should have samples"
        );
    }

    // 2.5 Save to file
    controller.save_to_file(output.to_str().unwrap())?;

    // ----- 3. Assert: output was created and is a valid non-empty audio file -----
    assert!(
        output.exists(),
        "Expected autotuned output file to exist at {:?}",
        output
    );
    let metadata = fs::metadata(&output)?;
    assert!(
        metadata.len() > 0,
        "Expected autotuned output file to be non-empty"
    );

    // Optional: reload via controller or lower-level API to ensure it parses.
    let mut verify_controller = AudioController::new();
    verify_controller.load_file(output.to_str().unwrap())?;
    let verify_audio = verify_controller
        .current_audio()
        .expect("reloaded output audio should be available");
    assert!(
        verify_audio.left().len() > 0,
        "reloaded autotuned audio should have samples"
    );

    Ok(())
}
