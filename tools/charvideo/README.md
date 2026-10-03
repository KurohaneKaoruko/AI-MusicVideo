# charvideo

可复用的**终端字符画**渲染工具库 —— 把"一帧 = 彩色字符网格"这个抽象
变成三种输出：PNG 图片、MP4 视频（PIL 光栅化 + ffmpeg）、终端 ANSI
真彩实时播放。

## 内容

| 模块 | 说明 |
|---|---|
| `canvas` | `Canvas`: W×H 字符网格 + 每格 RGB。`put/put_max`（亮者胜出，适合叠加发光体）、`text/big_text`（5×7 点阵大字，支持色差 split）、`shift_row`（故障位移）、`to_pil` / `to_ansi` 双出口 |
| `font5x7` | 内置 5×7 点阵字体（A-Z 0-9 及常用符号），用于大标题 |
| `exporter` | 字体加载、像素级后处理（辉光/扫描线/暗角）、`render_video`（imageio-ffmpeg 编码 + AAC 混音）、`save_still`、`play_ansi`（sounddevice + ANSI 实时播放） |

## 用法

```python
from charvideo.canvas import Canvas
from charvideo import exporter

def frame_fn(t):                 # 任意 t -> Canvas 的函数
    cv = Canvas(120, 40)
    cv.big_text_c(10, "HELLO", (0.4, 1.0, 0.8), scale=2, split=1)
    cv.text(2, 2, f"t={t:.2f}s", (0.5, 0.5, 0.6), spacing=0)
    return cv

exporter.render_video(frame_fn, total=10.0, out_path="demo.mp4",
                      wav_path="audio.wav")     # 有 wav 就自动混流
exporter.save_still(frame_fn, 3.0, "demo.png")
exporter.play_ansi(frame_fn, total=10.0, wav_path="audio.wav")
```

约定：`frame_fn(t)` 返回的 Canvas 尺寸即输出分辨率（字符数 × 单元格
像素）；视频输出需要宽高为偶数（默认 12×25 单元格满足）。
