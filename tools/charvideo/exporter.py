"""charvideo.exporter -- turn frame_fn(t) -> Canvas into MP4 or live ANSI.

frame_fn is any callable mapping time (seconds) to a charvideo Canvas.

Video pipeline:
    char grid -> fully-vectorized glyph-atlas compositing (numpy)
              -> float32 post (bloom / scanlines / vignette)
              -> NVENC GPU encode (if available) else libx264
    frame generation is parallelized across CPU cores (spawn pool).
"""
import os
import subprocess
import sys
import time

import numpy as np

FONT_CANDIDATES = [
    r"C:\Windows\Fonts\consola.ttf",
    r"C:\Windows\Fonts\cascmon.ttf",
    r"C:\Windows\Fonts\cour.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
    "/System/Library/Fonts/Menlo.ttc",
]

def find_font_path():
    for p in FONT_CANDIDATES:
        if os.path.exists(p):
            return p
    return None

# ---------------------------------------------------------- rasterizer ----
class Rasterizer:
    """CharCanvas -> PIL RGB frame via a glyph alpha atlas.

    The whole frame is composited with a handful of big vectorized numpy
    ops (no per-cell Python loops): unique-char indexing -> atlas gather
    -> one broadcast multiply into the output view.
    """

    def __init__(self, font_path, size, cell_w, cell_h):
        from PIL import ImageFont
        self.cw, self.ch = cell_w, cell_h
        if font_path:
            self.font = ImageFont.truetype(font_path, size)
        else:
            self.font = ImageFont.load_default()
        self.index = {' ': 0}
        self.alpha = [np.zeros((cell_h, cell_w), np.float32)]   # blank tile
        self._rebuild_bank()

    def _rebuild_bank(self):
        bank = np.stack(self.alpha)                  # (K, ch, cw)
        self.bank2 = np.ascontiguousarray(bank.transpose(1, 2, 0))  # (ch,cw,K)

    def _idx(self, ch):
        k = self.index.get(ch)
        if k is not None:
            return k
        from PIL import Image
        mask = self.font.getmask(ch, mode='L')
        tile = Image.new('L', (self.cw, self.ch), 0)
        if mask.size[0] > 0 and mask.size[1] > 0:
            m = Image.frombytes('L', mask.size, bytes(mask))
            tile.paste(m, (0, 0))
        a = np.asarray(tile, np.float32) / 255.0
        k = len(self.alpha)
        self.index[ch] = k
        self.alpha.append(a)
        self._rebuild_bank()
        return k

    def render(self, cv):
        from PIL import Image
        H, W = cv.h, cv.w
        cw, ch = self.cw, self.ch
        uniq, inv = np.unique(cv.ch, return_inverse=True)
        inv = inv.reshape(cv.ch.shape)
        aidx = np.fromiter((self._idx(str(c)) for c in uniq),
                           np.intp, len(uniq))
        idxm = aidx[inv]                              # (H, W)
        A2 = self.bank2[:, :, idxm]                   # (ch, cw, H, W)
        A2t = A2.transpose(2, 0, 3, 1)                # (H, ch, W, cw) view
        bgu = np.clip(cv.bg_row, 0, 1) * 255.0        # (H, 3)
        C = (cv.fg * 255.0 - bgu[:, None, :]).astype(np.float32)
        out = np.empty((H * ch, W * cw, 3), np.float32)
        out5 = out.reshape(H, ch, W, cw, 3)
        np.multiply(A2t[..., None], C[:, None, :, None, :], out=out5)
        out += np.repeat(bgu, ch, axis=0)[:, None, :]
        np.clip(out, 0, 255, out=out)
        return Image.fromarray(out.astype(np.uint8))

# ------------------------------------------------------- pixel post-FX ----
def make_vignette(w, h, strength=0.42):
    y, x = np.mgrid[0:h, 0:w]
    cx, cy = w / 2, h / 2
    d = np.sqrt(((x - cx) / cx) ** 2 + ((y - cy) / cy) ** 2)
    v = 1.0 - strength * np.clip(d, 0, 1) ** 2.2
    return np.ascontiguousarray(v[:, :, None], dtype=np.float32)

_GLOW_LUT = (np.clip(np.arange(256) * 0.55, 0, 255)).astype(np.uint8)

