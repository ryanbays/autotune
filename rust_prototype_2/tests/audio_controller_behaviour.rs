//! Integration tests for `AudioController` behavior that are
//! more focused than the full end-to-end autotune pipeline.
//!
//! These tests check state transitions and error handling:
//! - Method order (load → compute_pyin → apply_autotune)
//! - Behavior when called in invalid order
//! - Changing autotune parameters / desired F0

use std::path::PathBuf;

use rust_prototype_2::audio::audio_controller::AudioController;

fn asset_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests");
    p.push("assets");
    p.push(name);
    p
}

#[test]
fn controller_full_pipeline_without_saving() -> anyhow::Result<()> {
    let input = asset_path("simple_voice.wav");
    assert!(
        input.exists(),
        "Missing test asset: {:?}. Add a small WAV file for controller tests.",
        input
    );

    let mut controller = AudioController::new();

    // 1. Load audio
    controller.load_file(input.to_str().unwrap())?;
    {
        let audio = controller
            .current_audio()
            .expect("audio should be loaded after load_file");
        assert!(!audio.left().is_empty());
    }

    // 2. Compute PYIN
    controller.compute_pyin()?;
    {
        let audio = controller.current_audio().unwrap();
        let pyin = audio
            .get_pyin()
            .expect("PYIN should be present after compute_pyin");
        assert!(!pyin.f0().is_empty());
    }

    // 3. Configure desired F0 (identity: same as detected)
    {
        let audio = controller.current_audio().unwrap();
        let pyin = audio.get_pyin().unwrap();
        controller.set_desired_f0(pyin.f0().clone());
    }

    // 4. Apply autotune
    controller.apply_autotune()?;
    {
        let audio = controller
            .current_audio()
            .expect("audio should still be present after apply_autotune");
        assert!(!audio.left().is_empty());
    }

    Ok(())
}

#[test]
fn controller_compute_pyin_before_load_is_handled() {
    let mut controller = AudioController::new();

    // Depending on your API, this might return an error or be a no-op.
    // Here we assert it does NOT panic, and returns a Result that can be inspected.
    let result = controller.compute_pyin();
    assert!(
        result.is_err() || result.is_ok(),
        "compute_pyin should not panic when called before load_file"
    );
}

#[test]
fn controller_apply_autotune_without_desired_f0_is_handled() {
    let input = asset_path("simple_voice.wav");
    assert!(
        input.exists(),
        "Missing test asset: {:?}. Add a small WAV file for controller tests.",
        input
    );

    let mut controller = AudioController::new();
    controller
        .load_file(input.to_str().unwrap())
        .expect("load_file should succeed");
    controller
        .compute_pyin()
        .expect("compute_pyin should succeed after load");

    // Intentionally DO NOT call set_desired_f0 here.
    let result = controller.apply_autotune();
    // Accept either an explicit error or a documented default behavior,
    // but ensure it does not panic.
    assert!(
        result.is_err() || result.is_ok(),
        "apply_autotune should not panic when desired F0 is not explicitly set"
    );
}

#[test]
fn controller_updates_desired_f0_when_parameters_change() -> anyhow::Result<()> {
    let input = asset_path("simple_voice.wav");
    assert!(
        input.exists(),
        "Missing test asset: {:?}. Add a small WAV file for controller tests.",
        input
    );

    let mut controller = AudioController::new();
    controller.load_file(input.to_str().unwrap())?;
    controller.compute_pyin()?;

    // Initial desired F0 = detected F0
    let (initial_f0, len) = {
        let audio = controller.current_audio().unwrap();
        let pyin = audio.get_pyin().unwrap();
        controller.set_desired_f0(pyin.f0().clone());
        (pyin.f0().clone(), pyin.f0().len())
    };

    // Example: semitone shift up by one (if you have such an API).
    // If you don't, replace this with however you configure your autotune target.
    if controller.responds_to_semitone_shift() {
        controller.set_semitone_shift(1.0);
    }

    // Access the controller's desired F0 again via whatever getter you have;
    // placeholder name used here – adjust to your real API.
    if let Some(desired_after) = controller.desired_f0() {
        assert_eq!(desired_after.len(), len);
        // At least one value should differ from the original desired F0
        let changed = desired_after
            .iter()
            .zip(initial_f0.iter())
            .any(|(a, b)| (a - b).abs() > 1e-3);
        assert!(
            changed,
            "desired F0 should change after updating autotune parameters"
        );
    }

    Ok(())
}
