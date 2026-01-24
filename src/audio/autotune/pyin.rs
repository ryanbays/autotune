use crate::audio::autotune::{
    FRAME_LENGTH, HOP_LENGTH, MAX_F0, MIN_F0, PYIN_SIGMA, PYIN_THRESHOLD,
};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct PYINData {
    f0: Vec<f32>,
    voiced_flag: Vec<bool>,
    voiced_prob: Vec<f32>,
}

impl PYINData {
    pub fn new(f0: Vec<f32>, voiced_flag: Vec<bool>, voiced_prob: Vec<f32>) -> Self {
        Self {
            f0,
            voiced_flag,
            voiced_prob,
        }
    }
    pub fn f0(&self) -> &Vec<f32> {
        &self.f0
    }

    pub fn voiced_flag(&self) -> &Vec<bool> {
        &self.voiced_flag
    }

    pub fn voiced_prob(&self) -> &Vec<f32> {
        &self.voiced_prob
    }
}

/// Simple RMS energy of a frame, used for voicing / silence detection.
fn frame_rms(frame: &[f32]) -> f32 {
    if frame.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = frame.iter().map(|x| x * x).sum();
    (sum_sq / frame.len() as f32).sqrt()
}

fn difference_function(frame: &[f32], max_lag: usize) -> Vec<f32> {
    let n = frame.len();
    let mut d = vec![0.0; max_lag];

    for tau in 1..max_lag {
        let mut acc = 0.0;
        for i in 0..(n - tau) {
            let diff = frame[i] - frame[i + tau];
            acc += diff * diff;
        }
        d[tau] = acc;
    }
    d
}

fn cumulative_mean_normalized_difference(d: &[f32], max_lag: usize) -> Vec<f32> {
    let mut cmnd = vec![0.0; max_lag];
    let mut running_sum = 0.0;

    for tau in 1..max_lag {
        running_sum += d[tau];
        cmnd[tau] = if running_sum > 0.0 {
            d[tau] * (tau as f32) / running_sum
        } else {
            0.0
        };
    }

    cmnd
}

fn parabolic_interp(cmnd: &[f32], tau: usize) -> f32 {
    let x0 = cmnd[tau - 1];
    let x1 = cmnd[tau];
    let x2 = cmnd[tau + 1];
    let denom = 2.0 * (2.0 * x1 - x2 - x0);
    if denom.abs() < 1e-9 {
        tau as f32
    } else {
        tau as f32 + (x2 - x0) / denom
    }
}

fn find_pitch_candidates(
    cmnd: &[f32],
    threshold: f32,
    min_lag: usize,
    max_lag: usize,
    sample_rate: u32,
) -> (Vec<f32>, Vec<f32>) {
    let mut f0s = Vec::new();
    let mut ps = Vec::new();

    let mut found = false;
    for tau in (min_lag + 1)..(max_lag - 1) {
        if found {
            break;
        }

        let v = cmnd[tau];
        if v < threshold && v < cmnd[tau - 1] && v <= cmnd[tau + 1] {
            found = true;

            let refined_tau = parabolic_interp(cmnd, tau);
            let f0 = sample_rate as f32 / refined_tau;
            let p = (1.0 - v).clamp(0.0, 1.0);

            f0s.push(f0);
            ps.push(p);
        }
    }

    if f0s.is_empty() {
        (vec![0.0], vec![0.0])
    } else {
        (f0s, ps)
    }
}

