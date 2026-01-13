pub mod autotune;
pub mod clip_manager;
pub mod file;

use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    root: Note,
    scale: Scale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Note {
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
    Db,
    Eb,
    Gb,
    Ab,
    Bb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scale {
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
            Note::Cs | Note::Db => 1,
            Note::D => 2,
            Note::Ds | Note::Eb => 3,
            Note::E => 4,
            Note::F => 5,
            Note::Fs | Note::Gb => 6,
            Note::G => 7,
            Note::Gs | Note::Ab => 8,
            Note::A => 9,
            Note::As | Note::Bb => 10,
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
}

#[derive(Debug, Clone)]
pub struct AudioClip {
    pub name: String,
    pub sample_rate: u32,
    pub n_samples: usize,
    pub waveform: Vec<f32>, // Normalized waveform data (-1.0 to 1.0)
    pub uuid: egui::Id,     // Unique identifier for the clip
}
