**autotune > audio > autotune > psola**

# Module: audio::autotune::psola

## Contents

**Functions**

- [`compute_target_pitch_spacing`](#compute_target_pitch_spacing)
- [`find_pitch_marks`](#find_pitch_marks)
- [`overlap_add`](#overlap_add)
- [`psola`](#psola)

---

## autotune::audio::autotune::psola::compute_target_pitch_spacing

*Function*

```rust
fn compute_target_pitch_spacing(pyin_result: &crate::audio::autotune::pyin::PYINData, target_f0: &Vec<f32>, pitch_marks: &Vec<usize>) -> Vec<usize>
```



## autotune::audio::autotune::psola::find_pitch_marks

*Function*

```rust
fn find_pitch_marks(pyin: &crate::audio::autotune::pyin::PYINData, sample_rate: u32) -> Vec<usize>
```



## autotune::audio::autotune::psola::overlap_add

*Function*

```rust
fn overlap_add(audio: &Vec<f32>, pitch_marks: &Vec<usize>, shifted_marks: &Vec<usize>, frame_size: usize) -> Vec<f32>
```



## autotune::audio::autotune::psola::psola

*Function*

```rust
fn psola(audio: &Vec<f32>, sample_rate: u32, pyin_result: &crate::audio::autotune::pyin::PYINData, target_f0: &Vec<f32>, frame_size: Option<usize>, hop_size: Option<usize>) -> Vec<f32>
```



