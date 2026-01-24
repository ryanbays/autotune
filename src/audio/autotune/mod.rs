use crate::audio::Audio;

pub mod psola;
pub mod pyin;

// Constants for PYIN and PSOLA
pub const FRAME_LENGTH: usize = 2048;
pub const HOP_LENGTH: usize = 256;

// Constants for just PYIN
pub const PYIN_THRESHOLD: f32 = 0.1;
pub const PYIN_SIGMA: f32 = 0.2;
pub const MIN_F0: f32 = 50.0;
pub const MAX_F0: f32 = 2000.0;

/**
 * Computes a shifted audio signal using the Audio struct's desired f0 and PYIN data.
 * Returns the signal as a new audio struct.
**/
pub fn compute_shifted_audio(audio: &Audio) -> anyhow::Result<Audio> {
    let pyin_data = audio.get_pyin();
    match pyin_data {
        Some(pyin) => {
            let desired_f0: Vec<f32>;
            match &audio.desired_f0 {
                Some(f0) => {
                    desired_f0 = f0.clone();
                }
                None => {
                    return Err(anyhow::anyhow!("No desired F0 data available for audio"));
                }
            }
            let (shifted_left, shifted_right) = rayon::join(
                || {
                    psola::psola(
                        &audio.left().to_vec(),
                        audio.sample_rate(),
                        &pyin,
                        &desired_f0,
                        None,
                        None,
                    )
                },
                || {
                    psola::psola(
                        &audio.right().to_vec(),
                        audio.sample_rate(),
                        &pyin,
                        &desired_f0,
                        None,
                        None,
                    )
                },
            );
            Ok(Audio::new(audio.sample_rate(), shifted_left, shifted_right))
        }
        None => Err(anyhow::anyhow!("No PYIN data available for audio")),
    }
}
