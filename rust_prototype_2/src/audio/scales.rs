use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    root: Note,
    scale: Scale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Note {
    C,
    Cs,
    D,
    Ds,
    E,
    F,
    Fs,
    G,
    Gs,
    A,
    As,
    B,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    Major,
    Minor,
    Blues,
    Pentatonic,
    Chromatic,
}

impl FromStr for Note {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "c" => Ok(Note::C),
            "c#" | "db" => Ok(Note::Cs),
            "d" => Ok(Note::D),
            "d#" | "eb" => Ok(Note::Ds),
            "e" => Ok(Note::E),
            "f" => Ok(Note::F),
            "f#" | "gb" => Ok(Note::Fs),
            "g" => Ok(Note::G),
            "g#" | "ab" => Ok(Note::Gs),
            "a" => Ok(Note::A),
            "a#" | "bb" => Ok(Note::As),
            "b" => Ok(Note::B),
            _ => Err(format!("Invalid note: {}", s)),
        }
    }
}

impl Into<String> for Note {
    fn into(self) -> String {
        match self {
            Note::C => "C".to_string(),
            Note::Cs => "C#".to_string(),
            Note::D => "D".to_string(),
            Note::Ds => "D#".to_string(),
            Note::E => "E".to_string(),
            Note::F => "F".to_string(),
            Note::Fs => "F#".to_string(),
            Note::G => "G".to_string(),
            Note::Gs => "G#".to_string(),
            Note::A => "A".to_string(),
            Note::As => "A#".to_string(),
            Note::B => "B".to_string(),
        }
    }
}

impl FromStr for Scale {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "major" => Ok(Scale::Major),
            "minor" => Ok(Scale::Minor),
            "pentatonic" => Ok(Scale::Pentatonic),
            "blues" => Ok(Scale::Blues),
            "chromatic" => Ok(Scale::Chromatic),
            _ => Err(format!("Invalid scale: {}", s)),
        }
    }
}

impl FromStr for Key {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let root = parts.next().ok_or("Missing root note")?.parse::<Note>()?;
        let scale = parts.next().ok_or("Missing scale")?.parse::<Scale>()?;
        Ok(Key { root, scale })
    }
}

impl Key {
    pub fn new(root: Note, scale: Scale) -> Self {
        Self { root, scale }
    }
    pub fn get_midi_scale(&self, octave1: i8, octave2: i8) -> Vec<u8> {
        let scale_intervals = match self.scale {
            Scale::Major => vec![0, 2, 4, 5, 7, 9, 11],
            Scale::Minor => vec![0, 2, 3, 5, 7, 8, 10],
            Scale::Blues => vec![0, 3, 5, 6, 7, 10],
            Scale::Pentatonic => vec![0, 2, 4, 7, 9],
            Scale::Chromatic => (0..12).collect(),
        };

        let root_midi = match self.root {
            Note::C => 0,
            Note::Cs => 1,
            Note::D => 2,
            Note::Ds => 3,
            Note::E => 4,
            Note::F => 5,
            Note::Fs => 6,
            Note::G => 7,
            Note::Gs => 8,
            Note::A => 9,
            Note::As => 10,
            Note::B => 11,
        };

        let mut midi_scale = Vec::new();
        for octave in octave1..=octave2 {
            let base = (octave + 1) * 12; // MIDI octave starts at -1
            for &interval in &scale_intervals {
                let midi_note = base + root_midi + interval;
                if midi_note >= 0 && midi_note <= 127 {
                    midi_scale.push(midi_note as u8);
                }
            }
        }
        midi_scale.sort_unstable();
        midi_scale.dedup();
        midi_scale
    }
    pub fn get_scale_frequencies(&self, octave1: i8, octave2: i8) -> Vec<f32> {
        let midi_scale = self.get_midi_scale(octave1, octave2);
        midi_scale
            .iter()
            .map(|&m| 440.0 * 2f32.powf((m as f32 - 69.0) / 12.0))
            .collect()
    }
    pub fn get_scale_note_names(&self, octave1: i8, octave2: i8) -> Vec<String> {
        let midi_scale = self.get_midi_scale(octave1, octave2);
        midi_scale
            .iter()
            .map(|&m| {
                let note_index = m % 12;
                let octave = (m / 12) - 1;
                let note_name = match note_index {
                    0 => "C",
                    1 => "C#",
                    2 => "D",
                    3 => "D#",
                    4 => "E",
                    5 => "F",
                    6 => "F#",
                    7 => "G",
                    8 => "G#",
                    9 => "A",
                    10 => "A#",
                    11 => "B",
                    _ => unreachable!(),
                };
                format!("{}{}", note_name, octave)
            })
            .collect()
    }
}

pub fn frequency_to_midi_note(freq: f32) -> f32 {
    69.0 + 12.0 * (freq / 440.0).log2()
}
