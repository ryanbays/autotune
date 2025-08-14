pub mod clip_manager;
pub mod file;

#[derive(Debug, Clone)]
pub struct AudioClip {
    pub name: String,
    pub sample_rate: u32,
    pub n_samples: usize,
    pub waveform: Vec<f32>, // Normalized waveform data (-1.0 to 1.0)
}
