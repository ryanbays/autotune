import numpy as np
from typing import Tuple

def pyin(y: np.ndarray, 
         frame_length: int = 2048,
         hop_length: int = 512,
         sr: int = 44100,
         fmin: float = 65.0,
         fmax: float = 2093.0,
         threshold: float = 0.1) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
    """
    Implements the Probabilistic YIN (PYIN) pitch detection algorithm.
    
    Args:
        y: Input audio signal
        frame_length: Size of each analysis frame
        hop_length: Number of samples between successive frames
        sr: Sample rate
        fmin: Minimum frequency to detect (Hz)
        fmax: Maximum frequency to detect (Hz)
        threshold: Threshold for pitch detection
    
    Returns:
        Tuple containing:
        - Fundamental frequency estimates (f0)
        - Voice/unvoiced flag for each frame
        - Voiced probability for each frame
    """
    
    # Calculate number of frames
    n_frames = 1 + (len(y) - frame_length) // hop_length
    
    # Initialize output arrays
    f0 = np.zeros(n_frames)
    voiced_flag = np.zeros(n_frames, dtype=bool)
    voiced_prob = np.zeros(n_frames)
    
    for i in range(n_frames):
        # Get current frame
        start = i * hop_length
        frame = y[start:start + frame_length]
        
        # Compute difference function
        diff = difference_function(frame)
        
        # Compute cumulative mean normalized difference
        cmnd = cumulative_mean_normalized_difference(diff)
        
        # Find pitch period candidates
        tau_candidates = get_pitch_period_candidates(cmnd, threshold)
        
        if len(tau_candidates) > 0:
            # Convert best period to frequency
            period = tau_candidates[0]
            if period != 0:
                freq = sr / period
                
                # Check if frequency is within bounds
                if fmin <= freq <= fmax:
                    f0[i] = freq
                    voiced_flag[i] = True
                    voiced_prob[i] = 1.0 - cmnd[period]
    
    return f0, voiced_flag, voiced_prob

def difference_function(frame: np.ndarray) -> np.ndarray:
    """
    Compute the difference function for YIN algorithm.
    """
    frame_length = len(frame)
    diff = np.zeros(frame_length)
    
    for tau in range(frame_length):
        for j in range(frame_length - tau):
            diff[tau] += (frame[j] - frame[j + tau]) ** 2
            
    return diff

def cumulative_mean_normalized_difference(diff: np.ndarray) -> np.ndarray:
    """
    Compute the cumulative mean normalized difference function.
    """
    cmnd = np.zeros_like(diff)
    cmnd[0] = 1.0
    
    running_sum = 0.0
    for tau in range(1, len(diff)):
        running_sum += diff[tau]
        cmnd[tau] = diff[tau] * tau / running_sum if running_sum > 0 else 1.0
    
    return cmnd

def get_pitch_period_candidates(cmnd: np.ndarray, threshold: float) -> np.ndarray:
    """
    Find pitch period candidates from the CMND function.
    """
    # Find all local minima
    candidates = []
    
    for i in range(1, len(cmnd) - 1):
        if cmnd[i] < cmnd[i-1] and cmnd[i] < cmnd[i+1] and cmnd[i] < threshold:
            candidates.append(i)
    
    return np.array(candidates)
