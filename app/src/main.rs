#[allow(dead_code)]
mod audio;
mod gui;

use crate::gui::AutotuneApp;
use clap::Parser;
use eframe::egui;
use std::option::Option;
use std::path::PathBuf;

/// Simple autotune application
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(long, action)]
    nogui: bool,

    /// Key to autotune to (e.g., "C major", "A minor", "E blues")
    #[arg(short, long, default_value = "C major")]
    scale: Option<audio::Key>,

    /// Input audio file
    #[arg(required_if_eq("nogui", "true"))]
    input: Option<PathBuf>,

    /// Output audio file
    #[arg(required_if_eq("nogui", "true"))]
    output: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if !args.nogui {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([320.0, 240.0])
                .with_decorations(true),
            ..Default::default()
        };
        eframe::run_native(
            "My egui App",
            options,
            Box::new(|_cc| Ok(Box::new(AutotuneApp::default()))),
        )
        .map_err(|e| anyhow::anyhow!("{}", e))?;
        return Ok(());
    }
    let input = args.input.expect("No input file provided");
    let output = args.output.unwrap_or_else(|| PathBuf::from("output.wav"));
    let scale = args.scale.expect("No scale provided");
    println!("Input file: {}", input.to_string_lossy());
    println!("Output file: {}", output.to_string_lossy());
    println!("Scale: {:?}", scale);

    let mut file = audio::file::AudioFile::load(input).expect("Failed to load input file");
    println!("Loaded file with {} samples", file.get_n_samples());

    file.run_pyin(2048, 256, 50.0, 2100.0, 0.1);
    println!("Ran PYIN pitch detection");

    let f0 = file.get_pyin_result().f0.as_slice().unwrap();
    println!("Estimated f0 length: {}", f0.len());

    let snapped_f0 = audio::autotune::snap_to_scale(f0, scale);
    println!("Snapped f0 length: {}", snapped_f0.len());

    let processed_samples = audio::autotune::pitch_shift(
        file.get_samples(),
        &snapped_f0,
        file.get_sample_rate(),
        500,
        500,
        50.0,
        2100.0,
    );
    println!("Processed samples length: {}", processed_samples.len());
    /*
    let output_file = audio::file::AudioFile::new(processed_samples, file.get_spec());
    output_file
        .save(&output)
        .expect("Failed to save output file");
    */
    Ok(())
}
