**rust_prototype_1 > audio > file**

# Module: audio::file

## Contents

**Structs**

- [`AudioFile`](#audiofile)

---

## rust_prototype_1::audio::file::AudioFile

*Struct*

**Fields:**
- `samples: Vec<f32>`
- `sample_rate: u32`
- `channels: u16`
- `pyin_result: Option<crate::audio::autotune::pyin::PYinResult>`

**Methods:**

- `fn load<P>(path: P) -> Result<Self>` - Load an audio file from the given path
- `fn new(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Self`
- `fn save<P>(self: &Self, path: P) -> Result<()>` - Save audio data to a WAV file
- `fn run_pyin(self: & mut Self, frame_length: usize, hop_length: usize, f_min: f32, f_max: f32, threshold: f32) -> Result<()>` - Run PYin pitch detection on the audio samples
- `fn get_pyin_result(self: & mut Self) -> &PYinResult`
- `fn get_channels(self: &Self) -> u16`
- `fn get_samples(self: &Self) -> &[f32]`
- `fn get_sample_rate(self: &Self) -> u32`
- `fn get_n_samples(self: &Self) -> usize`



