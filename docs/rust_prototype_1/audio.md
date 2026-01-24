**rust_prototype_1 > audio**

# Module: audio

## Contents

**Modules**

- [`autotune`](#autotune)
- [`clip_manager`](#clip_manager)
- [`file`](#file)

**Structs**

- [`AudioClip`](#audioclip)
- [`Key`](#key)

**Enums**

- [`Note`](#note)
- [`Scale`](#scale)

---

## rust_prototype_1::audio::AudioClip

*Struct*

**Fields:**
- `name: String`
- `sample_rate: u32`
- `n_samples: usize`
- `waveform: Vec<f32>`
- `uuid: egui::Id`

**Trait Implementations:**

- **Clone**
  - `fn clone(self: &Self) -> AudioClip`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`



## rust_prototype_1::audio::Key

*Struct*

**Fields:**
- `root: Note`
- `scale: Scale`

**Methods:**

- `fn get_midi_scale(self: &Self, octave1: i8, octave2: i8) -> Vec<u8>`
- `fn get_scale_frequencies(self: &Self, octave1: i8, octave2: i8) -> Vec<f32>`

**Traits:** Copy, Eq

**Trait Implementations:**

- **Clone**
  - `fn clone(self: &Self) -> Key`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`
- **FromStr**
  - `fn from_str(s: &str) -> Result<Self, <Self as >::Err>`
- **PartialEq**
  - `fn eq(self: &Self, other: &Key) -> bool`



## rust_prototype_1::audio::Note

*Enum*

**Variants:**
- `C`
- `Cs`
- `D`
- `Ds`
- `E`
- `F`
- `Fs`
- `G`
- `Gs`
- `A`
- `As`
- `B`
- `Db`
- `Eb`
- `Gb`
- `Ab`
- `Bb`

**Traits:** Eq, Copy

**Trait Implementations:**

- **FromStr**
  - `fn from_str(s: &str) -> Result<Self, <Self as >::Err>`
- **PartialEq**
  - `fn eq(self: &Self, other: &Note) -> bool`
- **Clone**
  - `fn clone(self: &Self) -> Note`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`



## rust_prototype_1::audio::Scale

*Enum*

**Variants:**
- `Major`
- `Minor`
- `Blues`
- `Pentatonic`
- `Chromatic`

**Traits:** Copy, Eq

**Trait Implementations:**

- **FromStr**
  - `fn from_str(s: &str) -> Result<Self, <Self as >::Err>`
- **PartialEq**
  - `fn eq(self: &Self, other: &Scale) -> bool`
- **Clone**
  - `fn clone(self: &Self) -> Scale`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`



## Module: autotune



## Module: clip_manager



## Module: file



