use crate::audio::autotune::pyin::{PYinResult, pyin};
use anyhow::Result;
use hound::{WavSpec, WavWriter};
use rodio::{Decoder, Source};
use std::io;
use std::path::Path;

pub struct AudioFile {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
    pyin_result: Option<PYinResult>,
}

impl AudioFile {
    /// Load an audio file from the given path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let source = Decoder::new(io::BufReader::new(file))?;
        let sample_rate = source.sample_rate();
        let channels = source.channels();
        let samples: Vec<f32> = source.collect();

        Ok(AudioFile {
            samples,
            sample_rate,
            channels,
            pyin_result: None,
        })
    }
    pub fn new(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        AudioFile {
            samples,
            sample_rate,
            channels,
            pyin_result: None,
        }
    }
    /// Save audio data to a WAV file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let extension = path.as_ref().extension().and_then(|s| s.to_str());
        match extension {
            Some("wav") => {
                let spec = WavSpec {
                    channels: self.channels,
                    sample_rate: self.sample_rate,
                    bits_per_sample: 16,
                    sample_format: hound::SampleFormat::Int,
                };
                let mut writer = WavWriter::create(path, spec)?;
                for &sample in &self.samples {
                    let int_sample = (sample * i16::MAX as f32) as i16;
                    writer.write_sample(int_sample)?;
                }
                writer.finalize()?;
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Unsupported file format.")),
        }
    }

    /// Run PYin pitch detection on the audio samples
    pub fn run_pyin(
        &mut self,
        frame_length: usize,
        hop_length: usize,
        f_min: f32,
        f_max: f32,
        threshold: f32,
    ) -> Result<()> {
        let result = pyin(
            &ndarray::Array1::from_vec(self.samples.clone()),
            frame_length,
            hop_length,
            self.sample_rate,
            f_min,
            f_max,
            threshold,
        );
        self.pyin_result = Some(result);
        Ok(())
    }

    pub fn get_pyin_result(&mut self) -> &PYinResult {
        println!("{}", self.pyin_result.is_none());
        if self.pyin_result.is_none() {
            println!("Running pyin");
            self.run_pyin(2048, 256, 0.1, 0.2, 0.05);
        }
        self.pyin_result.as_ref().unwrap()
    }
    /// Get the number of channels in the audio
    pub fn get_channels(&self) -> u16 {
        self.channels
    }

    /// Get the audio samples as a slice
    pub fn get_samples(&self) -> &[f32] {
        &self.samples
    }

    /// Get the sample rate of the audio
    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get the number of samples in the audio
    pub fn get_n_samples(&self) -> usize {
        self.samples.len()
    }
}
