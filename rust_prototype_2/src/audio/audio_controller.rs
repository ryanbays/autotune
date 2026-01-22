use crate::audio::{Audio, interleave_stereo};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info};

/// Commands sent to the AudioController for processing
/// Each command represents an action to be performed on the audio playback system
/**
- SendTrack(Audio, u32): Send audio data to be played on a specific track index.
- RemoveTrack(u32): Remove the audio track at the specified index.
- ClearBuffer: Clear the current audio buffer.
- Play: Start audio playback.
- Stop: Stop audio playback.
- SetReadPosition(usize): Set the current read position in the audio buffer.
- SetVolume(f32): Set the playback volume.
- Shutdown: Shut down the audio controller and stop playback.
*/
#[derive(Debug)]
pub enum AudioCommand {
    SendTrack(Audio, u32),
    RemoveTrack(u32),
    ClearBuffer,
    Play,
    Stop,
    SetReadPosition(usize),
    SetVolume(f32),
    Shutdown,
}

/// Controller for managing audio playback using CPAL
/// It handles commands to play, stop, and manipulate audio tracks
/// and mixes multiple audio tracks into a single output buffer.
pub struct AudioController {
    receiver: tokio::sync::mpsc::Receiver<AudioCommand>,
    tracks: HashMap<u32, Audio>,
    audio_buffer: Arc<Mutex<Audio>>,
    volume: Arc<Mutex<f32>>,
    position: Arc<Mutex<usize>>,
    playing: Arc<Mutex<bool>>,
    _stream: cpal::Stream,
}

