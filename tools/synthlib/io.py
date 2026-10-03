"""synthlib.io -- WAV import/export helpers."""
import numpy as np
from scipy.io import wavfile

from .dsp import SR

def write_wav(path, mix, sr=SR):
    """mix: (n,) mono or (n, 2) stereo float array, clipped to [-1, 1]."""
    x = np.asarray(mix)
    if x.ndim == 1:
        x = np.stack([x, x], axis=1)
    x = np.clip(x, -1.0, 1.0)
    wavfile.write(str(path), sr, (x * 32767.0).astype(np.int16))
    return path

def read_wav(path):
    """Returns (sr, float32 (n, 2))."""
    sr, x = wavfile.read(str(path))
    x = x.astype(np.float32)
    if x.ndim == 1:
        x = np.stack([x, x], axis=1)
    return sr, x / 32767.0
