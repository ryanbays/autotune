**autotune > audio > scales**

# Module: audio::scales

## Contents

**Structs**

- [`Key`](#key) - Represents a musical key, defined by a root note and a scale type

**Enums**

- [`Note`](#note) - Represents a musical note (C, Cs, D, etc.). Only sharps are supported for simplicity.
- [`Scale`](#scale) - Represents different types of musical scales

**Functions**

- [`frequency_to_midi_note`](#frequency_to_midi_note) - Converts a frequency in Hz to a MIDI note number (where A4 = 69 and 440 Hz)
- [`midi_note_to_frequency`](#midi_note_to_frequency) - Converts a MIDI note number to a frequency in Hz (where A4 = 69 and 440 Hz)
- [`note_name_to_midi_note`](#note_name_to_midi_note)

---

## autotune::audio::scales::Key

*Struct*

Represents a musical key, defined by a root note and a scale type

**Fields:**
- `root: Note`
- `scale: Scale`

**Methods:**

- `fn new(root: Note, scale: Scale) -> Self`
- `fn get_midi_scale(self: &Self, octave1: i8, octave2: i8) -> Vec<u8>` - Returns a vector of MIDI note numbers that belong to the key's scale across the specified octave range
- `fn get_scale_note_names(self: &Self, octave1: i8, octave2: i8) -> Vec<String>` - Returns a vector of note names (e.g., "C4", "D#4") that belong to the key's scale across the specified octave range

**Traits:** Copy, Eq

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

Represents a musical note (C, Cs, D, etc.). Only sharps are supported for simplicity.

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

Represents different types of musical scales

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

Converts a frequency in Hz to a MIDI note number (where A4 = 69 and 440 Hz)

```rust
fn frequency_to_midi_note(freq: f32) -> f32
```



## autotune::audio::scales::midi_note_to_frequency

*Function*

Converts a MIDI note number to a frequency in Hz (where A4 = 69 and 440 Hz)

```rust
fn midi_note_to_frequency(midi_note: f32) -> f32
```



## autotune::audio::scales::note_name_to_midi_note

*Function*

```rust
fn note_name_to_midi_note(name: &str) -> anyhow::Result<f32, String>
```



