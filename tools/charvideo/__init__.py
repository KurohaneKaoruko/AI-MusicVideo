"""charvideo -- reusable terminal-character graphics toolkit.

Render frames as a grid of colored characters, then either:
  * rasterize to PNG / encode to MP4 (via Pillow + imageio-ffmpeg), or
  * play live in the terminal with ANSI truecolor escape codes.

Project-independent: takes any `frame_fn(t) -> Canvas`.
"""
from . import font5x7, canvas, exporter

__all__ = ['font5x7', 'canvas', 'exporter']
