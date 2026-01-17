use std::time::Duration;

use crate::audio::audio_controller;
use audio::audio_controller::AudioCommand;
use tokio::{sync::mpsc, time::sleep};
#[allow(dead_code, unused)]
mod audio;
mod gui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Start audio controller in its own thread

    let audio_file = audio::file::AudioFileData::load("/home/rb/Downloads/Radiohead - Nude.wav")?;
    println!(
        "Sample rate: {}, channels: {}, n_samples: {}",
        audio_file.sample_rate(),
        audio_file.n_channels(),
        audio_file.n_samples()
    );
    let audio = audio_file.to_audio();
    //   println!("{:?}", audio);
    let init_audio = audio.clone();
    let (tx, rx) = mpsc::channel::<audio_controller::AudioCommand>(32);
    // Start audio controller in its own Tokio task
    let _audio_handle = tokio::spawn(async move {
        let result = audio_controller::AudioController::new(rx, Some(init_audio))
            .map_err(|e| anyhow::anyhow!("AudioController::new failed: {e:?}"));
        let mut audio_controller = result.unwrap();
        audio_controller.run().await;
    });
    tx.send(AudioCommand::Play).await?;
    sleep(Duration::from_secs(5)).await;
    tx.send(AudioCommand::SetVolume(0.5)).await?;
    // tx.send(AudioCommand::Stop).await?;
    gui::run().map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(())
}
