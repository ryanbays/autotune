//! Integration tests for low-level audio file I/O.
//!
//! These tests exercise the public audio I/O API using real files:
//! - Loading valid audio
//! - Handling invalid / missing files
//! - Round-tripping simple audio through save + load

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use rust_prototype_2::audio::file;

fn asset_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests");
    p.push("assets");
    p.push(name);
    p
}

fn output_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests");
    p.push("out");
    fs::create_dir_all(&p).expect("failed to create tests/out directory");
    p.push(name);
    p
}

#[test]
fn load_valid_wav_file() -> anyhow::Result<()> {
    let input = asset_path("simple_voice.wav");
    assert!(
        input.exists(),
        "Missing test asset: {:?}. Add a small WAV file for I/O tests.",
        input
    );

    let audio = file::load_audio_from_path(&input)?;
    let left = audio.left();
    let right = audio.right();

    assert!(!left.is_empty(), "loaded audio should have samples");
    assert_eq!(
        left.len(),
        right.len(),
        "left/right channels should have same length"
    );
    assert!(
        audio.sample_rate() > 0,
        "audio sample rate should be positive"
    );

    Ok(())
}

#[test]
fn loading_nonexistent_file_returns_error() {
    let bogus = asset_path("this_file_should_not_exist_12345.wav");
    assert!(
        !bogus.exists(),
        "bogus path unexpectedly exists: {:?}",
        bogus
    );

    let result = file::load_audio_from_path(&bogus);
    assert!(
        result.is_err(),
        "expected error when loading nonexistent file, got: {:?}",
        result
    );
}

#[test]
fn loading_invalid_file_returns_error() -> anyhow::Result<()> {
    let path = output_path("not_audio.txt");

    // Create a small text file that is not valid audio.
    {
        let mut f = fs::File::create(&path)?;
        writeln!(f, "this is not an audio file")?;
    }

    let result = file::load_audio_from_path(&path);
    assert!(
        result.is_err(),
        "expected error when loading invalid audio file, got: {:?}",
        result
    );

    Ok(())
}

#[test]
fn round_trip_save_and_load_wav() -> anyhow::Result<()> {
    // Construct a tiny synthetic mono buffer; your API may require stereo,
    // so adapt this to however `Audio` is constructed in `audio::file`.
    let sample_rate = 44100;
    let n_samples = sample_rate / 100; // 10 ms
    let mut left = Vec::with_capacity(n_samples);
    let mut right = Vec::with_capacity(n_samples);

    for n in 0..n_samples {
        let t = n as f32 / sample_rate as f32;
        let sample = (2.0 * std::f32::consts::PI * 440.0 * t).sin();
        left.push(sample);
        right.push(sample);
    }

    let audio = file::AudioBuffer::from_stereo(left.clone(), right.clone(), sample_rate);
    // If your type / constructor differs, adjust the above line accordingly.

    let out_path = output_path("round_trip.wav");
    if out_path.exists() {
        fs::remove_file(&out_path)?;
    }

    file::save_audio_to_path(&audio, &out_path)?;

    assert!(
        out_path.exists(),
        "expected output file to exist at {:?}",
        out_path
    );

    let reloaded = file::load_audio_from_path(&out_path)?;
    assert_eq!(reloaded.sample_rate(), sample_rate);
    assert_eq!(reloaded.left().len(), left.len());
    assert_eq!(reloaded.right().len(), right.len());

    Ok(())
}

