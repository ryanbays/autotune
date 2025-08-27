use crate::audio::autotune::pyin::pyin;
use ndarray::{Array1, s};

pub fn find_pitch_marks(
    y: &Array1<f32>,
    f0: &Array1<f32>,
    voiced_flag: &Array1<bool>,
    sample_rate: u32,
    hop_length: usize,
) -> Vec<usize> {
    let mut pitch_marks = Vec::new();
    let mut current_position = 0;

    for (i, &f0_val) in f0.iter().enumerate() {
        if voiced_flag[i] && f0_val > 0.0 {
            let period = (sample_rate as f32 / f0_val) as usize;
            current_position += hop_length;
            while current_position < y.len() {
                pitch_marks.push(current_position);
                current_position += period;
            }
        } else {
            current_position += hop_length;
        }
    }

    pitch_marks
}

pub fn extract_frames(
    y: &Array1<f32>,
    pitch_marks: &Vec<usize>,
    frame_length: usize,
) -> Vec<Array1<f32>> {
    let mut frames = Vec::new();
    let half_frame = frame_length / 2;

    for &mark in pitch_marks {
        if mark >= half_frame && mark + half_frame < y.len() {
            let frame = y.slice(s![mark - half_frame..mark + half_frame]).to_owned();
            frames.push(frame);
        }
    }

    frames
}
pub fn compute_scaling_factors(
    source_f0: &Array1<f32>,
    target_f0: &Array1<f32>,
    voiced_flag: &Array1<bool>,
) -> Vec<f32> {
    let mut scaling_factors = Vec::new();

    for (i, &f0_val) in source_f0.iter().enumerate() {
        if voiced_flag[i] && f0_val > 0.0 && target_f0[i] > 0.0 {
            scaling_factors.push(target_f0[i] / f0_val);
        } else {
            scaling_factors.push(1.0);
        }
    }

    scaling_factors
}

pub fn generate_new_pitch_marks(
    pitch_marks: &Vec<usize>,
    scaling_factors: &Vec<f32>,
) -> Vec<usize> {
    let mut new_pitch_marks = Vec::new();
    let mut current_position = 0.0;

    for (i, &mark) in pitch_marks.iter().enumerate() {
        let scaling = if i < scaling_factors.len() {
            scaling_factors[i]
        } else {
            1.0
        };
        current_position += (mark as f32) * scaling;
        new_pitch_marks.push(current_position as usize);
    }

    new_pitch_marks
}

pub fn synthesize(
    frames: &Vec<Array1<f32>>,
    new_pitch_marks: &Vec<usize>,
    output_length: usize,
) -> Array1<f32> {
    let mut output = Array1::<f32>::zeros(output_length);
    let frame_length = if !frames.is_empty() {
        frames[0].len()
    } else {
        0
    };
    let half_frame = frame_length / 2;

    for (i, &mark) in new_pitch_marks.iter().enumerate() {
        if i < frames.len() {
            let start = if mark >= half_frame {
                mark - half_frame
            } else {
                0
            };
            let end = if start + frame_length < output_length {
                start + frame_length
            } else {
                output_length
            };
            let frame = &frames[i];

            for j in 0..(end - start) {
                output[start + j] += frame[j];
            }
        }
    }

    output
}

pub fn psola(
    y: &Array1<f32>,
    target_f0: &Array1<f32>,
    sample_rate: u32,
    frame_length: usize,
    hop_length: usize,
    f_min: f32,
    f_max: f32,
) -> Array1<f32> {
    // Get source pitch using PYIN
    let pyin_output = pyin(y, frame_length, hop_length, sample_rate, f_min, f_max, 0.1);

    // Find pitch marks
    let pitch_marks = find_pitch_marks(
        y,
        &pyin_output.f0,
        &pyin_output.voiced_flag,
        sample_rate,
        hop_length,
    );

    // Extract frames
    let frames = extract_frames(y, &pitch_marks, frame_length);

    // Compute scaling factors
    let scaling_factors =
        compute_scaling_factors(&pyin_output.f0, target_f0, &pyin_output.voiced_flag);

    // Generate new pitch marks
    let new_pitch_marks = generate_new_pitch_marks(&pitch_marks, &scaling_factors);

    // Synthesize output
    synthesize(&frames, &new_pitch_marks, y.len())
}
