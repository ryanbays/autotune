use crate::audio::autotune::{FRAME_LENGTH, HOP_LENGTH, pyin::PYINData};
use tracing::debug;

fn find_pitch_marks(pyin: &PYINData, sample_rate: u32) -> Vec<usize> {
    let mut pitch_marks = Vec::new();
    let mut pos = 0.0_f32;

    for i in 0..pyin.f0().len() {
        if !pyin.voiced_flag()[i] || pyin.f0()[i] <= 0.0 {
            continue;
        }
        let period = sample_rate as f32 / pyin.f0()[i];
        let frame_start = i * HOP_LENGTH;

        if pos < frame_start as f32 {
            pos = frame_start as f32;
        }

        while pos < (frame_start + FRAME_LENGTH) as f32 {
            pitch_marks.push(pos.round() as usize);
            pos += period;
        }
    }

    pitch_marks
}

fn compute_target_pitch_spacing(
    pyin_result: &PYINData,
    target_f0: &Vec<f32>,
    pitch_marks: &Vec<usize>,
) -> Vec<usize> {
    let mut shifted_marks = Vec::new();
    if pitch_marks.is_empty() {
        return shifted_marks;
    }

    shifted_marks.push(pitch_marks[0]);

    for i in 1..pitch_marks.len() {
        let frame_index =
            (pitch_marks[i] / HOP_LENGTH).min(pyin_result.f0().len().saturating_sub(1));
        if frame_index >= pyin_result.f0().len() {
            break;
        }

        if !pyin_result.voiced_flag()[frame_index] || pyin_result.f0()[frame_index] <= 0.0 {
            shifted_marks.push(shifted_marks[i - 1] + (pitch_marks[i] - pitch_marks[i - 1]));
            continue;
        }

        let old_spacing = pitch_marks[i] - pitch_marks[i - 1];
        let ratio = target_f0[frame_index] / pyin_result.f0()[frame_index];
        let new_spacing = (old_spacing as f32 * ratio).max(1.0); // avoid zero spacing
        shifted_marks.push(shifted_marks[i - 1] + new_spacing as usize);
    }

    shifted_marks
}

fn overlap_add(
    audio: &Vec<f32>,
    pitch_marks: &Vec<usize>,
    shifted_marks: &Vec<usize>,
    frame_size: usize,
) -> Vec<f32> {
    if pitch_marks.is_empty() || shifted_marks.is_empty() {
        return Vec::new();
    }

    let output_length = (*shifted_marks.last().unwrap() + frame_size).min(audio.len() * 2);
    let mut output = vec![0.0; output_length];
    let half_frame = frame_size / 2;

    // Hann window
    let window: Vec<f32> = (0..frame_size)
        .map(|n| {
            let x = std::f32::consts::PI * 2.0 * n as f32 / (frame_size as f32 - 1.0);
            0.5 * (1.0 - x.cos())
        })
        .collect();

    for i in 0..pitch_marks.len().min(shifted_marks.len()) {
        let orig_pos = pitch_marks[i];
        let new_pos = shifted_marks[i];

        let start_orig = orig_pos.saturating_sub(half_frame);
        let end_orig = (orig_pos + half_frame).min(audio.len());
        let start_new = new_pos.saturating_sub(half_frame);
        let end_new = (new_pos + half_frame).min(output.len());

        let max_len_orig = end_orig.saturating_sub(start_orig);
        let max_len_new = end_new.saturating_sub(start_new);
        let len = max_len_orig.min(max_len_new);

        if len == 0 {
            continue;
        }

        let win_start = half_frame.saturating_sub(orig_pos.saturating_sub(start_orig));
        for j in 0..len {
            let w = window[win_start + j];
            output[start_new + j] += audio[start_orig + j] * w;
        }
    }

    output
}

