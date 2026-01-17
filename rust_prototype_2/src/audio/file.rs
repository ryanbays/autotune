use crate::audio::Audio;
use anyhow::{Result, anyhow};
use cpal::Sample;
use hound::{WavSpec, WavWriter};
use rodio::{Decoder, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Audio file with interleaved samples:
/// layout = [ch0_f0, ch1_f0, ..., ch{n-1}_f0, ch0_f1, ch1_f1, ...]
pub struct AudioFileData {
    samples: Vec<f32>,
    n_samples: usize,
    sample_rate: u32,
    n_channels: usize,
}

impl AudioFileData {
    /// Uses rodio::Decoder, which yields interleaved samples for multichannel audio.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(&path)?;
        let source = Decoder::new(BufReader::new(file))?;

        let sample_rate = source.sample_rate();
        let n_channels = source.channels() as usize;

        let samples: Vec<f32> = source.map(Sample::to_sample::<f32>).collect();

        if n_channels == 0 {
            return Err(anyhow!(
                "Decoder reported 0 channels for file {:?}",
                path.as_ref()
            ));
        }

        if samples.len() % n_channels != 0 {
            return Err(anyhow!(
                "Sample count {} is not divisible by channel count {} for file {:?}",
                samples.len(),
                n_channels,
                path.as_ref()
            ));
        }

        let n_samples = samples.len() / n_channels;

        Ok(AudioFileData {
            samples,
            sample_rate,
            n_samples,
            n_channels,
        })
    }

    // Construct from already-interleaved samples.
    // NOTE: This function could be unnecessary if we always use `load` to read from files.
    // `samples` layout must be: [ch0_f0, ch1_f0, ..., ch{n-1}_f0, ch0_f1, ...].
    pub fn new(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Result<Self> {
        if channels == 0 {
            return Err(anyhow!("channels must be > 0"));
        }
        if samples.len() % channels as usize != 0 {
            return Err(anyhow!(
                "samples length {} is not divisible by channels {}",
                samples.len(),
                channels
            ));
        }

        let n_channels = channels as usize;
        let n_samples = samples.len() / n_channels;

        Ok(AudioFileData {
            samples,
            sample_rate,
            n_channels,
            n_samples,
        })
    }

    /// Save audio data to a WAV file (16-bit PCM, interleaved channels).
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let extension = path.as_ref().extension().and_then(|s| s.to_str());
        match extension {
            Some("wav") => {
                let spec = WavSpec {
                    channels: self.n_channels as u16,
                    sample_rate: self.sample_rate,
                    bits_per_sample: 16,
                    sample_format: hound::SampleFormat::Int,
                };

                let mut writer = WavWriter::create(path, spec)?;

                for &sample in &self.samples {
                    // Clamp to [-1.0, 1.0] before scaling to i16
                    let clamped = sample.clamp(-1.0, 1.0);
                    let int_sample = (clamped * i16::MAX as f32) as i16;
                    writer.write_sample(int_sample)?;
                }

                writer.finalize()?;
                Ok(())
            }
            _ => Err(anyhow!("Unsupported file format; only .wav is supported.")),
        }
    }
    pub fn to_audio(&self) -> Audio {
        if self.n_channels == 1 {
            return Audio::new(self.sample_rate, self.samples.clone(), self.samples.clone());
        }
        let mut left = Vec::with_capacity(self.n_samples);
        let mut right = Vec::with_capacity(self.n_samples);

        for frame in 0..self.n_samples {
            left.push(self.samples[frame * self.n_channels]);
            right.push(self.samples[frame * self.n_channels + 1]);
        }
        Audio::new(self.sample_rate, left, right)
    }

    pub fn n_channels(&self) -> usize {
        self.n_channels
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn n_samples(&self) -> usize {
        self.n_samples
    }

    /// Total number of interleaved sample values (frames * channels).
    pub fn len_samples_raw(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn samples(&self) -> &[f32] {
        &self.samples
    }
}
