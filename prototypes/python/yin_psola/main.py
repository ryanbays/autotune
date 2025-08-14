import numpy as np
import librosa  # still needed for audio loading and music theory functions
import scipy.signal as sig
import soundfile as sf
from pyin import pyin
from psola import psola
import time

# Constant definition
SEMITONES_IN_OCTAVE = 12

def get_closest_pitch(value, scale):
    if np.isnan(value):
        return np.nan

    degrees = librosa.key_to_degrees(scale)

    # if the value is very close to the start of the next octave,
    # ensure that we round up correctly 
    degrees = np.concatenate((degrees, [degrees[0] + SEMITONES_IN_OCTAVE]))

    midi_note = librosa.hz_to_midi(value)
    degree = librosa.hz_to_midi(value) % SEMITONES_IN_OCTAVE
    closest_pitch_class = np.argmin(np.abs(degrees-degree))

    # get diff btwn input vs desired pitch class
    degree_diff = degree - degrees[closest_pitch_class]

    midi_note -= degree_diff
    return librosa.midi_to_hz(midi_note)  # convert back to hertz


def calculate_correct_pitch(f0, scale):
    print("Calculating correct pitch for each frame...")
    closest = np.zeros_like(f0)

    # for each pitch, get the closest pitch on the scale
    for i in range(f0.shape[0]): 
        closest[i] = get_closest_pitch(f0[i], scale)
        if i % 1000 == 0:  # Log every 1000 frames
            print(f"Processed {i}/{f0.shape[0]} frames...")

    # smooth over time
    print("Applying median filtering...")
    med_filtered = sig.medfilt(closest, kernel_size=11)
    med_filtered[np.isnan(med_filtered)] = closest[np.isnan(med_filtered)]
    return med_filtered


def autotune(y, sr, scale):
    print("\nStarting autotune process...")
    start_time = time.time()
    
    fmin = librosa.note_to_hz('C2')  # minimum frequency
    fmax = librosa.note_to_hz('C7')  # maximum frequency
    frame_length = 2048
    hop_length = frame_length // 4

    print("1. Detecting pitch using PYIN algorithm...")
    f0, voiced_flag, voiced_prob = pyin(
        y,
        frame_length=frame_length,
        hop_length=hop_length,
        sr=sr,
        fmin=fmin,
        fmax=fmax
    )
    print(f"Pitch detection complete. Found {len(f0)} frames.")

    print("\n2. Calculating correct pitch for each frame...")
    corrected_f0 = calculate_correct_pitch(f0, scale)

    print("\n3. Applying pitch shifting using PSOLA...")
    pitch_shifted = psola(
        y,
        target_f0=corrected_f0,
        sr=sr,
        frame_length=frame_length,
        hop_length=hop_length,
        fmin=fmin,
        fmax=fmax
    )

    elapsed_time = time.time() - start_time
    print(f"\nAutotune process completed in {elapsed_time:.2f} seconds")
    return pitch_shifted


def main():
    print("Audio Autotune Tool")
    print("==================")
    
    audio_file_path = ""
    while True:
        try:
            audio_file_path = input("Enter the path to the audio file: ").strip()
            if not audio_file_path:
                print("Error: Please enter a valid file path")
                continue
            print("Loading audio file...")
            # returns audio time series and sampling rate
            # mono=True returns only one channel
            y, sr = librosa.load(audio_file_path, mono=True)
            print(f"Successfully loaded audio file: {len(y)/sr:.2f} seconds long")
            break
        except FileNotFoundError:
            print(f"Error: File '{audio_file_path}' not found")
        except Exception as e:
            print(f"Error loading audio file: {str(e)}")
            print(f"Audio time series: {y}")
    print(f"Sampling rate: {sr} Hz")

    scale = "C:min"  # scale we will use to calculate the right pitch
    print(f"Using scale: {scale}")
    
    autotune_result = autotune(y, sr, scale)

    # write to an output file
    filepath = "/content/output.wav"
    print(f"\nSaving output to: {filepath}")
    sf.write(str(filepath), autotune_result, sr)
    print("Processing complete!")

if __name__ == "__main__":
    main()