pub fn psola(
    audio: &Vec<f32>,
    sample_rate: u32,
    pyin_result: &PYINData,
    target_f0: &Vec<f32>,
    frame_size: Option<usize>,
    hop_size: Option<usize>,
) -> Vec<f32> {
    let frame_size = frame_size.unwrap_or(FRAME_LENGTH);
    let hop_size = hop_size.unwrap_or(HOP_LENGTH);
    debug!(
        frame_size,
        hop_size,
        n_samples = audio.len(),
        "Starting PSOLA pitch shifting"
    );

    if audio.is_empty() || pyin_result.f0().is_empty() || target_f0.is_empty() {
        return Vec::new();
    }

    let pitch_marks = find_pitch_marks(pyin_result, sample_rate);
    let shifted_marks = compute_target_pitch_spacing(pyin_result, target_f0, &pitch_marks);
    let output = overlap_add(audio, &pitch_marks, &shifted_marks, frame_size);

    debug!(n_samples = output.len(), "Completed PSOLA pitch shifting");
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyPYIN {
        f0: Vec<f32>,
        voiced_flag: Vec<bool>,
        voiced_prob: Vec<f32>,
    }

    impl DummyPYIN {
        fn new(f0: Vec<f32>, voiced_flag: Vec<bool>) -> Self {
            let voiced_prob = voiced_flag
                .iter()
                .map(|&v| if v { 1.0 } else { 0.0 })
                .collect();
            Self {
                f0,
                voiced_flag,
                voiced_prob,
            }
        }
    }

    impl DummyPYIN {
        fn as_pyin_data(&self) -> PYINData {
            PYINData::new(
                self.f0.clone(),
                self.voiced_flag.clone(),
                self.voiced_prob.clone(),
            )
        }
    }

    #[test]
    fn test_find_pitch_marks_basic_monotone() {
        let sample_rate = 1000;
        let f0 = vec![100.0; 5]; // 100 Hz
        let voiced_flag = vec![true; 5];
        let pyin = DummyPYIN::new(f0, voiced_flag).as_pyin_data();

        let marks = find_pitch_marks(&pyin, sample_rate);
        assert!(!marks.is_empty());

        let period = (sample_rate as f32 / 100.0).round() as usize;
        for pair in marks.windows(2) {
            assert!((pair[1] - pair[0]) as isize - period as isize <= 1);
        }
    }

    #[test]
    fn test_compute_target_pitch_spacing_identity_when_same_f0() {
        let f0 = vec![100.0; 4];
        let voiced_flag = vec![true; 4];
        let pyin = DummyPYIN::new(f0.clone(), voiced_flag).as_pyin_data();

        let pitch_marks = vec![0, 100, 200, 300];
        let target_f0 = f0;

        let shifted = compute_target_pitch_spacing(&pyin, &target_f0, &pitch_marks);
        assert_eq!(shifted, pitch_marks);
    }

    #[test]
    fn test_compute_target_pitch_spacing_changes_spacing_with_pitch_shift() {
        let f0 = vec![100.0; 4];
        let voiced_flag = vec![true; 4];
        let pyin = DummyPYIN::new(f0.clone(), voiced_flag).as_pyin_data();

        let pitch_marks = vec![0, 100, 200, 300];
        // Double the pitch
        let target_f0 = vec![200.0; 4];

        let shifted = compute_target_pitch_spacing(&pyin, &target_f0, &pitch_marks);
        assert_eq!(shifted.len(), pitch_marks.len());
        // Spacing should be roughly halved between marks
        assert!(shifted[1] - shifted[0] < pitch_marks[1] - pitch_marks[0]);
    }

    #[test]
    fn test_overlap_add_no_panics_and_nonzero_output() {
        let audio: Vec<f32> = (0..200).map(|x| x as f32).collect();
        let pitch_marks = vec![40, 80, 120, 160];
        let shifted_marks = pitch_marks.clone();
        let frame_size = 32;

        let out = overlap_add(&audio, &pitch_marks, &shifted_marks, frame_size);
        assert!(!out.is_empty());
        // Hann windowing should produce non-zero energy near marks
        for &pm in &pitch_marks {
            assert!(pm < out.len());
            assert!(out[pm] != 0.0);
        }
    }

    #[test]
    fn test_psola_handles_empty_inputs() {
        let audio = Vec::new();
        let pyin = DummyPYIN::new(vec![], vec![]).as_pyin_data();
        let target_f0 = Vec::new();

        let out = psola(&audio, 44100, &pyin, &target_f0, None, None);
        assert!(out.is_empty());
    }

    #[test]
    fn test_psola_runs_with_simple_constant_pitch() {
        let audio: Vec<f32> = (0..(FRAME_LENGTH * 4)).map(|x| (x as f32).sin()).collect();
        let f0 = vec![100.0; 10];
        let voiced_flag = vec![true; 10];
        let pyin = DummyPYIN::new(f0.clone(), voiced_flag).as_pyin_data();
        let target_f0 = f0;

        let out = psola(&audio, 44100, &pyin, &target_f0, None, None);
        assert!(!out.is_empty());
    }
}
