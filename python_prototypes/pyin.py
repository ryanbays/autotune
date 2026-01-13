import numpy as np
from typing import Tuple, List


def autocorrelation_function(signal: np.ndarray, tau_max: int) -> np.ndarray:
    """Compute autocorrelation of the input signal."""
    length = len(signal)
    acf = np.zeros(tau_max)
    for tau in range(tau_max):
        for i in range(length - tau):
            acf[tau] += signal[i] * signal[i + tau]
    return acf


def difference_function(signal: np.ndarray, tau_max: int) -> np.ndarray:
    """Compute difference function for YIN algorithm."""
    length = len(signal)
    df = np.zeros(tau_max)
    for tau in range(1, tau_max):
        for i in range(length - tau):
            df[tau] += (signal[i] - signal[i + tau]) ** 2
    return df


def cumulative_mean_normalized_difference(df: np.ndarray) -> np.ndarray:
    """Compute CMND function for YIN algorithm."""
    cmnd = np.zeros_like(df)
    cmnd[0] = 1.0
    cumsum = 0.0
    for tau in range(1, len(df)):
        cumsum += df[tau]
        cmnd[tau] = df[tau] * tau / cumsum
    return cmnd


def parabolic_interpolation(cmnd: np.ndarray, tau: int) -> Tuple[float, float]:
    """Perform parabolic interpolation to refine frequency estimate."""
    if tau < 1 or tau >= len(cmnd) - 1:
        return tau, cmnd[tau]
    
    alpha = cmnd[tau - 1]
    beta = cmnd[tau]
    gamma = cmnd[tau + 1]
    peak_pos = tau + 0.5 * (alpha - gamma) / (alpha - 2*beta + gamma)
    peak_val = beta - 0.25 * (alpha - gamma) * (peak_pos - tau)
    return peak_pos, peak_val


def pyin(signal: np.ndarray, sample_rate: int, threshold: float = 0.1) -> Tuple[float, float]:
    """
    Implements the pYIN pitch detection algorithm.
    
    Args:
        signal: Input audio signal
        sample_rate: Sampling rate of the signal
        threshold: Threshold for pitch detection (default: 0.1)
    
    Returns:
        Tuple of (estimated frequency in Hz, confidence)
    """
    # Parameters
    tau_max = len(signal) // 3
    
    # Step 1: Compute difference function
    df = difference_function(signal, tau_max)
    
    # Step 2: Compute cumulative mean normalized difference
    cmnd = cumulative_mean_normalized_difference(df)
    
    # Step 3: Find local minima
    local_minima = []
    for i in range(1, len(cmnd) - 1):
        if cmnd[i] < cmnd[i-1] and cmnd[i] < cmnd[i+1]:
            if cmnd[i] < threshold:
                local_minima.append(i)
    
    if not local_minima:
        return 0.0, 0.0  # No pitch detected
    
    # Step 4: Pick the best period candidate
    best_period, best_value = parabolic_interpolation(cmnd, local_minima[0])
    
    # Convert period to frequency
    frequency = sample_rate / best_period if best_period > 0 else 0.0
    confidence = 1.0 - best_value
    
    return frequency, confidence

