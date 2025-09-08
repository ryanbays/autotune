use crate::audio::Key;
use ndarray::{Array1, ArrayView1, s};

#[derive(Debug, Clone)]
pub struct PYinResult {
    pub f0: Array1<f32>,
    pub voiced_flag: Array1<bool>,
    pub voiced_prob: Array1<f32>,
}
impl PYinResult {
    pub fn snap_to_scale(&self, key: Key) -> Array1<f32> {
        let scale_frequencies = key.get_scale_frequencies(2, 6); // From octave 2 to 6
        self.f0
            .iter()
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
}

pub fn difference_function(frame: &ArrayView1<f32>) -> Array1<f32> {
    let frame_length = frame.len();
    let mut diff = Array1::zeros(frame_length);

    for tau in 1..frame_length {
        let mut sum = 0.0;
        for j in 0..(frame_length - tau) {
            let delta = frame[j] - frame[j + tau];
            sum += delta * delta;
        }
        diff[tau] = sum;
    }

    diff
}

pub fn cumulative_mean_normalized_difference(diff: &Array1<f32>) -> Array1<f32> {
    let frame_length = diff.len();
    let mut cmnd = Array1::zeros(frame_length);
    cmnd[0] = 1.0;

    let mut running_sum = 0.0;
    for tau in 1..frame_length {
        running_sum += diff[tau];
        cmnd[tau] = diff[tau] * (tau as f32) / running_sum.max(1e-10);
    }

    cmnd
}

pub fn get_pitch_period_candidates(cmnd: &Array1<f32>, threshold: f32) -> Vec<usize> {
    let mut candidates = Vec::new();
    let frame_length = cmnd.len();

    for tau in 1..frame_length - 1 {
        if cmnd[tau] < threshold && cmnd[tau] < cmnd[tau - 1] && cmnd[tau] < cmnd[tau + 1] {
            candidates.push(tau);
        }
    }

    candidates
}

pub fn pyin(
    y: &Array1<f32>,
    frame_length: usize,
    hop_length: usize,
    sample_rate: u32,
    f_min: f32,
    f_max: f32,
    threshold: f32,
) -> PYinResult {
    let n_frames = 1 + (y.len() - frame_length) / hop_length;

    let mut f0 = Array1::zeros(n_frames);
    let mut voiced_flag = Array1::from_elem(n_frames, false);
    let mut voiced_prob = Array1::zeros(n_frames);

    for i in 0..n_frames {
        let start = i * hop_length;
        let frame = y.slice(s![start..start + frame_length]);

        let diff = difference_function(&frame);
        let cmnd = cumulative_mean_normalized_difference(&diff);
        let candidates = get_pitch_period_candidates(&cmnd, threshold);

        if !candidates.is_empty() {
            let period = candidates[0];
            if period > 0 {
                let freq = sample_rate as f32 / period as f32;

                if f_min <= freq && freq <= f_max {
                    f0[i] = freq;
                    voiced_flag[i] = true;
                    voiced_prob[i] = 1.0 - cmnd[period];
                }
            }
        }
    }

    PYinResult {
        f0,
        voiced_flag,
        voiced_prob,
    }
}
