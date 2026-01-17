use std::time::Duration;

use crate::audio::audio_controller;
use audio::audio_controller::AudioCommand;
use tokio::{sync::mpsc, time::sleep};
#[allow(dead_code, unused)]
mod audio;
mod gui;
use crate::audio::autotune;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Start audio controller in its own thread

    // let audio_file = audio::file::AudioFileData::load("/home/rb/Downloads/Radiohead - Nude.wav")?;
    let audio_file = audio::file::AudioFileData::load("../audio/vocals1.wav")?;
    println!(
        "Sample rate: {}, channels: {}, n_samples: {}",
        audio_file.sample_rate(),
        audio_file.n_channels(),
        audio_file.n_samples()
    );
    let mut audio = audio_file.to_audio();
    audio.perform_pyin_async().await;
    let (tx, rx) = mpsc::channel::<audio_controller::AudioCommand>(32);
    let _audio_handle = tokio::spawn(async move {
        let result = audio_controller::AudioController::new(rx, None)
            .map_err(|e| anyhow::anyhow!("AudioController::new failed: {e:?}"));
        let mut audio_controller = result.unwrap();
        audio_controller.run().await;
    });
    tx.send(AudioCommand::SendAudio(audio.clone())).await?;
    tx.send(AudioCommand::Play).await?;
    // Pitch shift audio clip by 5 semitones
    let semitone_ratio = 2f32.powf(5.0 / 12.0);
    let shifted_f0 = audio
        .get_pyin()
        .unwrap()
        .f0()
        .iter()
        .map(|f0| f0 * semitone_ratio)
        .collect::<Vec<f32>>();
    println!("Starting PSOLA pitch shift");
    let new_signal = autotune::psola::psola(
        &audio.left().to_vec(),
        audio.sample_rate(),
        &audio.get_pyin().unwrap(),
        &shifted_f0,
        None,
        None,
    );
    println!("PSOLA pitch shift complete");
    let new_audio = audio::Audio::new(audio.sample_rate(), new_signal.clone(), new_signal.clone());
    tx.send(AudioCommand::SendAudio(new_audio)).await?;
    sleep(Duration::from_secs(5)).await;
    tx.send(AudioCommand::SetVolume(0.5)).await?;
    gui::run().map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(())
}
