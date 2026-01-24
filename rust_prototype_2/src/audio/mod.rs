pub mod audio_controller;
pub mod autotune;
pub mod file;

use crate::audio::autotune::pyin::{self, PYINData};
use std::sync::{Arc, RwLock};
use std::thread;
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
    #[allow(dead_code)]
    pub fn get_pyin_blocking(&self) -> Option<PYINData> {
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

    /// Synchronous wrapper.
    /// NOTE: Do NOT call this on the GUI thread; prefer `perform_pyin_background`.
    pub fn perform_pyin(&mut self) {
        compute_pyin_blocking(
            self.sample_rate,
            self.left.clone(),
            self.right.clone(),
            self.pyin_handle(),
        );
    }

    /// Starts PYIN analysis on a background OS thread and returns immediately.
    /// Store the JoinHandle if you want (optional). If you drop it, it still runs.
    pub fn perform_pyin_background(&mut self) -> thread::JoinHandle<()> {
        let left = self.left.clone();
        let right = self.right.clone();
        let sample_rate = self.sample_rate;
        let pyin_ref = self.pyin_handle();

        thread::spawn(move || {
            compute_pyin_blocking(sample_rate, left, right, pyin_ref);
        })
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

/// Internal helper: runs pyin on left/right on the current thread.
/// (Call this from a background thread to keep the GUI responsive.)
fn compute_pyin_blocking(
    sample_rate: u32,
    left: Vec<f32>,
    right: Vec<f32>,
    pyin_ref: Arc<RwLock<Option<PYINData>>>,
) {
    debug!("Starting PYIN analysis for both channels (background thread)");
    let start_time = std::time::Instant::now();
    let (left_pyin, right_pyin) = rayon::join(
        || pyin::pyin(&left, sample_rate, None, None, None, None, None, None),
        || pyin::pyin(&right, sample_rate, None, None, None, None, None, None),
    );

    debug!(
        right_len = right_pyin.f0().len(),
        left_len = left_pyin.f0().len(),
        "Completed PYIN analysis for both channels"
    );

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
    let elapsed = start_time.elapsed();
    debug!(time = ?elapsed, "Combined PYIN data from both channels");

    match pyin_ref.write() {
        Ok(mut guard) => {
            *guard = Some(PYINData::new(f0, voiced_flags, prob));
        }
        Err(e) => {
            info!("Failed to acquire PYIN write lock: {:?}", e);
        }
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
