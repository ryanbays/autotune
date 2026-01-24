**rust_prototype_1 > audio > autotune > pyin**

# Module: audio::autotune::pyin

## Contents

**Structs**

- [`PYinResult`](#pyinresult)

**Functions**

- [`cumulative_mean_normalized_difference`](#cumulative_mean_normalized_difference)
- [`difference_function`](#difference_function)
- [`get_pitch_period_candidates`](#get_pitch_period_candidates)
- [`pyin`](#pyin)

---

## rust_prototype_1::audio::autotune::pyin::PYinResult

*Struct*

**Fields:**
- `f0: ndarray::Array1<f32>`
- `voiced_flag: ndarray::Array1<bool>`
- `voiced_prob: ndarray::Array1<f32>`

**Methods:**

- `fn snap_to_scale(self: &Self, key: Key) -> Array1<f32>`

**Trait Implementations:**

- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`
- **Clone**
  - `fn clone(self: &Self) -> PYinResult`



## rust_prototype_1::audio::autotune::pyin::cumulative_mean_normalized_difference

*Function*

```rust
fn cumulative_mean_normalized_difference(diff: &ndarray::Array1<f32>) -> ndarray::Array1<f32>
```



## rust_prototype_1::audio::autotune::pyin::difference_function

*Function*

```rust
fn difference_function(frame: &ndarray::ArrayView1<f32>) -> ndarray::Array1<f32>
```



## rust_prototype_1::audio::autotune::pyin::get_pitch_period_candidates

*Function*

```rust
fn get_pitch_period_candidates(cmnd: &ndarray::Array1<f32>, threshold: f32) -> Vec<usize>
```



## rust_prototype_1::audio::autotune::pyin::pyin

*Function*

```rust
fn pyin(y: &ndarray::Array1<f32>, frame_length: usize, hop_length: usize, sample_rate: u32, f_min: f32, f_max: f32, threshold: f32) -> PYinResult
```



