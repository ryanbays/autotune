use crate::audio::autotune::pyin::{PYinOutput, pyin};
use ndarray::{Array1, s};

pub fn find_pitch_marks(
    y: &Array1<f32>,
    f0: &Array1<f32>,
    voiced_flag: &Array1<bool>,
    sample_rate: u32,
    hop_length: usize,
) -> Vec<usize> {
    let mut pitch_marks = Vec::new();

    // Find the first voiced frame
    if let Some((first_i, &f0_start)) = f0
        .iter()
        .enumerate()
        .find(|(i, f)| voiced_flag[*i] && **f > 0.0)
    {
        let mut current_position = first_i * hop_length;
        let mut current_period = (sample_rate as f32 / f0_start) as usize;

        while current_position < y.len() {
            pitch_marks.push(current_position);

            // Estimate next period from local f0 if available
            let frame_index = current_position / hop_length;
            if frame_index < f0.len() && voiced_flag[frame_index] && f0[frame_index] > 0.0 {
                current_period = (sample_rate as f32 / f0[frame_index]) as usize;
            }

            current_position += current_period.max(1); // Ensure we move forward at least 1 sample
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

    // Precompute Hann window
    let hann: Array1<f32> = Array1::from_iter((0..frame_length).map(|n| {
        (std::f32::consts::PI * 2.0 * n as f32 / (frame_length as f32))
            .sin()
            .powi(2)
    }));

    for &mark in pitch_marks {
        if mark >= half_frame && mark + half_frame < y.len() {
            let mut frame = y.slice(s![mark - half_frame..mark + half_frame]).to_owned();
            frame *= &hann;
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
    if pitch_marks.is_empty() {
        return Vec::new();
    }

    let mut new_pitch_marks = Vec::new();
    let mut current_position = pitch_marks[0] as f32;
    new_pitch_marks.push(current_position as usize);

    for i in 1..pitch_marks.len() {
        let period = (pitch_marks[i] - pitch_marks[i - 1]) as f32;
        let scaling = scaling_factors.get(i).cloned().unwrap_or(1.0);
        let scaled_period = period * scaling;
        current_position += scaled_period;
        new_pitch_marks.push(current_position.round() as usize);
    }

    new_pitch_marks
}

pub fn synthesize(
    frames: &Vec<Array1<f32>>,
    new_pitch_marks: &Vec<usize>,
    output_length: usize,
) -> Array1<f32> {
    let mut output = Array1::<f32>::zeros(output_length);

    if frames.is_empty() {
        return output;
    }

    let frame_length = frames[0].len();
    let half_frame = frame_length / 2;

    // Precompute Hann window (same as in extract_frames)
    let hann: Array1<f32> = Array1::from_iter((0..frame_length).map(|n| {
        (std::f32::consts::PI * 2.0 * n as f32 / (frame_length as f32))
            .sin()
            .powi(2)
    }));

    // For normalization (to fix window overlap sum ≠ 1)
    let mut weight = Array1::<f32>::zeros(output_length);

    for (i, &mark) in new_pitch_marks.iter().enumerate() {
        if i >= frames.len() {
            break;
        }

        let start = if mark >= half_frame {
            mark - half_frame
        } else {
            0
        };
        let end = (start + frame_length).min(output_length);

        if end <= start {
            continue;
        }

        let frame = &frames[i];

        for j in 0..(end - start) {
            output[start + j] += frame[j] * hann[j];
            weight[start + j] += hann[j];
        }
    }

    // Normalize by window weights to avoid amplitude changes
    for i in 0..output.len() {
        if weight[i] > 1e-6 {
            output[i] /= weight[i];
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
    pyin_result: Option<PYinOutput>,
) -> Array1<f32> {
    // Get source pitch using PYIN
    let pyin_output = pyin_result.unwrap_or(pyin(
        y,
        frame_length,
        hop_length,
        sample_rate,
        f_min,
        f_max,
        0.1,
    ));

    // Find pitch marks
    let pitch_marks = find_pitch_marks(
        y,
        &pyin_output.f0,
        &pyin_output.voiced_flag,
        sample_rate,
        hop_length,
    );
    println!("Found {} pitch marks", pitch_marks.len());

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