fn probabilistic_f0_selection(
    f0_candidates: &[f32],
    candidate_probs: &[f32],
    sigma: f32,
    previous_f0: Option<f32>,
) -> (f32, bool, f32) {
    if f0_candidates.is_empty() {
        return (0.0, false, 0.0);
    }
    let mut best_score = 0.0;
    let mut best_f0_i: usize = 0;
    let sigma2 = sigma * sigma;

    for i in 0..f0_candidates.len() {
        let candidate = f0_candidates[i];

        // Hard octave / subharmonic guard
        if let Some(pf0) = previous_f0 {
            if pf0 > 0.0 {
                let ratio = candidate / pf0;
                if ratio < 0.7 || ratio > 1.5 {
                    continue;
                }
            }
        }
        let prob = candidate_probs[i];
        let continuity = if let Some(pf0) = previous_f0 {
            if pf0 > 0.0 && candidate > 0.0 {
                let ratio = candidate / pf0;
                let octave_distance = ratio.log2();
                (-0.5 * (octave_distance * octave_distance) / sigma2).exp()
            } else {
                1.0
            }
        } else {
            1.0
        };
        let score = prob * continuity;
        if score > best_score {
            best_score = score;
            best_f0_i = i;
        }
    }
    // WARNING: Need to add threshold as a parameter to control voiced/unvoiced decision
    let voiced_flag = best_score > 0.5;
    (f0_candidates[best_f0_i], voiced_flag, best_score)
}

