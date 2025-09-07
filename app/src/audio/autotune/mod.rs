pub mod psola;
pub mod pyin;

use crate::audio::Key;
use ndarray::Array1;

pub fn estimate_f0(
    samples: &[f32],
    frame_length: usize,
    hop_length: usize,
    sample_rate: u32,
    f_min: f32,
    f_max: f32,
    threshold: f32,
) -> Vec<f32> {
    // Convert input slice to ndarray
    let y = Array1::from_vec(samples.to_vec());
    pyin::pyin(
        &y,
        frame_length,
        hop_length,
        sample_rate,
        f_min,
        f_max,
        threshold,
    )
    .f0
    .to_vec()
}

pub fn snap_to_scale(f0: &[f32], key: Key) -> Vec<f32> {
    let scale_frequencies = key.get_scale_frequencies(2, 6); // From octave 2 to 6
    f0.iter()
        .map(|&freq| {
            if freq <= 0.0 {
                return 0.0;
            }
            let mut closest_freq = scale_frequencies[0];
            let mut min_diff = (freq - closest_freq).abs();
            for &scale_freq in &scale_frequencies[1..] {
                let diff = (freq - scale_freq).abs();
                if diff < min_diff {
                    min_diff = diff;
                    closest_freq = scale_freq;
                }
            }
            closest_freq
        })
        .collect()
}

pub fn pitch_shift(
    samples: &[f32],
    target_f0: &[f32],
    sample_rate: u32,
    frame_length: usize,
    hop_length: usize,
    f_min: f32,
    f_max: f32,
) -> Vec<f32> {
    // Convert input slice to ndarray
    let y = Array1::from_vec(samples.to_vec());
    let target_f0 = Array1::from_vec(target_f0.to_vec());

    psola::psola(
        &y,
        &target_f0,
        sample_rate,
        frame_length,
        hop_length,
        f_min,
        f_max,
        None,
    )
    .to_vec()
}