impl AudioController {
    pub fn new(receiver: tokio::sync::mpsc::Receiver<AudioCommand>) -> anyhow::Result<Self> {
        info!("Initializing AudioController");
        let host = cpal::default_host();
        debug!(audio_host = ?host.id(), "Using audio host");
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No output device available"))?;
        let supported_config = device.default_output_config()?;
        debug!("Default output config: {:?}", supported_config);
        let sample_format = supported_config.sample_format();
        let mut config = supported_config.config();
        config.buffer_size = cpal::BufferSize::Fixed(512);
        debug!("CPAL StreamConfig: {:?}", config);
        let channels = config.channels as usize;
        if channels != 2 {
            return Err(anyhow::anyhow!("expected stereo output, got {channels}"));
        }

        let volume = Arc::new(Mutex::new(1.0f32));
        let position = Arc::new(Mutex::new(0usize));
        let audio_buffer = Arc::new(Mutex::new(Audio::new(44100, Vec::new(), Vec::new())));
        let playing = Arc::new(Mutex::new(false));

        let shared_volume = Arc::clone(&volume);
        let shared_position = Arc::clone(&position);
        let audio_for_callback = Arc::clone(&audio_buffer);
        let playing_for_callback = Arc::clone(&playing);

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &config,
                move |output: &mut [f32], _| {
                    Self::fill_output_buffer(
                        &audio_for_callback,
                        &shared_position,
                        &shared_volume,
                        &playing_for_callback,
                        output,
                        channels,
                    );
                },
                move |err| {
                    info!("CPAL stream error: {err}");
                },
                None,
            )?,
            other => {
                return Err(anyhow::anyhow!("Unsupported sample format: {other:?}"));
            }
        };
        stream.play()?;
        Ok(Self {
            receiver,
            audio_buffer,
            volume,
            tracks: HashMap::new(),
            position,
            playing,
            _stream: stream,
        })
    }

    /// Get the current volume level
    pub fn get_volume(&self) -> f32 {
        *self.volume.lock().unwrap()
    }

    /// Check if audio is currently playing
    pub fn is_playing(&self) -> bool {
        *self.playing.lock().unwrap()
    }

    /// Get the current read position in the audio buffer
    pub fn get_position(&self) -> usize {
        *self.position.lock().unwrap()
    }

    /// Fills the output buffer with audio data from the shared audio buffer
    /// Applies volume control and handles playback state
    /// This function is called within the CPAL audio callback
    fn fill_output_buffer(
        audio_for_callback: &Arc<Mutex<Audio>>,
        shared_position: &Arc<Mutex<usize>>,
        shared_volume: &Arc<Mutex<f32>>,
        playing: &Arc<Mutex<bool>>,
        output: &mut [f32],
        channels: usize,
    ) {
        // Panicking out of a callback is bad, so handle mutex poisoning gracefully
        let audio_lock = match audio_for_callback.lock() {
            Ok(g) => g,
            Err(e) => {
                error!("audio_for_callback mutex poisoned: {e}");
                for s in output.iter_mut() {
                    *s = 0.0;
                }
                return;
            }
        };
        let mut pos = match shared_position.lock() {
            Ok(g) => g,
            Err(e) => {
                error!("shared_position mutex poisoned: {e}");
                for s in output.iter_mut() {
                    *s = 0.0;
                }
                return;
            }
        };
        let vol = match shared_volume.lock() {
            Ok(g) => *g,
            Err(e) => {
                error!("shared_volume mutex poisoned: {e}");
                for s in output.iter_mut() {
                    *s = 0.0;
                }
                return;
            }
        };
        let is_playing = match playing.lock() {
            Ok(g) => *g,
            Err(e) => {
                error!("playing mutex poisoned: {e}");
                for s in output.iter_mut() {
                    *s = 0.0;
                }
                return;
            }
        };

        // Always clear the buffer first
        for sample in output.iter_mut() {
            *sample = 0.0;
        }

        if !is_playing {
            return;
        }

        let audio = &*audio_lock;
        let left = &audio.left;
        let right = &audio.right;

        let frames_out = output.len() / channels;
        let remaining_frames = left.len().min(right.len()).saturating_sub(*pos);
        let frames_to_write = frames_out.min(remaining_frames);

        if frames_to_write == 0 {
            return;
        }

        let start = *pos;
        let end = start + frames_to_write;
        interleave_stereo(
            &left[start..end],
            &right[start..end],
            &mut output[..frames_to_write * channels],
        );

        if vol != 1.0 {
            for s in &mut output[..frames_to_write * channels] {
                *s *= vol;
            }
        }

        *pos += frames_to_write;

        if *pos > left.len().min(right.len()) {
            *pos = 0;
        }
    }

    /// Mixes all tracks into the audio buffer, applying autotuning if desired F0 is provided.
    /// This function should be called whenever tracks are added, removed, or modified.
    /// It locks the audio buffer mutex to update the mixed audio.
    fn mix_tracks(&mut self) {
        let time_start = std::time::Instant::now();

        let mut mixed_audio = Audio::new(44100, Vec::new(), Vec::new());
        for key in &self.tracks.keys().cloned().collect::<Vec<u32>>() {
            let track = &self.tracks[key];
            if let Some(desired_f0) = &track.desired_f0 {
                debug!(
                    "AudioController: Autotuning track with desired F0 of length {}",
                    desired_f0.len()
                );
                match crate::audio::autotune::compute_shifted_audio(track) {
                    Ok(shifted_audio) => {
                        let result = mixed_audio.add_audio_at(0, &shifted_audio);
                        if let Err(e) = result {
                            error!("AudioController: Failed to add autotuned track: {}", e);
                        }
                    }
                    Err(e) => {
                        error!(
                            "AudioController: Autotuning failed, adding original track: {}",
                            e
                        );
                        let result = mixed_audio.add_audio_at(0, track);
                        if let Err(e) = result {
                            error!("AudioController: Failed to add track: {}", e);
                        }
                    }
                }
            } else {
                debug!("AudioController: No desired F0, adding original track");
                let result = mixed_audio.add_audio_at(0, track);
                if let Err(e) = result {
                    error!("AudioController: Failed to add track: {}", e);
                }
            }
        }
        *self.audio_buffer.lock().unwrap() = mixed_audio;

        let duration = time_start.elapsed();
        debug!(
            "AudioController: Mixing {} tracks took {:?}",
            self.tracks.len(),
            duration
        );
    }

    /// Main loop processing incoming audio commands
    pub async fn run(&mut self) {
        while let Some(command) = self.receiver.recv().await {
            match command {
                AudioCommand::SendTrack(data, id) => {
                    debug!("AudioController: SendAudio command received");
                    self.mix_tracks();
                    self.tracks.insert(id, data);
                }
                AudioCommand::RemoveTrack(id) => {
                    self.mix_tracks();
                    debug!("AudioController: RemoteTrack command received: {}", id);
                    if (id as usize) < self.tracks.len() {
                        self.tracks.remove(&id);
                    } else {
                        error!("AudioController: RemoteTrack id out of bounds: {}", id);
                    }
                }
                AudioCommand::SetReadPosition(position) => {
                    debug!(
                        "AudioController: SetReadPosition command received: {}",
                        position
                    );
                    *self.position.lock().unwrap() = position;
                }
                AudioCommand::Play => {
                    debug!("AudioController: Play command received");
                    if self.playing.lock().unwrap().clone() {
                        debug!("AudioController: Already playing, ignoring Play command");
                        continue;
                    }
                    *self.playing.lock().unwrap() = true;
                }
                AudioCommand::Stop => {
                    debug!("AudioController: Stop command received");
                    *self.playing.lock().unwrap() = false;
                }
                AudioCommand::SetVolume(volume) => {
                    debug!("AudioController: SetVolume command received: {}", volume);
                    *self.volume.lock().unwrap() = volume;
                }
                AudioCommand::ClearBuffer => {
                    debug!("AudioController: ClearBuffer command received");
                }
                AudioCommand::Shutdown => {
                    debug!("AudioController: Shutdown command received");
                    break;
                }
            }
        }
    }
}
