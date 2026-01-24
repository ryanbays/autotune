**rust_prototype_1 > audio > autotune > psola**

# Module: audio::autotune::psola

## Contents

**Functions**

- [`compute_scaling_factors`](#compute_scaling_factors)
- [`extract_frames`](#extract_frames)
- [`find_pitch_marks`](#find_pitch_marks)
- [`generate_new_pitch_marks`](#generate_new_pitch_marks)
- [`psola`](#psola)
- [`synthesize`](#synthesize)

---

## rust_prototype_1::audio::autotune::psola::compute_scaling_factors

*Function*

```rust
fn compute_scaling_factors(source_f0: &ndarray::Array1<f32>, target_f0: &ndarray::Array1<f32>, voiced_flag: &ndarray::Array1<bool>) -> Vec<f32>
```



## rust_prototype_1::audio::autotune::psola::extract_frames

*Function*

```rust
fn extract_frames(y: &ndarray::Array1<f32>, pitch_marks: &Vec<usize>, frame_length: usize) -> Vec<ndarray::Array1<f32>>
```



## rust_prototype_1::audio::autotune::psola::find_pitch_marks

*Function*

```rust
fn find_pitch_marks(y: &ndarray::Array1<f32>, f0: &ndarray::Array1<f32>, voiced_flag: &ndarray::Array1<bool>, sample_rate: u32, hop_length: usize) -> Vec<usize>
```



## rust_prototype_1::audio::autotune::psola::generate_new_pitch_marks

*Function*

```rust
fn generate_new_pitch_marks(pitch_marks: &Vec<usize>, scaling_factors: &Vec<f32>) -> Vec<usize>
```



## rust_prototype_1::audio::autotune::psola::psola

*Function*

```rust
fn psola(y: &ndarray::Array1<f32>, target_f0: &ndarray::Array1<f32>, sample_rate: u32, frame_length: usize, hop_length: usize, f_min: f32, f_max: f32, pyin_result: Option<crate::audio::autotune::pyin::PYinResult>) -> ndarray::Array1<f32>
```



## rust_prototype_1::audio::autotune::psola::synthesize

*Function*

```rust
fn synthesize(frames: &Vec<ndarray::Array1<f32>>, new_pitch_marks: &Vec<usize>, output_length: usize) -> ndarray::Array1<f32>
```



