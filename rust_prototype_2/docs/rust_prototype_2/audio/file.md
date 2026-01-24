**rust_prototype_2 > audio > file**

# Module: audio::file

## Contents

**Structs**

- [`AudioFileData`](#audiofiledata) - Audio file with interleaved samples:

---

## rust_prototype_2::audio::file::AudioFileData

*Struct*

Audio file with interleaved samples:
layout = [ch0_f0, ch1_f0, ..., ch{n-1}_f0, ch0_f1, ch1_f1, ...]

**Fields:**
- `samples: Vec<f32>`
- `n_samples: usize`
- `sample_rate: u32`
- `n_channels: usize`
- `file_path: std::path::PathBuf`

**Methods:**

- `fn load<P>(path: P) -> Result<Self>` - Uses rodio::Decoder, which yields interleaved samples for multichannel audio.
- `fn new(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Result<Self>`
- `fn save<P>(self: &Self, path: P) -> Result<()>` - Save audio data to a WAV file (16-bit PCM, interleaved channels).
- `fn to_audio(self: &Self) -> Audio`
- `fn from_audio(audio: &Audio) -> Self`
- `fn n_channels(self: &Self) -> usize`
- `fn sample_rate(self: &Self) -> u32`
- `fn n_samples(self: &Self) -> usize`
- `fn len_samples_raw(self: &Self) -> usize` - Total number of interleaved sample values (frames * channels).
- `fn is_empty(self: &Self) -> bool`
- `fn samples(self: &Self) -> &[f32]`

**Trait Implementations:**

- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`
- **Clone**
  - `fn clone(self: &Self) -> AudioFileData`



