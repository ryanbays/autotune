**autotune > audio > autotune > pyin**

# Module: audio::autotune::pyin

## Contents

**Structs**

- [`PYINData`](#pyindata)

**Functions**

- [`cumulative_mean_normalized_difference`](#cumulative_mean_normalized_difference)
- [`difference_function`](#difference_function)
- [`find_pitch_candidates`](#find_pitch_candidates)
- [`frame_rms`](#frame_rms) - Simple RMS energy of a frame, used for voicing / silence detection.
- [`parabolic_interp`](#parabolic_interp)
- [`probabilistic_f0_selection`](#probabilistic_f0_selection)
- [`pyin`](#pyin)

---

## autotune::audio::autotune::pyin::PYINData

*Struct*

**Fields:**
- `f0: Vec<f32>`
- `voiced_flag: Vec<bool>`
- `voiced_prob: Vec<f32>`

**Methods:**

- `fn new(f0: Vec<f32>, voiced_flag: Vec<bool>, voiced_prob: Vec<f32>) -> Self`
- `fn f0(self: &Self) -> &Vec<f32>`
- `fn voiced_flag(self: &Self) -> &Vec<bool>`
- `fn voiced_prob(self: &Self) -> &Vec<f32>`

**Trait Implementations:**

- **Clone**
  - `fn clone(self: &Self) -> PYINData`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`



## autotune::audio::autotune::pyin::cumulative_mean_normalized_difference

*Function*

```rust
fn cumulative_mean_normalized_difference(d: &[f32], max_lag: usize) -> Vec<f32>
```



## autotune::audio::autotune::pyin::difference_function

*Function*

```rust
fn difference_function(frame: &[f32], max_lag: usize) -> Vec<f32>
```



## autotune::audio::autotune::pyin::find_pitch_candidates

*Function*

```rust
fn find_pitch_candidates(cmnd: &[f32], threshold: f32, min_lag: usize, max_lag: usize, sample_rate: u32) -> (Vec<f32>, Vec<f32>)
```



## autotune::audio::autotune::pyin::frame_rms

*Function*

Simple RMS energy of a frame, used for voicing / silence detection.

```rust
fn frame_rms(frame: &[f32]) -> f32
```



## autotune::audio::autotune::pyin::parabolic_interp

*Function*

```rust
fn parabolic_interp(cmnd: &[f32], tau: usize) -> f32
```



## autotune::audio::autotune::pyin::probabilistic_f0_selection

*Function*

```rust
fn probabilistic_f0_selection(f0_candidates: &[f32], candidate_probs: &[f32], sigma: f32, previous_f0: Option<f32>) -> (f32, bool, f32)
```



## autotune::audio::autotune::pyin::pyin

*Function*

```rust
fn pyin(signal: &[f32], sample_rate: u32, frame_length: Option<usize>, hop_length: Option<usize>, fmin: Option<f32>, fmax: Option<f32>, threshold: Option<f32>, sigma: Option<f32>) -> PYINData
```



