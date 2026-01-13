use crate::audio::Key;
use ndarray::{Array1, ArrayView1, Axis, s};

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

    println!("{}", n_frames);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::{Key, Note, Scale};

    const sample_rate: u32 = 44100;

    fn generate_sine_wave(freq: f32, duration: f32) -> Array1<f32> {
        let n_samples = (sample_rate as f32 * duration) as usize;
        Array1::from_iter((0..n_samples).map(|i| {
            let t = i as f32 / sample_rate as f32;
            (2.0 * std::f32::consts::PI * freq * t).sin()
        }))
    }

    fn generate_test_signal(freqs: Vec<f32>) -> Array1<f32> {
        let mut signal = Array1::zeros(0);
        for &freq in &freqs {
            let sine_wave = generate_sine_wave(freq, 1.0);
            signal.append(Axis(0), sine_wave.view()).unwrap();
        }
        signal
    }
    fn generate_silent_signal() -> Array1<f32> {
        Array1::zeros(44100)
    }
    #[test]
    fn test_pyin() {
        let frequencies = vec![440.0, 360.0, 220.0, 618.0];
        let signal = generate_test_signal(frequencies);
        let frame_length = 2048;
        let hop_length = 512;
        let f_min = 80.0;
        let f_max = 1000.0;
        let threshold = 0.1;

        let pyin_result = pyin(
            &signal,
            frame_length,
            hop_length,
            sample_rate,
            f_min,
            f_max,
            threshold,
        );

        // Check that some pitches were detected
        assert!(pyin_result.f0.iter().any(|&f| f > 0.0));
    }

    #[test]
    fn test_snap_to_scale() {
        // Create test PYinResult
        let f0 = Array1::from_vec(vec![440.0, 445.0, 0.0, 220.0, 880.0]);
        let voiced_flag = Array1::from_vec(vec![true, true, false, true, true]);
        let voiced_prob = Array1::from_vec(vec![0.9, 0.8, 0.1, 0.95, 0.85]);

        let pyin_result = PYinResult {
            f0,
            voiced_flag,
            voiced_prob,
        };

        // Create a C major scale
        let key = Key {
            root: Note::C, // Middle C
            scale: Scale::Major,
        };

        // Snap frequencies to scale
        let snapped = pyin_result.snap_to_scale(key);

        assert!((snapped[0] - 440.0).abs() < 1.0);
        assert!((snapped[1] - 440.0).abs() < 1.0);
        assert!(snapped[2] == 0.0);
        assert!((snapped[3] - 220.0).abs() < 1.0);
        assert!((snapped[4] - 880.0).abs() < 1.0);
    }

    #[test]
    fn test_snap_to_scale_empty() {
        let pyin_result = PYinResult {
            f0: Array1::zeros(0),
            voiced_flag: Array1::from_elem(0, false),
            voiced_prob: Array1::zeros(0),
        };

        let key = Key {
            root: Note::C,
            scale: Scale::Major,
        };

        let snapped = pyin_result.snap_to_scale(key);
        assert_eq!(snapped.len(), 0);
    }
}
