use hound::{WavReader, WavSpec, WavWriter};
use std::io;
use std::path::Path;

pub struct AudioFile {
    samples: Vec<f32>,
    spec: WavSpec,
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

        Ok(AudioFile { samples, spec })
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

    /// Get the audio samples as a slice
    pub fn samples(&self) -> &[f32] {
        &self.samples
    }

    /// Get the audio specification
    pub fn spec(&self) -> WavSpec {
        self.spec
    }
}

