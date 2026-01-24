**autotune > audio > audio_controller**

# Module: audio::audio_controller

## Contents

**Structs**

- [`AudioController`](#audiocontroller) - Controller for managing audio playback using CPAL

**Enums**

- [`AudioCommand`](#audiocommand) -  Commands sent to the AudioController for processing

---

## autotune::audio::audio_controller::AudioCommand

*Enum*

 Commands sent to the AudioController for processing
 Each command represents an action to be performed on the audio playback system
- SendTrack(Audio, u32): Send audio data to be played on a specific track index.
- RemoveTrack(u32): Remove the audio track at the specified index.
- ClearBuffer: Clear the current audio buffer.
- Play: Start audio playback.
- Stop: Stop audio playback.
- SetReadPosition(usize): Set the current read position in the audio buffer.
- SetVolume(f32): Set the playback volume.
- Shutdown: Shut down the audio controller and stop playback.

**Variants:**
- `SendTrack(crate::audio::Audio, u32)`
- `RemoveTrack(u32)`
- `ClearBuffer`
- `Play`
- `Stop`
- `SetReadPosition(usize)`
- `BroadcastPosition`
- `SetVolume(f32)`
- `Shutdown`

**Trait Implementations:**

- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`



## autotune::audio::audio_controller::AudioController

*Struct*

Controller for managing audio playback using CPAL
It handles commands to play, stop, and manipulate audio tracks
and mixes multiple audio tracks into a single output buffer.

**Fields:**
- `receiver: tokio::sync::mpsc::Receiver<AudioCommand>`
- `track_manager_sender: tokio::sync::mpsc::Sender<track::TrackManagerCommand>`
- `tracks: std::collections::HashMap<u32, crate::audio::Audio>`
- `audio_buffer: std::sync::Arc<std::sync::Mutex<crate::audio::Audio>>`
- `volume: std::sync::Arc<std::sync::Mutex<f32>>`
- `position: std::sync::Arc<std::sync::Mutex<usize>>`
- `playing: std::sync::Arc<std::sync::Mutex<bool>>`
- `_stream: cpal::Stream`

**Methods:**

- `fn new(receiver: tokio::sync::mpsc::Receiver<AudioCommand>, track_manager_sender: tokio::sync::mpsc::Sender<track::TrackManagerCommand>) -> anyhow::Result<Self>`
- `fn get_volume(self: &Self) -> f32` - Get the current volume level
- `fn is_playing(self: &Self) -> bool` - Check if audio is currently playing
- `fn get_position(self: &Self) -> usize` - Get the current read position in the audio buffer
- `fn fill_output_buffer(audio_for_callback: &Arc<Mutex<Audio>>, shared_position: &Arc<Mutex<usize>>, shared_volume: &Arc<Mutex<f32>>, playing: &Arc<Mutex<bool>>, output: & mut [f32], channels: usize)` - Fills the output buffer with audio data from the shared audio buffer
- `fn mix_tracks(self: & mut Self)` - Mixes all tracks into the audio buffer, applying autotuning if desired F0 is provided.
- `fn run(self: & mut Self)` - Main loop processing incoming audio commands



