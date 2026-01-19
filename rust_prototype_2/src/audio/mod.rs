pub mod audio_controller;
pub mod autotune;
pub mod file;
use crate::audio::autotune::pyin::{self, PYINData};
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

#[derive(Clone, Debug)]
pub struct Audio {
    sample_rate: u32,
    length: usize,
    left: Vec<f32>,
    right: Vec<f32>,
    pyin: Arc<RwLock<Option<PYINData>>>, // To ensure thread-safe access
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

    pub fn interleaved(&self) -> Vec<f32> {
        let mut out = vec![0.0; self.length * 2];
        interleave_stereo(&self.left, &self.right, &mut out);
        out
    }
}

fn interleave_stereo(left: &[f32], right: &[f32], out: &mut [f32]) {
    for (i, frame) in out.chunks_exact_mut(2).enumerate() {
        frame[0] = left.get(i).copied().unwrap_or(0.0);
        frame[1] = right.get(i).copied().unwrap_or(0.0);
    }
}
