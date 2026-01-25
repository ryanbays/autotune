**autotune > audio > scales**

# Module: audio::scales

## Contents

**Structs**

- [`Key`](#key)

**Enums**

- [`Note`](#note)
- [`Scale`](#scale)

**Functions**

- [`frequency_to_midi_note`](#frequency_to_midi_note)
- [`midi_note_to_frequency`](#midi_note_to_frequency)
- [`note_name_to_midi_note`](#note_name_to_midi_note)

---

## autotune::audio::scales::Key

*Struct*

**Fields:**
- `root: Note`
- `scale: Scale`

**Methods:**

- `fn new(root: Note, scale: Scale) -> Self`
- `fn get_midi_scale(self: &Self, octave1: i8, octave2: i8) -> Vec<u8>`
- `fn get_scale_frequencies(self: &Self, octave1: i8, octave2: i8) -> Vec<f32>`
- `fn get_scale_note_names(self: &Self, octave1: i8, octave2: i8) -> Vec<String>`

**Traits:** Eq, Copy

**Trait Implementations:**

- **FromStr**
  - `fn from_str(s: &str) -> Result<Self, <Self as >::Err>`
- **PartialEq**
  - `fn eq(self: &Self, other: &Key) -> bool`
- **Clone**
  - `fn clone(self: &Self) -> Key`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`



## autotune::audio::scales::Note

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

**Traits:** Eq, Copy

**Trait Implementations:**

- **Into**
  - `fn into(self: Self) -> String`
- **FromStr**
  - `fn from_str(s: &str) -> Result<Self, <Self as >::Err>`
- **PartialEq**
  - `fn eq(self: &Self, other: &Note) -> bool`
- **Clone**
  - `fn clone(self: &Self) -> Note`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`



## autotune::audio::scales::Scale

*Enum*

**Variants:**
- `Major`
- `Minor`
- `Blues`
- `Pentatonic`
- `Chromatic`

**Traits:** Eq, Copy

**Trait Implementations:**

- **FromStr**
  - `fn from_str(s: &str) -> Result<Self, <Self as >::Err>`
- **PartialEq**
  - `fn eq(self: &Self, other: &Scale) -> bool`
- **Clone**
  - `fn clone(self: &Self) -> Scale`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`



## autotune::audio::scales::frequency_to_midi_note

*Function*

```rust
fn frequency_to_midi_note(freq: f32) -> f32
```



## autotune::audio::scales::midi_note_to_frequency

*Function*

```rust
fn midi_note_to_frequency(midi_note: f32) -> f32
```



## autotune::audio::scales::note_name_to_midi_note

*Function*

```rust
fn note_name_to_midi_note(name: &str) -> anyhow::Result<f32, String>
```



