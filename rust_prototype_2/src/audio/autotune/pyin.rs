use crate::autotune::{FRAME_LENGTH, HOP_LENGTH, MAX_F0, MIN_F0, PYIN_SIGMA, PYIN_THRESHOLD};
use tracing::{debug, info};

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
/// Suggested by AI
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
    let mut continuity: f32;
    let mut score: f32;
    let sigma2 = sigma * sigma;

    for i in 0..f0_candidates.len() {
        let candidate = f0_candidates[i];

        // Hard octave / subharmonic guard
        if let Some(pf0) = previous_f0 {
            if pf0 > 0.0 {
                let ratio = candidate / pf0;
                if ratio < 0.7 || ratio > 1.5 {
                    continue; // skip this candidate entirely
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
        score = prob * continuity;
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

// AI written tests
#[cfg(test)]
mod tests {
    use super::*;

    fn sine_wave(freq: f32, sr: u32, len: usize) -> Vec<f32> {
        (0..len)
            .map(|n| (2.0 * std::f32::consts::PI * freq * n as f32 / sr as f32).sin())
            .collect()
    }

    #[test]
    fn test_difference_function_basic() {
        let frame = vec![1.0, 2.0, 3.0, 4.0];
        let min_lag = 1;
        let max_lag = 3;
        let d = difference_function(&frame, max_lag);

        // d[1] = (1-2)^2 + (2-3)^2 + (3-4)^2 = 3
        assert!((d[1] - 3.0).abs() < 1e-6);
        // d[2] = (1-3)^2 + (2-4)^2 = 8
        assert!((d[2] - 8.0).abs() < 1e-6);
    }

    #[test]
    fn test_cumulative_mean_normalized_difference_monotonic() {
        let d = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let min_lag = 1;
        let max_lag = 5;
        let cmnd = cumulative_mean_normalized_difference(&d, max_lag);

        // CMND is defined only from min_lag..max_lag
        // Check that it is finite and non‑negative
        for tau in min_lag..max_lag {
            assert!(cmnd[tau].is_finite());
            assert!(cmnd[tau] >= 0.0);
        }
    }

    #[test]
    fn test_find_pitch_candidates_detects_peak() {
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
    fn test_find_pitch_candidates_returns_dummy_when_empty() {
        let cmnd = vec![1.0; 100];
        let (f0_candidates, candidate_probs) = find_pitch_candidates(&cmnd, 0.1, 10, 90, 16000);

        assert_eq!(f0_candidates.len(), 1);
        assert_eq!(candidate_probs.len(), 1);
        assert_eq!(f0_candidates[0], 0.0);
        assert_eq!(candidate_probs[0], 0.0);
    }

    #[test]
    fn test_probabilistic_f0_selection_no_candidates() {
        let (f0, voiced, prob) = probabilistic_f0_selection(&[], &[], PYIN_SIGMA, None);
        assert_eq!(f0, 0.0);
        assert!(!voiced);
        assert_eq!(prob, 0.0);
    }

    #[test]
    fn test_probabilistic_f0_selection_picks_highest_prob() {
        let f0_candidates = vec![100.0, 200.0, 300.0];
        let candidate_probs = vec![0.1, 0.8, 0.3];

        let (f0, voiced, prob) =
            probabilistic_f0_selection(&f0_candidates, &candidate_probs, PYIN_SIGMA, None);

        assert_eq!(f0, 200.0);
        assert!(voiced);
        // continuity = 1.0 when previous_f0 is None, so prob == best candidate prob
        assert!((prob - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_probabilistic_f0_selection_respects_continuity() {
        let f0_candidates = vec![100.0, 200.0];
        // Raw probability prefers 200 Hz
        let candidate_probs = vec![0.6, 0.9];
        let previous_f0 = Some(100.0);

        let (f0, _voiced, _prob) =
            probabilistic_f0_selection(&f0_candidates, &candidate_probs, 0.1, previous_f0);

        // With strong continuity penalty, should prefer 100 Hz (closer to previous_f0)
        assert_eq!(f0, 100.0);
    }

    #[test]
    fn test_pyin_detects_sine_pitch() {
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

        // Basic sanity: vectors are non-empty and have matching lengths
        assert!(!result.f0().is_empty(), "PYIN returned empty f0");
        assert_eq!(result.f0().len(), result.voiced_flag().len());
        assert_eq!(result.f0().len(), result.voiced_prob().len());

        // There should be at least some voiced frames
        let voiced_indices: Vec<usize> = result
            .voiced_flag()
            .iter()
            .enumerate()
            .filter_map(|(i, &v)| if v { Some(i) } else { None })
            .collect();
        assert!(
            !voiced_indices.is_empty(),
            "PYIN returned no voiced frames for a clean sine"
        );

        // For voiced frames, f0 should be near the true value and prob > 0
        for &i in &voiced_indices {
            let f0_est = result.f0()[i];
            let prob = result.voiced_prob()[i];

            assert!(
                f0_est > 0.0,
                "Voiced frame has non-positive f0 at index {i}"
            );
            assert!(
                (f0_est - f0_hz).abs() < 10.0,
                "Estimated f0 {} too far from true {} at index {}",
                f0_est,
                f0_hz,
                i
            );
            assert!(
                prob > 0.0,
                "Voiced frame has non-positive probability at index {i}"
            );
        }
    }

    #[test]
    fn test_pyin_detects_multiple_sine_pitches() {
        let sr = 16000;
        // Frequences of multiple notes
        let freqs = [100.0, 250.0, 300.0, 450.0, 500.0];
        let duration_s = 0.5;
        let len = (sr as f32 * duration_s) as usize;

        for &f0_hz in &freqs {
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

            assert!(
                !result.f0().is_empty(),
                "PYIN returned empty f0 for freq {}",
                f0_hz
            );
            assert_eq!(result.f0().len(), result.voiced_flag().len());
            assert_eq!(result.f0().len(), result.voiced_prob().len());

            let voiced_indices: Vec<usize> = result
                .voiced_flag()
                .iter()
                .enumerate()
                .filter_map(|(i, &v)| if v { Some(i) } else { None })
                .collect();
            assert!(
                !voiced_indices.is_empty(),
                "PYIN returned no voiced frames for freq {}",
                f0_hz
            );

            let mut mean_f0 = 0.0;
            for &i in &voiced_indices {
                let f0_est = result.f0()[i];
                let prob = result.voiced_prob()[i];

                assert!(
                    f0_est > 0.0,
                    "Voiced frame has non-positive f0 at index {} for freq {}",
                    i,
                    f0_hz
                );
                assert!(
                    prob > 0.0,
                    "Voiced frame has non-positive prob at index {} for freq {}",
                    i,
                    f0_hz
                );
                mean_f0 += f0_est;
            }
            mean_f0 /= voiced_indices.len() as f32;

            assert!(
                (mean_f0 - f0_hz).abs() < 10.0,
                "Mean f0 {} too far from true {}",
                mean_f0,
                f0_hz
            );
        }
    }
    #[test]
    fn test_pyin_silence_is_unvoiced() {
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
            assert!(
                !result.voiced_flag()[i],
                "Silence frame incorrectly marked voiced at {i}"
            );
            assert!(
                result.f0()[i] == 0.0,
                "Silence frame has non-zero f0 at {i}"
            );
            assert!(
                result.voiced_prob()[i] == 0.0,
                "Silence frame has non-zero prob at {i}"
            );
        }
    }

    #[test]
    fn test_pyin_frequency_outside_range_is_unvoiced() {
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

        // We allow a few spurious voiced frames but expect most to be unvoiced
        let voiced_count = result.voiced_flag().iter().filter(|&&v| v).count();
        let total = result.voiced_flag().len();
        assert!(
            voiced_count * 4 < total, // < 25% voiced
            "Too many voiced frames for frequency outside range"
        );
    }

    #[test]
    fn test_pyin_tracks_pitch_glide() {
        let sr = 16000;
        let duration_s = 0.5;
        let len = (sr as f32 * duration_s) as usize;

        // Linear glide from 200 Hz to 400 Hz
        let mut signal = Vec::with_capacity(len);
        for n in 0..len {
            let t = n as f32 / sr as f32;
            let frac = t / duration_s;
            let f = 200.0 + (400.0 - 200.0) * frac;
            let phase = 2.0 * std::f32::consts::PI * f * t;
            signal.push(phase.sin());
        }

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

        // Consider only voiced frames
        let voiced_f0: Vec<f32> = result
            .f0()
            .iter()
            .zip(result.voiced_flag())
            .filter_map(|(&f, &v)| if v { Some(f) } else { None })
            .collect();

        // Need enough voiced frames to say anything
        assert!(
            voiced_f0.len() > 3,
            "Not enough voiced frames in pitch glide"
        );

        // Expect generally increasing f0; allow a few local deviations
        let mut non_increasing = 0;
        for w in voiced_f0.windows(2) {
            if w[1] + 5.0 < w[0] {
                non_increasing += 1;
            }
        }
        assert!(
            non_increasing * 4 < voiced_f0.len(), // < 25% strong drops
            "Estimated f0 does not monotonically increase for glide"
        );
    }

    #[test]
    fn test_pyin_is_invariant_to_global_gain() {
        let sr = 16000;
        let f0_hz = 220.0;
        let duration_s = 0.5;
        let len = (sr as f32 * duration_s) as usize;

        let base = sine_wave(f0_hz, sr, len);
        let gains = [0.1, 0.5, 1.0, 2.0];

        let mut prev_f0_tracks: Option<Vec<f32>> = None;

        for &g in &gains {
            let signal: Vec<f32> = base.iter().map(|x| x * g).collect();

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

            // Extract f0 only for voiced frames
            let f0_track: Vec<f32> = result
                .f0()
                .iter()
                .zip(result.voiced_flag())
                .filter_map(|(&f, &v)| if v { Some(f) } else { None })
                .collect();

            assert!(!f0_track.is_empty(), "No voiced frames for gain {}", g);

            // Mean estimated f0 should be close to target
            let mean_f0: f32 = f0_track.iter().sum::<f32>() / f0_track.len() as f32;
            assert!(
                (mean_f0 - f0_hz).abs() < 10.0,
                "Mean f0 {} too far from {} for gain {}",
                mean_f0,
                f0_hz,
                g
            );

            if let Some(prev) = &prev_f0_tracks {
                let cur = &f0_track;
                let m = prev.len().min(cur.len());
                // Compare only on overlapping prefix
                let mut diff_sum = 0.0;
                for i in 0..m {
                    diff_sum += (prev[i] - cur[i]).abs();
                }
                let mean_diff = diff_sum / m as f32;
                assert!(
                    mean_diff < 5.0,
                    "f0 track changed too much across gains (mean diff {})",
                    mean_diff
                );
            }

            prev_f0_tracks = Some(f0_track);
        }
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
