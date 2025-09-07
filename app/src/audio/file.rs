use crate::audio::autotune::pyin::{PYinOutput, pyin};
use hound::{WavReader, WavSpec, WavWriter};
use std::io;
use std::path::Path;

pub struct AudioFile {
    samples: Vec<f32>,
    spec: WavSpec,
    pyin_result: Option<PYinOutput>,
}

impl AudioFile {
    /// Load an audio file from the given path
    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut reader =
            WavReader::open(path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let spec = reader.spec();

        let samples: Vec<f32> = reader
            .samples()
            .map(|sample| sample.unwrap_or(0) as f32 / i16::MAX as f32)
            .collect();

        Ok(AudioFile {
            samples,
            spec,
            pyin_result: None,
        })
    }

    /// Save audio data to a WAV file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut writer = WavWriter::create(path, self.spec)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        for sample in &self.samples {
            let value = (sample * i16::MAX as f32) as i16;
            writer
                .write_sample(value)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }

        writer
            .finalize()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(())
    }

    pub fn new(samples: Vec<f32>, spec: WavSpec) -> Self {
        AudioFile {
            samples,
            spec,
            pyin_result: None,
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
    ) {
        let result = pyin(
            &ndarray::Array1::from_vec(self.samples.clone()),
            frame_length,
            hop_length,
            self.spec.sample_rate,
            f_min,
            f_max,
            threshold,
        );
        self.pyin_result = Some(result);
    }

    pub fn get_pyin_result(&mut self) -> &PYinOutput {
        if self.pyin_result.is_none() {
            self.run_pyin(2048, 256, 0.1, 0.2, 0.05);
        }
        self.pyin_result.as_ref().unwrap()
    }

    /// Get the audio samples as a slice
    pub fn get_samples(&self) -> &[f32] {
        &self.samples
    }

    /// Get the audio specification
    pub fn get_spec(&self) -> WavSpec {
        self.spec
    }
}
