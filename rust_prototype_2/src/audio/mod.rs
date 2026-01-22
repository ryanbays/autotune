pub mod audio_controller;
pub mod autotune;
pub mod file;
use crate::audio::autotune::pyin::{self, PYINData};
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

/// Represents stereo audio data along with associated PYIN analysis.
/// Thread-safe access to PYIN data is ensured via RwLock.
#[derive(Clone, Debug)]
pub struct Audio {
    sample_rate: u32,
    length: usize,
    left: Vec<f32>,
    right: Vec<f32>,
    pyin: Arc<RwLock<Option<PYINData>>>, // To ensure thread-safe access
    pub desired_f0: Option<Vec<f32>>,
}

impl Audio {
    pub fn new(sample_rate: u32, left: Vec<f32>, right: Vec<f32>) -> Self {
        assert_eq!(
            left.len(),
            right.len(),
            "Left and right channel lengths must match"
        );
        let length = left.len().max(right.len());
        Self {
            sample_rate,
            length,
            left,
            right,
            desired_f0: None,
            pyin: Arc::new(RwLock::new(None)),
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn left(&self) -> &[f32] {
        &self.left
    }

    pub fn right(&self) -> &[f32] {
        &self.right
    }

    /// Get a cloned PYIN data (if available) in a thread-safe way.
    pub fn get_pyin(&self) -> Option<PYINData> {
        self.pyin.read().ok().and_then(|g| g.clone())
    }

    /// Gets the PYIN data, blocking until it is available.
    pub fn get_pyin_blocking(&self) -> Option<PYINData> {
        use std::thread;
        use std::time::Duration;

        loop {
            match self.pyin.read() {
                Ok(guard) => {
                    if let Some(data) = guard.clone() {
                        return Some(data);
                    }
                }
                Err(e) => {
                    info!("Failed to acquire PYIN read lock: {:?}", e);
                    return None;
                }
            }
            // Avoid busy-waiting
            thread::sleep(Duration::from_millis(10));
        }
    }

    /// Expose shared handle if callers want to manage locking themselves.
    /// NOTE: Might be an unnecessary function.
    pub fn pyin_handle(&self) -> Arc<RwLock<Option<PYINData>>> {
        Arc::clone(&self.pyin)
    }

    /// Synchronous wrapper that blocks on the async implementation.
    pub fn perform_pyin(&mut self) {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(self.perform_pyin_async());
    }

    /// Asynchronously performs PYIN analysis on both channels and stores the result.
    pub async fn perform_pyin_async(&mut self) {
        debug!("Async perform PYIN called");
        let stereo = self.compute_pyin_async().await;
        if let Ok(mut guard) = self.pyin.write() {
            *guard = Some(stereo);
        }
    }

    /// Internal helper: runs pyin on left/right in a blocking thread.
    async fn compute_pyin_async(&self) -> PYINData {
        let left = self.left.clone();
        let right = self.right.clone();
        let sample_rate = self.sample_rate;

        tokio::task::spawn_blocking(move || {
            debug!("Starting PYIN analysis for both channels asynchronously");
            let (left_pyin, right_pyin) = rayon::join(
                || pyin::pyin(&left, sample_rate, None, None, None, None, None, None),
                || pyin::pyin(&right, sample_rate, None, None, None, None, None, None),
            );
            debug!(
                right_len = right_pyin.f0().len(),
                left_len = left_pyin.f0().len(),
                "Completed PYIN analysis for both channels"
            );
            // Combine results based on highest probabilty of voicing
            let length = left_pyin.f0().len().max(right_pyin.f0().len());
            let mut f0 = vec![0.0; length];
            let mut voiced_flags = vec![false; length];
            let mut prob = vec![0.0; length];
            for i in 0..length {
                let left_prob = left_pyin.voiced_prob().get(i).copied().unwrap_or(0.0);
                let right_prob = right_pyin.voiced_prob().get(i).copied().unwrap_or(0.0);
                if left_prob >= right_prob {
                    f0[i] = left_pyin.f0().get(i).copied().unwrap_or(0.0);
                    voiced_flags[i] = left_pyin.voiced_flag().get(i).copied().unwrap_or(false);
                    prob[i] = left_prob;
                } else {
                    f0[i] = right_pyin.f0().get(i).copied().unwrap_or(0.0);
                    voiced_flags[i] = right_pyin.voiced_flag().get(i).copied().unwrap_or(false);
                    prob[i] = right_prob;
                }
            }
            debug!("Combined PYIN data from both channels");
            PYINData::new(f0, voiced_flags, prob)
        })
        .await
        .expect("spawn_blocking panicked")
    }

    /// Returns interleaved stereo samples as a Vec<f32>
    pub fn interleaved(&self) -> Vec<f32> {
        let mut out = vec![0.0; self.length * 2];
        interleave_stereo(&self.left, &self.right, &mut out);
        out
    }

    /// Inserts the audio from `other` into `self` starting at `position`. (Overwrites existing
    /// samples)
    /// If `other` extends beyond the current length of `self`, `self` is resized accordingly.
    /// Returns an error if the sample rates do not match.
    pub fn insert_audio_at(&mut self, position: usize, other: &Audio) -> anyhow::Result<()> {
        debug!(
            position,
            other_length = other.length(),
            self_length = self.length,
            "Inserting audio at position"
        );
        if self.sample_rate != other.sample_rate {
            anyhow::bail!("Sample rates must match to insert audio");
        }

        let end_position = position + other.length();
        if end_position > self.length {
            self.left.resize(end_position, 0.0);
            self.right.resize(end_position, 0.0);
            self.length = end_position;
        }
        for i in 0..other.length() {
            self.left[position + i] = other.left().get(i).copied().unwrap_or(0.0);
            self.right[position + i] = other.right().get(i).copied().unwrap_or(0.0);
        }
        debug!(self_length = self.length, "Completed audio insertion");
        Ok(())
    }
    /// Adds the audio from `other` into `self` starting at `position`. (Adds to existing
    /// samples)
    /// If `other` extends beyond the current length of `self`, `self` is resized accordingly.
    /// Returns an error if the sample rates do not match.
    pub fn add_audio_at(&mut self, position: usize, other: &Audio) -> anyhow::Result<()> {
        debug!(
            position,
            other_length = other.length(),
            self_length = self.length,
            "Adding audio at position"
        );
        if self.sample_rate != other.sample_rate {
            anyhow::bail!("Sample rates must match to add audio");
        }
        let end_position = position + other.length();
        if end_position > self.length {
            self.left.resize(end_position, 0.0);
            self.right.resize(end_position, 0.0);
            self.length = end_position;
        }
        for i in 0..other.length() {
            self.left[position + i] += other.left().get(i).copied().unwrap_or(0.0);
            self.right[position + i] += other.right().get(i).copied().unwrap_or(0.0);
        }
        debug!(self_length = self.length, "Completed audio addition");
        Ok(())
    }
}

/// Helper function to interleave two stereo channels into a single output buffer.
/// Assumes `out` has enough space to hold interleaved samples.
fn interleave_stereo(left: &[f32], right: &[f32], out: &mut [f32]) {
    for (i, frame) in out.chunks_exact_mut(2).enumerate() {
        frame[0] = left.get(i).copied().unwrap_or(0.0);
        frame[1] = right.get(i).copied().unwrap_or(0.0);
    }
}
