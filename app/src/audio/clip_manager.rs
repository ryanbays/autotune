use crate::audio::AudioClip;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct ClipManager {
    pub clips: Vec<AudioClip>,
    clip_receiver: UnboundedReceiver<AudioClip>,
    clip_sender: UnboundedSender<AudioClip>,
}

impl ClipManager {
    pub fn new() -> Self {
        let (clip_sender, clip_receiver) = tokio::sync::mpsc::unbounded_channel();
        Self {
            clips: Vec::new(),
            clip_receiver,
            clip_sender,
        }
    }

    pub fn update(&mut self) {
        while let Ok(clip) = self.clip_receiver.try_recv() {
            println!("Adding new clip: {}", clip.name);
            self.clips.push(clip);
            println!("Total clips: {:?}", self.clips);
        }
    }

    pub fn load_through_rfd(&mut self) {
        let clips_clone = self.clips.clone();
        let sender = self.clip_sender.clone();

        tokio::spawn(async move {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Audio", &["wav"])
                .set_directory("~")
                .pick_file()
            {
                if let Ok(audio) = crate::audio::file::AudioFile::load(&path) {
                    for clip in &clips_clone {
                        if clip.uuid == egui::Id::new(path.to_string_lossy()) {
                            println!("Clip already exists: {}", clip.name);
                            return;
                        }
                    }
                    let clip = AudioClip {
                        name: path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned(),
                        sample_rate: audio.get_spec().sample_rate,
                        n_samples: audio.get_samples().len(),
                        waveform: audio.get_samples().to_vec(),
                        uuid: egui::Id::new(path.to_string_lossy()),
                    };

                    if let Err(e) = sender.send(clip) {
                        println!("Failed to send clip: {}", e);
                    }
                }
            }
        });
    }
}
