mod psola;
mod pyin;

use ndarray::Array1;

pub fn estimate_f0(
    samples: &[f32],
    sample_rate: u32,
    frame_length: usize,
    hop_length: usize,
    f_min: f32,
    f_max: f32,
    threshold: f32,
) -> Vec<f32> {
    // Convert input slice to ndarray
    let y = Array1::from_vec(samples.to_vec());
    pyin::pyin(
        &y,
        sample_rate,
        frame_length,
        hop_length,
        f_min,
        f_max,
        threshold,
    )
    .to_vec()
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
    )
    .to_vec()
}