def post_frame(img, vignette=None, scanlines=True, glow=0.55):
    """img: PIL RGB -> processed PIL RGB (bloom + scanlines + vignette).
    All numpy float32, in-place where possible."""
    from PIL import Image, ImageChops, ImageFilter
    if glow > 0:
        w, h = img.size
        small = img.resize((max(w // 4, 1), max(h // 4, 1)), Image.BILINEAR)
        blur = small.filter(ImageFilter.GaussianBlur(3)) \
                    .resize((w, h), Image.BILINEAR)
        lut = _GLOW_LUT if glow == 0.55 else \
            np.clip(np.arange(256) * glow, 0, 255).astype(np.uint8)
        img = ImageChops.add(img, Image.fromarray(lut[np.asarray(blur, np.uint8)]))
    arr = np.asarray(img, np.float32)
    if scanlines:
        arr[::2] *= 0.84
    if vignette is not None:
        arr *= vignette
    np.clip(arr, 0, 255, out=arr)
    return Image.fromarray(arr.astype(np.uint8))

# ---------------------------------------------------------------- video ----
def detect_nvenc():
    try:
        import imageio_ffmpeg
        exe = imageio_ffmpeg.get_ffmpeg_exe()
    except Exception:
        return False
    try:
        r = subprocess.run([exe, '-hide_banner', '-encoders'],
                           capture_output=True, text=True, timeout=20)
        return 'h264_nvenc' in (r.stdout + r.stderr)
    except Exception:
        return False

def _get_writer(out_path, fps, nvenc):
    import imageio
    if nvenc:
        return imageio.get_writer(
            str(out_path), fps=fps, codec='h264_nvenc', quality=None,
            pixelformat='yuv420p', macro_block_size=1,
            ffmpeg_params=['-preset', 'p5', '-rc', 'vbr',
                           '-cq', '19', '-b:v', '0'])
    try:
        return imageio.get_writer(
            str(out_path), fps=fps, codec='libx264', quality=None,
            pixelformat='yuv420p', macro_block_size=1,
            ffmpeg_params=['-crf', '18', '-preset', 'faster'])
    except TypeError:
        return imageio.get_writer(str(out_path), fps=fps)

def _mux(video, wav, out):
    try:
        import imageio_ffmpeg
        exe = imageio_ffmpeg.get_ffmpeg_exe()
    except Exception:
        exe = 'ffmpeg'
    tmp = str(out) + '.tmp.mp4'
    os.replace(video, tmp)
    r = subprocess.run([exe, '-y', '-i', tmp, '-i', str(wav),
                        '-c:v', 'copy', '-c:a', 'aac', '-b:a', '192k',
                        '-shortest', str(out)],
                       capture_output=True, text=True)
    os.remove(tmp)
    if r.returncode != 0:
        raise RuntimeError('ffmpeg mux failed:\n' + r.stderr[-1500:])
    return out

# ------------------------------------------------------- worker support ----
_W = {}

def _worker_init(frame_fn, vig, cw, chh, font_size, glow, fps, t0):
    _W['frame_fn'] = frame_fn
    _W['vig'] = vig
    _W['glow'] = glow
    _W['fps'] = fps
    _W['t0'] = t0
    _W['ras'] = Rasterizer(find_font_path(), font_size, cw, chh)

def _worker_render(i):
    t = _W['t0'] + i / _W['fps']
    img = _W['ras'].render(_W['frame_fn'](t))
    img = post_frame(img, _W['vig'], scanlines=True, glow=_W['glow'])
    return np.asarray(img)

def render_video(frame_fn, total, out_path, wav_path=None,
                 fps=30, cell_w=12, cell_h=25, t0=0.0, t1=None,
                 font_size=22, glow=0.55, codec='auto', workers=None,
                 canvas_size=None, verbose=True):
    """Render [t0, t1) of the song to MP4.  frame_fn(t) -> Canvas.

    canvas_size: (w, h) in cells -- lets the worker pool precompute the
    vignette; if omitted, pool workers skip the vignette (serial mode
    always applies it)."""
    t1 = total if t1 is None else min(t1, total)
    if verbose:
        print(f'  font: {find_font_path() or "<default>"}')
    n_frames = max(int((t1 - t0) * fps), 1)

    if codec == 'auto':
        nvenc = detect_nvenc()
    else:
        nvenc = codec == 'nvenc'
    if verbose:
        print(f'  encoder: {"h264_nvenc (GPU)" if nvenc else "libx264 (CPU)"}')

    if workers is None:
        workers = max(1, min((os.cpu_count() or 2) - 1, 10))
    writer = _get_writer(out_path, fps, nvenc)
    t_start = time.perf_counter()

    def _report(i):
        if verbose and i % 100 == 0:
            el = time.perf_counter() - t_start
            eta = el / (i + 1) * (n_frames - i - 1)
            print(f'  frame {i + 1}/{n_frames}  ({(i + 1)/el:.1f} fps, '
                  f'eta {eta/60:.1f} min)   ', flush=True)

    try:
        if workers > 1:
            import multiprocessing as mp
            ctx = mp.get_context('spawn')
            vig = make_vignette(canvas_size[0] * cell_w,
                                canvas_size[1] * cell_h) if canvas_size else None
            with ctx.Pool(workers, initializer=_worker_init,
                          initargs=(frame_fn, vig, cell_w, cell_h,
                                    font_size, glow, fps, t0)) as pool:
                for i, arr in enumerate(pool.imap(_worker_render, range(n_frames),
                                                  chunksize=8)):
                    writer.append_data(arr)
                    _report(i)
        else:
            ras = Rasterizer(find_font_path(), font_size, cell_w, cell_h)
            probe = frame_fn(t0)
            vig = make_vignette(probe.w * cell_w, probe.h * cell_h)
            for i in range(n_frames):
                img = ras.render(frame_fn(t0 + i / fps))
                img = post_frame(img, vig, scanlines=True, glow=glow)
                writer.append_data(np.asarray(img))
                _report(i)
    finally:
        writer.close()
    if verbose:
        el = time.perf_counter() - t_start
        print(f'  encoded {out_path}  ({n_frames/el:.1f} fps avg)')
    if wav_path:
        _mux(out_path, wav_path, out_path)
        if verbose:
            print(f'  muxed audio -> {out_path}')
    return out_path

def save_still(frame_fn, t, out_path, cell_w=12, cell_h=25, font_size=22,
               glow=0.55):
    ras = Rasterizer(find_font_path(), font_size, cell_w, cell_h)
    img = ras.render(frame_fn(t))
    vig = make_vignette(img.size[0], img.size[1])
    post_frame(img, vig, scanlines=True, glow=glow).save(out_path)
    return out_path

# ------------------------------------------------------------ playback ----
def enable_ansi():
    if os.name == 'nt':
        os.system('')                      # enables VT processing in conhost
    try:
        sys.stdout.reconfigure(encoding='utf-8')
    except Exception:
        pass

def play_ansi(frame_fn, total, wav_path=None, fps=30, cols=None, rows=None,
              t0=0.0, mute=False, verbose=True):
    """Realtime playback in the terminal.  Returns the time reached."""
    enable_ansi()
    try:
        cols = cols or min(120, os.get_terminal_size().columns - 1)
        rows = rows or min(40, os.get_terminal_size().lines - 1)
    except Exception:
        cols, rows = 100, 34

    stream = None
    audio_t0 = time.perf_counter()
    if wav_path and not mute:
        try:
            import sounddevice as sd
            from synthlib.io import read_wav
            sr, data = read_wav(wav_path)
            cursor = [0]

            def callback(outdata, frames, time_info, status):
                i0 = cursor[0]
                i1 = min(i0 + frames, len(data))
                outdata[:] = 0
                if i1 > i0:
                    outdata[: i1 - i0] = data[i0:i1]
                cursor[0] = i1

            stream = sd.OutputStream(samplerate=sr, channels=2, dtype='float32',
                                     callback=callback)
            stream.start()
        except Exception as e:
            if verbose:
                print(f'[audio unavailable: {e}; playing silent]', file=sys.stderr)

    sys.stdout.write('\x1b[?25l\x1b[2J')
    t_end = t0 + (total - t0)
    try:
        i = 0
        while True:
            t = t0 + (time.perf_counter() - audio_t0)
            if t >= min(t_end, total - 0.05):
                break
            cv = frame_fn(t)
            sys.stdout.write(cv.to_ansi())
            sys.stdout.flush()
            i += 1
            nxt = audio_t0 + (i / fps)
            delay = nxt - time.perf_counter()
            if delay > 0:
                time.sleep(delay)
    except KeyboardInterrupt:
        pass
    finally:
        sys.stdout.write('\x1b[?25h\x1b[0m\n')
        sys.stdout.flush()
        if stream is not None:
            stream.stop(); stream.close()
    return time.perf_counter() - audio_t0
