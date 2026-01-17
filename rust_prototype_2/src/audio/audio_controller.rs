use crate::audio::{Audio, interleave_stereo};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub enum AudioCommand {
    SendAudio(Audio),
    ClearBuffer,
    Play,
    Stop,
    SetVolume(f32),
    Shutdown,
}

pub struct AudioController {
    receiver: tokio::sync::mpsc::Receiver<AudioCommand>,
    audio: Arc<Mutex<Option<Audio>>>,
    volume: Arc<Mutex<f32>>,
    position: Arc<Mutex<usize>>,
    playing: Arc<Mutex<bool>>,
    _stream: cpal::Stream,
}

impl AudioController {
    pub fn new(
        receiver: tokio::sync::mpsc::Receiver<AudioCommand>,
        initial_audio: Option<Audio>,
    ) -> anyhow::Result<Self> {
        let host = cpal::default_host();
        println!("Using audio host: {:?}", host.id());
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No output device available"))?;
        let supported_config = device.default_output_config()?;
        println!("Default output config: {:?}", supported_config);
        let sample_format = supported_config.sample_format();
        let mut config = supported_config.config();
        config.buffer_size = cpal::BufferSize::Fixed(512);
        println!("CPAL StreamConfig: {:?}", config);
        let channels = config.channels as usize;
        if channels != 2 {
            return Err(anyhow::anyhow!("expected stereo output, got {channels}"));
        }

        let volume = Arc::new(Mutex::new(1.0f32));
        let position = Arc::new(Mutex::new(0usize));
        let audio: Arc<Mutex<Option<Audio>>> = Arc::new(Mutex::new(initial_audio));
        let playing = Arc::new(Mutex::new(false));

        let shared_volume = Arc::clone(&volume);
        let shared_position = Arc::clone(&position);
        let audio_for_callback = Arc::clone(&audio);
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
                    eprintln!("CPAL stream error: {err}");
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
            audio,
            volume,
            position,
            playing,
            _stream: stream,
        })
    }

    fn fill_output_buffer(
        audio_for_callback: &Arc<Mutex<Option<Audio>>>,
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
                eprintln!("audio_for_callback mutex poisoned: {e}");
                for s in output.iter_mut() {
                    *s = 0.0;
                }
                return;
            }
        };
        let mut pos = match shared_position.lock() {
            Ok(g) => g,
            Err(e) => {
                eprintln!("shared_position mutex poisoned: {e}");
                for s in output.iter_mut() {
                    *s = 0.0;
                }
                return;
            }
        };
        let vol = match shared_volume.lock() {
            Ok(g) => *g,
            Err(e) => {
                eprintln!("shared_volume mutex poisoned: {e}");
                for s in output.iter_mut() {
                    *s = 0.0;
                }
                return;
            }
        };
        let is_playing = match playing.lock() {
            Ok(g) => *g,
            Err(e) => {
                eprintln!("playing mutex poisoned: {e}");
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
        /*
        eprintln!(
            "fill_output_buffer: len={} channels={} pos={} playing={} vol={}",
            output.len(),
            channels,
            *pos,
            is_playing,
            vol,
        );
        */

        if let Some(audio) = &*audio_lock {
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

            if *pos >= left.len().min(right.len()) {
                *pos = 0;
            }
        }
    }

    pub async fn run(&mut self) {
        while let Some(command) = self.receiver.recv().await {
            match command {
                // Adding audio to the buffer without overwriting it or starting playback
                // Also checking that the stereo channels have the same length
                AudioCommand::SendAudio(data) => {
                    let mut audio_lock = self.audio.lock().unwrap();

                    match &mut *audio_lock {
                        Some(existing) => {
                            // Ensure channels have same length
                            assert_eq!(existing.left.len(), existing.right.len());
                            assert_eq!(data.left.len(), data.right.len());

                            existing.left.extend_from_slice(&data.left);
                            existing.right.extend_from_slice(&data.right);
                        }
                        None => {
                            *audio_lock = Some(data);
                            *self.position.lock().unwrap() = 0;
                        }
                    }
                }
                AudioCommand::Play => {
                    println!("AudioController: Play command received");
                    *self.position.lock().unwrap() = 0;
                    *self.playing.lock().unwrap() = true;
                }
                AudioCommand::Stop => {
                    *self.playing.lock().unwrap() = false;
                    *self.position.lock().unwrap() = 0;
                }
                AudioCommand::SetVolume(volume) => {
                    *self.volume.lock().unwrap() = volume;
                }
                AudioCommand::ClearBuffer => {
                    *self.audio.lock().unwrap() = None;
                    *self.position.lock().unwrap() = 0;
                }
                AudioCommand::Shutdown => break,
            }
        }
    }
}
