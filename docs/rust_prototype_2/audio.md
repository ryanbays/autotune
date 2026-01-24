**rust_prototype_2 > audio**

# Module: audio

## Contents

**Modules**

- [`audio_controller`](#audio_controller)
- [`autotune`](#autotune)
- [`file`](#file)
- [`scales`](#scales)

**Structs**

- [`Audio`](#audio) - Represents stereo audio data along with associated PYIN analysis.

**Functions**

- [`compute_pyin_blocking`](#compute_pyin_blocking) - Internal helper: runs pyin on left/right on the current thread.
- [`interleave_stereo`](#interleave_stereo) - Helper function to interleave two stereo channels into a single output buffer.

---

## rust_prototype_2::audio::Audio

*Struct*

Represents stereo audio data along with associated PYIN analysis.
Thread-safe access to PYIN data is ensured via RwLock.

**Fields:**
- `sample_rate: u32`
- `length: usize`
- `left: Vec<f32>`
- `right: Vec<f32>`
- `pyin: std::sync::Arc<std::sync::RwLock<Option<crate::audio::autotune::pyin::PYINData>>>`
- `desired_f0: Option<Vec<f32>>`

**Methods:**

- `fn new(sample_rate: u32, left: Vec<f32>, right: Vec<f32>) -> Self`
- `fn sample_rate(self: &Self) -> u32`
- `fn length(self: &Self) -> usize`
- `fn left(self: &Self) -> &[f32]`
- `fn right(self: &Self) -> &[f32]`
- `fn get_pyin(self: &Self) -> Option<PYINData>` - Get a cloned PYIN data (if available) in a thread-safe way.
- `fn get_pyin_blocking(self: &Self) -> Option<PYINData>` - Gets the PYIN data, blocking until it is available.
- `fn pyin_handle(self: &Self) -> Arc<RwLock<Option<PYINData>>>` - Expose shared handle if callers want to manage locking themselves.
- `fn perform_pyin(self: & mut Self)` - Synchronous wrapper.
- `fn perform_pyin_background(self: & mut Self) -> thread::JoinHandle<()>` - Starts PYIN analysis on a background OS thread and returns immediately.
- `fn interleaved(self: &Self) -> Vec<f32>` - Returns interleaved stereo samples as a Vec<f32>
- `fn insert_audio_at(self: & mut Self, position: usize, other: &Audio) -> anyhow::Result<()>` - Inserts the audio from `other` into `self` starting at `position`. (Overwrites existing
- `fn add_audio_at(self: & mut Self, position: usize, other: &Audio) -> anyhow::Result<()>` - Adds the audio from `other` into `self` starting at `position`. (Adds to existing

**Trait Implementations:**

- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`
- **Clone**
  - `fn clone(self: &Self) -> Audio`



## Module: audio_controller



## Module: autotune



## rust_prototype_2::audio::compute_pyin_blocking

*Function*

Internal helper: runs pyin on left/right on the current thread.
(Call this from a background thread to keep the GUI responsive.)

```rust
fn compute_pyin_blocking(sample_rate: u32, left: Vec<f32>, right: Vec<f32>, pyin_ref: std::sync::Arc<std::sync::RwLock<Option<crate::audio::autotune::pyin::PYINData>>>)
```



## Module: file



## rust_prototype_2::audio::interleave_stereo

*Function*

Helper function to interleave two stereo channels into a single output buffer.
Assumes `out` has enough space to hold interleaved samples.

```rust
fn interleave_stereo(left: &[f32], right: &[f32], out: & mut [f32])
```



## Module: scales



