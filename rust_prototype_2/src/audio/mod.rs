pub mod audio_controller;
pub mod autotune;
pub mod file;

#[derive(Clone, Debug)]
pub struct Audio {
    sample_rate: u32,
    length: usize,
    left: Vec<f32>,
    right: Vec<f32>,
}

fn interleave_stereo(left: &[f32], right: &[f32], out: &mut [f32]) {
    for (i, frame) in out.chunks_exact_mut(2).enumerate() {
        frame[0] = left.get(i).copied().unwrap_or(0.0);
        frame[1] = right.get(i).copied().unwrap_or(0.0);
    }
}
