**autotune > audio > autotune > psola**

# Module: audio::autotune::psola

## Contents

**Functions**

- [`compute_target_pitch_spacing`](#compute_target_pitch_spacing) - Computes new pitch mark positions based on the target f0.
- [`find_pitch_marks`](#find_pitch_marks) - Finds pitch marks based on PYIN analysis. Only considers voiced frames with valid f0 values.
- [`overlap_add`](#overlap_add) - Performs overlap-add synthesis to reconstruct the output audio based on the original audio
- [`psola`](#psola) - Main PSOLA function that executes the pitch shifting process.

---

## autotune::audio::autotune::psola::compute_target_pitch_spacing

*Function*

Computes new pitch mark positions based on the target f0.
Adjusts spacing between marks according to the ratio of target f0 to original f0 at each frame.

```rust
fn compute_target_pitch_spacing(pyin_result: &crate::audio::autotune::pyin::PYINData, target_f0: &Vec<f32>, pitch_marks: &Vec<usize>) -> Vec<usize>
```



## autotune::audio::autotune::psola::find_pitch_marks

*Function*

Finds pitch marks based on PYIN analysis. Only considers voiced frames with valid f0 values.

```rust
fn find_pitch_marks(pyin: &crate::audio::autotune::pyin::PYINData, sample_rate: u32) -> Vec<usize>
```



## autotune::audio::autotune::psola::overlap_add

*Function*

Performs overlap-add synthesis to reconstruct the output audio based on the original audio
and the new pitch mark positions.

```rust
fn overlap_add(audio: &Vec<f32>, pitch_marks: &Vec<usize>, shifted_marks: &Vec<usize>, frame_size: usize) -> Vec<f32>
```



## autotune::audio::autotune::psola::psola

*Function*

Main PSOLA function that executes the pitch shifting process.

```rust
fn psola(audio: &Vec<f32>, sample_rate: u32, pyin_result: &crate::audio::autotune::pyin::PYINData, target_f0: &Vec<f32>, frame_size: Option<usize>, hop_size: Option<usize>) -> Vec<f32>
```



