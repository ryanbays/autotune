import numpy as np
from typing import Tuple
from pyin import pyin

def psola(y: np.ndarray,
          target_f0: np.ndarray,
          sr: int = 44100,
          frame_length: int = 2048,
          hop_length: int = 512,
          fmin: float = 65.0,
          fmax: float = 2093.0) -> np.ndarray:
    """
    Implements the Pitch Synchronous Overlap and Add (PSOLA) algorithm for pitch shifting.
    
    Args:
        y: Input audio signal
        target_f0: Target fundamental frequency contour
        sr: Sample rate
        frame_length: Size of each analysis frame
        hop_length: Number of samples between successive frames
        fmin: Minimum frequency to detect (Hz)
        fmax: Maximum frequency to detect (Hz)
    
    Returns:
        Modified audio signal with altered pitch
    """
    # Get the source pitch using PYIN
    source_f0, voiced_flag, _ = pyin(y, frame_length, hop_length, sr, fmin, fmax)
    
    # Find pitch marks (peak locations)
    pitch_marks = find_pitch_marks(y, source_f0, voiced_flag, sr, hop_length)
    
    # Extract pitch-synchronous frames
    frames = extract_frames(y, pitch_marks, frame_length)
    
    # Compute time scaling factor for each frame
    scaling_factors = compute_scaling_factors(source_f0, target_f0, voiced_flag)
    
    # Generate new pitch marks
    new_pitch_marks = generate_new_pitch_marks(pitch_marks, scaling_factors)
    
    # Synthesize the output signal
    output = synthesize(frames, new_pitch_marks, len(y))
    
    return output

def find_pitch_marks(y: np.ndarray, 
                    f0: np.ndarray, 
                    voiced_flag: np.ndarray,
                    sr: int,
                    hop_length: int) -> np.ndarray:
    """
    Locate pitch marks (peaks) in the signal based on F0 estimates.
    """
    pitch_marks = []
    current_pos = 0
    
    for i, (freq, is_voiced) in enumerate(zip(f0, voiced_flag)):
        if is_voiced and freq > 0:
            period = int(sr / freq)
            frame_start = i * hop_length
            
            # Find local maximum in the period
            while current_pos < frame_start + hop_length and current_pos < len(y):
                # Search for peak in one period window
                window = y[current_pos:min(current_pos + period, len(y))]
                if len(window) > 0:
                    peak_offset = np.argmax(np.abs(window))
                    pitch_marks.append(current_pos + peak_offset)
                    current_pos += period
                else:
                    break
                    
    return np.array(pitch_marks)

def extract_frames(y: np.ndarray, 
                  pitch_marks: np.ndarray, 
                  frame_length: int) -> list:
    """
    Extract frames centered at pitch marks.
    """
    frames = []
    half_length = frame_length // 2
    
    for mark in pitch_marks:
        start = max(0, mark - half_length)
        end = min(len(y), mark + half_length)
        
        # Create frame and apply Hanning window
        frame = np.zeros(frame_length)
        actual_start = half_length - (mark - start)
        actual_end = actual_start + (end - start)
        frame[actual_start:actual_end] = y[start:end]
        frame *= np.hanning(frame_length)
        
        frames.append(frame)
        
    return frames

def compute_scaling_factors(source_f0: np.ndarray, 
                          target_f0: np.ndarray, 
                          voiced_flag: np.ndarray) -> np.ndarray:
    """
    Compute time scaling factors based on source and target F0.
    """
    scaling_factors = np.ones_like(source_f0)
    mask = (source_f0 > 0) & voiced_flag
    scaling_factors[mask] = target_f0[mask] / source_f0[mask]
    return scaling_factors

def generate_new_pitch_marks(pitch_marks: np.ndarray, 
                           scaling_factors: np.ndarray) -> np.ndarray:
    """
    Generate new pitch mark positions based on scaling factors.
    """
    new_pitch_marks = []
    current_pos = 0
    
    for i in range(len(pitch_marks) - 1):
        period = pitch_marks[i + 1] - pitch_marks[i]
        new_period = int(period * scaling_factors[i])
        new_pitch_marks.append(current_pos)
        current_pos += new_period
        
    new_pitch_marks.append(current_pos)
    return np.array(new_pitch_marks)

def synthesize(frames: list, 
              pitch_marks: np.ndarray, 
              output_length: int) -> np.ndarray:
    """
    Synthesize the output signal by overlap-adding frames at new pitch marks.
    """
    output = np.zeros(output_length)
    
    for i, (frame, mark) in enumerate(zip(frames, pitch_marks)):
        if mark < output_length:
            start = max(0, mark - len(frame)//2)
            end = min(output_length, mark + len(frame)//2)
            frame_start = max(0, len(frame)//2 - mark)
            frame_end = frame_start + (end - start)
            output[start:end] += frame[frame_start:frame_end]
            
    return output
