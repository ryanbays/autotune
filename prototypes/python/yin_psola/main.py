import numpy as np
import librosa  # still needed for audio loading and music theory functions
import scipy.signal as sig
import soundfile as sf
from .pyin import pyin
from .psola import psola

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
    closest = np.zeros_like(f0)

    # for each pitch, get the closest pitch on the scale
    for i in range(f0.shape[0]): 
        closest[i] = get_closest_pitch(f0[i], scale)

    # smooth over time
    med_filtered = sig.medfilt(closest, kernel_size=11)
    med_filtered[np.isnan(med_filtered)] = closest[np.isnan(med_filtered)]
    return med_filtered


def autotune(y, sr, scale):
    fmin = librosa.note_to_hz('C2')  # minimum frequency
    fmax = librosa.note_to_hz('C7')  # maximum frequency
    frame_length = 2048
    hop_length = frame_length // 4

    # 1. pitch detection using our PYIN implementation
    f0, voiced_flag, voiced_prob = pyin(
        y,
        frame_length=frame_length,
        hop_length=hop_length,
        sr=sr,
        fmin=fmin,
        fmax=fmax
    )

    # 2. calculate the correct pitch
    corrected_f0 = calculate_correct_pitch(f0, scale)

    # 3. pitch shifting using our PSOLA implementation
    pitch_shifted = psola(
        y,
        target_f0=corrected_f0,
        sr=sr,
        frame_length=frame_length,
        hop_length=hop_length,
        fmin=fmin,
        fmax=fmax
    )

    return pitch_shifted


def main():
    audio_file_path = '/content/skyfallwav.wav'

    # returns audio time series and sampling rate
    # mono=True returns only one channel
    y, sr = librosa.load(audio_file_path, mono=True)
    print(f"Audio time series: {y}")
    print(f"Sampling rate: {sr} Hz")

    scale = "C:min"  # scale we will use to calculate the right pitch
    autotune_result = autotune(y, sr, scale)

    # write to an output file
    filepath = "/content/output.wav"
    sf.write(str(filepath), autotune_result, sr)

if __name__ == "__main__":
    main()