pub fn pyin(
    signal: &[f32],
    sample_rate: u32,
    frame_length: Option<usize>,
    hop_length: Option<usize>,
    fmin: Option<f32>,
    fmax: Option<f32>,
    threshold: Option<f32>,
    sigma: Option<f32>,
) -> PYINData {
    let frame_length = frame_length.unwrap_or(FRAME_LENGTH);
    let hop_length = hop_length.unwrap_or(HOP_LENGTH);
    let fmin = fmin.unwrap_or(MIN_F0);
    let fmax = fmax.unwrap_or(MAX_F0);
    let min_lag = (sample_rate as f32 / fmax).floor() as usize;
    let max_lag = (sample_rate as f32 / fmin).ceil() as usize;
    let threshold = threshold.unwrap_or(PYIN_THRESHOLD);
    let sigma = sigma.unwrap_or(PYIN_SIGMA);
    debug!(
        frame_length,
        hop_length, fmin, fmax, min_lag, max_lag, threshold, sigma, "PYIN parameters"
    );

    if signal.len() < frame_length {
        return PYINData {
            f0: Vec::new(),
            voiced_flag: Vec::new(),
            voiced_prob: Vec::new(),
        };
    }

    let n_frames = (signal.len() - frame_length) / hop_length + 1;

    let mut f0 = vec![0.0; n_frames];
    let mut voiced_flag = vec![false; n_frames];
    let mut voiced_prob = vec![0.0; n_frames];
    let mut previous_f0: Option<f32> = None;

    // Simple global RMS to derive a silence threshold.
    let global_rms = frame_rms(signal);
    let silence_rms_threshold = global_rms * 0.02 + 1e-6;
    for i in 0..n_frames {
        let start = i * hop_length;
        let end = start + frame_length;
        let frame = &signal[start..end];

        // Silence / very low energy handling: mark as unvoiced directly.
        let frame_energy = frame_rms(frame);
        if frame_energy < silence_rms_threshold {
            f0[i] = 0.0;
            voiced_flag[i] = false;
            voiced_prob[i] = 0.0;
            previous_f0 = None;
            continue;
        }

        if max_lag <= min_lag + 2 || max_lag >= frame_length {
            f0[i] = 0.0;
            voiced_flag[i] = false;
            voiced_prob[i] = 0.0;
            previous_f0 = None;
            continue;
        }

        let d = difference_function(frame, max_lag);
        let cmnd = cumulative_mean_normalized_difference(&d, max_lag);
        let (f0_candidates, candidate_probs) =
            find_pitch_candidates(&cmnd, threshold, min_lag, max_lag, sample_rate);
        let (best_f0, is_voiced, best_prob) =
            probabilistic_f0_selection(&f0_candidates, &candidate_probs, sigma, previous_f0);

        // Additional guard: reject obviously out-of-range or unstable f0 as unvoiced.
        let mut final_f0 = best_f0;
        let mut final_voiced = is_voiced;
        let mut final_prob = best_prob;

        if !final_voiced || final_f0 <= 0.0 || final_f0 < fmin * 0.8 || final_f0 > fmax * 1.2 {
            final_f0 = 0.0;
            final_voiced = false;
            final_prob = 0.0;
            previous_f0 = None;
        } else {
            previous_f0 = Some(final_f0);
        }

        f0[i] = final_f0;
        voiced_flag[i] = final_voiced;
        voiced_prob[i] = final_prob;
    }

    PYINData {
        f0,
        voiced_flag,
        voiced_prob,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine_wave(freq: f32, sr: u32, len: usize) -> Vec<f32> {
        (0..len)
            .map(|n| (2.0 * std::f32::consts::PI * freq * n as f32 / sr as f32).sin())
            .collect()
    }

    // -------- Low-level helpers --------

    #[test]
    fn test_frame_rms_basic_and_empty() {
        let empty: Vec<f32> = vec![];
        assert_eq!(frame_rms(&empty), 0.0);

        let ones = vec![1.0; 100];
        let rms = frame_rms(&ones);
        assert!((rms - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_difference_function_basic() {
        let frame = vec![1.0, 2.0, 3.0, 4.0];
        let max_lag = 3;
        let d = difference_function(&frame, max_lag);

        // d[1] = (1-2)^2 + (2-3)^2 + (3-4)^2 = 3
        assert!((d[1] - 3.0).abs() < 1e-6);
        // d[2] = (1-3)^2 + (2-4)^2 = 8
        assert!((d[2] - 8.0).abs() < 1e-6);
    }

    #[test]
    fn test_cumulative_mean_normalized_difference_non_negative() {
        let d = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let max_lag = d.len();
        let cmnd = cumulative_mean_normalized_difference(&d, max_lag);

        for tau in 1..max_lag {
            assert!(cmnd[tau].is_finite());
            assert!(cmnd[tau] >= 0.0);
        }
    }

    #[test]
    fn test_find_pitch_candidates_detects_minimum() {
        let sr = 16000;
        let f0_hz = 200.0;
        let tau = (sr as f32 / f0_hz).round() as usize;

        // Simple synthetic CMND: one good minimum around tau
        let mut cmnd = vec![1.0; 512];
        cmnd[tau] = 0.05;
        cmnd[tau - 1] = 0.2;
        cmnd[tau + 1] = 0.2;

        let threshold = 0.1;
        let min_lag = 10;
        let max_lag = 400;
        let (f0_candidates, candidate_probs) =
            find_pitch_candidates(&cmnd, threshold, min_lag, max_lag, sr);

        assert!(!f0_candidates.is_empty());
        assert_eq!(f0_candidates.len(), candidate_probs.len());

        let detected_f0 = f0_candidates[0];
        assert!((detected_f0 - f0_hz).abs() < 5.0);
        assert!((candidate_probs[0] - (1.0 - 0.05)).abs() < 1e-6);
    }

    #[test]
    fn test_find_pitch_candidates_returns_dummy_when_no_minima() {
        let cmnd = vec![1.0; 100];
        let (f0_candidates, candidate_probs) = find_pitch_candidates(&cmnd, 0.1, 10, 90, 16000);

        assert_eq!(f0_candidates.len(), 1);
        assert_eq!(candidate_probs.len(), 1);
        assert_eq!(f0_candidates[0], 0.0);
        assert_eq!(candidate_probs[0], 0.0);
    }

    #[test]
    fn test_probabilistic_f0_selection_empty_input() {
        let (f0, voiced, prob) = probabilistic_f0_selection(&[], &[], PYIN_SIGMA, None);
        assert_eq!(f0, 0.0);
        assert!(!voiced);
        assert_eq!(prob, 0.0);
    }

    #[test]
    fn test_probabilistic_f0_selection_picks_highest_prob_without_continuity() {
        let f0_candidates = vec![100.0, 200.0, 300.0];
        let candidate_probs = vec![0.1, 0.8, 0.3];

        let (f0, voiced, prob) =
            probabilistic_f0_selection(&f0_candidates, &candidate_probs, PYIN_SIGMA, None);

        assert_eq!(f0, 200.0);
        assert!(voiced);
        assert!((prob - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_probabilistic_f0_selection_prefers_continuous_candidate() {
        let f0_candidates = vec![100.0, 200.0];
        // Raw probability prefers 200 Hz
        let candidate_probs = vec![0.6, 0.9];
        let previous_f0 = Some(100.0);

        let (f0, _voiced, _prob) =
            probabilistic_f0_selection(&f0_candidates, &candidate_probs, 0.1, previous_f0);

        // With strong continuity penalty, should prefer 100 Hz (closer to previous_f0)
        assert_eq!(f0, 100.0);
    }

    // -------- High-level pyin behavior --------

    #[test]
    fn test_pyin_detects_clean_sine_pitch() {
        let sr = 16000;
        let f0_hz = 220.0;
        let duration_s = 0.5;
        let len = (sr as f32 * duration_s) as usize;

        let signal = sine_wave(f0_hz, sr, len);

        let result = pyin(
            &signal,
            sr,
            Some(FRAME_LENGTH),
            Some(HOP_LENGTH),
            Some(50.0),
            Some(500.0),
            Some(0.1),
            Some(0.2),
        );

        assert!(!result.f0().is_empty());
        assert_eq!(result.f0().len(), result.voiced_flag().len());
        assert_eq!(result.f0().len(), result.voiced_prob().len());

        let voiced_indices: Vec<usize> = result
            .voiced_flag()
            .iter()
            .enumerate()
            .filter_map(|(i, &v)| if v { Some(i) } else { None })
            .collect();
        assert!(!voiced_indices.is_empty());

        for &i in &voiced_indices {
            let f0_est = result.f0()[i];
            let prob = result.voiced_prob()[i];

            assert!(f0_est > 0.0);
            assert!((f0_est - f0_hz).abs() < 10.0);
            assert!(prob > 0.0);
        }
    }

    #[test]
    fn test_pyin_treats_silence_as_unvoiced() {
        let sr = 16000;
        let duration_s = 0.5;
        let len = (sr as f32 * duration_s) as usize;

        let signal = vec![0.0; len];

        let result = pyin(
            &signal,
            sr,
            Some(FRAME_LENGTH),
            Some(HOP_LENGTH),
            Some(50.0),
            Some(500.0),
            Some(0.1),
            Some(0.2),
        );

        assert_eq!(result.f0().len(), result.voiced_flag().len());
        assert_eq!(result.f0().len(), result.voiced_prob().len());

        for i in 0..result.f0().len() {
            assert!(!result.voiced_flag()[i]);
            assert_eq!(result.f0()[i], 0.0);
            assert_eq!(result.voiced_prob()[i], 0.0);
        }
    }

    #[test]
    fn test_pyin_frequency_outside_range_mostly_unvoiced() {
        let sr = 16000;
        // Use a frequency clearly below fmin
        let f0_hz = 20.0;
        let duration_s = 0.5;
        let len = (sr as f32 * duration_s) as usize;

        let signal = sine_wave(f0_hz, sr, len);

        let result = pyin(
            &signal,
            sr,
            Some(FRAME_LENGTH),
            Some(HOP_LENGTH),
            Some(50.0),
            Some(500.0),
            Some(0.1),
            Some(0.2),
        );

        let voiced_count = result.voiced_flag().iter().filter(|&&v| v).count();
        let total = result.voiced_flag().len();
        assert!(voiced_count * 4 < total); // < 25% voiced
    }

    #[test]
    fn test_pyin_constants_are_sane() {
        assert!(MIN_F0 > 0.0);
        assert!(MAX_F0 > MIN_F0);
        assert!(FRAME_LENGTH > 0);
        assert!(HOP_LENGTH > 0);
        assert!(PYIN_THRESHOLD > 0.0);
        assert!(PYIN_SIGMA > 0.0);
    }
}

