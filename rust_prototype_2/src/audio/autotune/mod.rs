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

// AI written tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::autotune::psola;
    use crate::audio::autotune::pyin;

    fn gen_sine(freq: f32, sample_rate: u32, duration_s: f32) -> Vec<f32> {
        let n_samples = (duration_s * sample_rate as f32).round() as usize;
        let mut out = Vec::with_capacity(n_samples);
        let two_pi_f = 2.0_f32 * std::f32::consts::PI * freq;
        for n in 0..n_samples {
            let t = n as f32 / sample_rate as f32;
            out.push((two_pi_f * t).sin());
        }
        out
    }

    /// Very small DFT-based dominant frequency estimator for testing.
    fn estimate_dominant_freq(signal: &[f32], sample_rate: u32) -> f32 {
        let n = signal.len();
        if n == 0 {
            return 0.0;
        }

        // Zero-mean to reduce DC
        let mean = signal.iter().copied().sum::<f32>() / n as f32;
        let mut x: Vec<f32> = signal.iter().map(|v| v - mean).collect();

        // Hann window
        for (i, v) in x.iter_mut().enumerate() {
            let w = 0.5_f32
                - 0.5_f32 * (2.0 * std::f32::consts::PI * i as f32 / (n as f32 - 1.0)).cos();
            *v *= w;
        }

        let max_bin = (n / 2).max(1);
        let mut best_bin = 0;
        let mut best_mag = 0.0_f32;

        for k in 1..max_bin {
            let mut re = 0.0_f32;
            let mut im = 0.0_f32;
            let omega = 2.0_f32 * std::f32::consts::PI * k as f32 / n as f32;
            for (n_idx, &xn) in x.iter().enumerate() {
                let angle = omega * n_idx as f32;
                re += xn * angle.cos();
                im -= xn * angle.sin();
            }
            let mag = (re * re + im * im).sqrt();
            if mag > best_mag {
                best_mag = mag;
                best_bin = k;
            }
        }

        best_bin as f32 * sample_rate as f32 / n as f32
    }

    #[test]
    fn pyin_then_psola_pipeline_runs_and_preserves_length() {
        let sample_rate = 44100;
        let input_freq = 220.0;
        let duration_s = 0.5;

        let audio = gen_sine(input_freq, sample_rate, duration_s);

        // Run PYIN
        let pyin_result = pyin::pyin(&audio, sample_rate, None, None, None, None, None, None);

        assert!(!pyin_result.f0().is_empty(), "PYIN produced no frames");
        assert_eq!(
            pyin_result.f0().len(),
            pyin_result.voiced_prob().len(),
            "PYIN f0 / voiced_prob length mismatch"
        );
        assert_eq!(
            pyin_result.f0().len(),
            pyin_result.voiced_flag().len(),
            "PYIN f0 / voiced_flag length mismatch"
        );

        // Use original f0 as target to check that PSOLA does not explode / shrink
        let target_f0 = pyin_result.f0().clone();

        let out = psola::psola(&audio, sample_rate, &pyin_result, &target_f0, None, None);

        // Output should be roughly same length (allowing some margin around edges)
        let in_len = audio.len();
        let out_len = out.len();
        let diff = (out_len as isize - in_len as isize).abs() as usize;
        assert!(
            diff < HOP_LENGTH * 2,
            "PSOLA changed length too much: in={} out={} diff={}",
            in_len,
            out_len,
            diff
        );
    }

    #[test]
    fn pyin_then_psola_retunes_frequency_upwards() {
        let sample_rate = 44100;
        let input_freq = 220.0;
        let duration_s = 0.5;

        let audio = gen_sine(input_freq, sample_rate, duration_s);

        // Baseline estimated freq from input
        let base_est_freq = estimate_dominant_freq(&audio, sample_rate);
        assert!(
            (base_est_freq - input_freq).abs() < 10.0,
            "Baseline freq estimate too far from {} Hz: got {}",
            input_freq,
            base_est_freq
        );

        // Run PYIN on input
        let pyin_result = pyin::pyin(&audio, sample_rate, None, None, None, None, None, None);
        assert!(!pyin_result.f0().is_empty(), "PYIN produced no frames");

        // Build a target F0 track that is one octave up (2x)
        let target_f0: Vec<f32> = pyin_result
            .f0()
            .iter()
            .map(|f| if *f > 0.0 { f * 2.0 } else { *f })
            .collect();

        let out = psola::psola(&audio, sample_rate, &pyin_result, &target_f0, None, None);

        // Estimate dominant frequency after retuning
        let out_est_freq = estimate_dominant_freq(&out, sample_rate);

        // Expect something roughly between 1.7x and 2.3x of original,
        // to allow for PYIN + PSOLA inaccuracies.
        let ratio = out_est_freq / base_est_freq.max(1.0);
        assert!(
            ratio > 1.5 && ratio < 2.5,
            "Expected frequency to move about 2x, got ratio {:.2} (in={}Hz, out={}Hz)",
            ratio,
            base_est_freq,
            out_est_freq,
        );
    }
}
