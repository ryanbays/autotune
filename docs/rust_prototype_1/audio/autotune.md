**rust_prototype_1 > audio > autotune**

# Module: audio::autotune

## Contents

**Modules**

- [`psola`](#psola)
- [`pyin`](#pyin)

**Functions**

- [`pitch_shift`](#pitch_shift)
- [`snap_to_scale`](#snap_to_scale)

---

## rust_prototype_1::audio::autotune::pitch_shift

*Function*

```rust
fn pitch_shift(samples: &[f32], target_f0: &[f32], sample_rate: u32, frame_length: usize, hop_length: usize, f_min: f32, f_max: f32) -> Vec<f32>
```



## Module: psola



## Module: pyin



## rust_prototype_1::audio::autotune::snap_to_scale

*Function*

```rust
fn snap_to_scale(f0: &[f32], key: crate::audio::Key) -> Vec<f32>
```



