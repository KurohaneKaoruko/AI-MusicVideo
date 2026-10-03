"""synthlib -- reusable pure-synthesis audio toolkit.

Everything here is project-independent: oscillators, biquad filters,
envelopes, reverb/delay/sidechain effects, a synthesized instrument
library and WAV import/export.  No samples, no external assets --
sound comes entirely from numpy math.

Quick start:
    from synthlib import dsp, io
    from synthlib.instruments import kick, pad, lead
    n = int(2.0 * dsp.SR)
    L, R = np.zeros(n), np.zeros(n)
    dsp.place(L, R, 0.0, kick(1.0))
    io.write_wav("out.wav", np.stack([L, R], 1))
"""
from . import dsp, io, instruments

__all__ = ['dsp', 'io', 'instruments']
