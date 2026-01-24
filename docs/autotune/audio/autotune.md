**autotune > audio > autotune**

# Module: audio::autotune

## Contents

**Modules**

- [`psola`](#psola)
- [`pyin`](#pyin)

**Functions**

- [`compute_shifted_audio`](#compute_shifted_audio) - Computes a shifted audio signal using the Audio struct's desired f0 and PYIN data.

**Constants**

- [`FRAME_LENGTH`](#frame_length)
- [`HOP_LENGTH`](#hop_length)
- [`MAX_F0`](#max_f0)
- [`MIN_F0`](#min_f0)
- [`PYIN_SIGMA`](#pyin_sigma)
- [`PYIN_THRESHOLD`](#pyin_threshold)

---

## autotune::audio::autotune::FRAME_LENGTH

*Constant*: `usize`



## autotune::audio::autotune::HOP_LENGTH

*Constant*: `usize`



## autotune::audio::autotune::MAX_F0

*Constant*: `f32`



## autotune::audio::autotune::MIN_F0

*Constant*: `f32`



## autotune::audio::autotune::PYIN_SIGMA

*Constant*: `f32`



## autotune::audio::autotune::PYIN_THRESHOLD

*Constant*: `f32`



## autotune::audio::autotune::compute_shifted_audio

*Function*

Computes a shifted audio signal using the Audio struct's desired f0 and PYIN data.
Returns the signal as a new audio struct.

```rust
fn compute_shifted_audio(audio: &crate::audio::Audio) -> anyhow::Result<crate::audio::Audio>
```



## Module: psola



## Module: pyin



