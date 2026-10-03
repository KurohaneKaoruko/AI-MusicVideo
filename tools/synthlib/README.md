# synthlib

可复用的**纯程序合成**音频工具库 —— 不依赖任何项目，不做任何素材下载，
所有声音都来自 numpy/scipy 的数学运算。

## 内容

| 模块 | 说明 |
|---|---|
| `dsp` | 振荡器（saw/square/sine/tri，支持频率包络）、RBJ biquad 滤波器、时变低通（状态连续无zipper噪声）、FFT卷积混响（合成IR）、乒乓延迟、侧链闪避包络、软削波/归一化 |
| `instruments` | 合成乐器库：kick / snare / clap / hat / ride / sub / reese bass / supersaw pad / pluck / lead / FM bell / riser / impact / reverse cymbal / downlifter / blip / zap，均输出 `(L, R)` 立体声数组 |
| `io` | WAV 读写（16bit PCM ↔ float32） |

## 用法

```python
import numpy as np
from synthlib import dsp, io
from synthlib.instruments import kick, pad, lead

sr = dsp.SR
n = int(4.0 * sr)
L, R = np.zeros(n), np.zeros(n)

dsp.place(L, R, 0.0, kick(1.0))                      # 放置一个底鼓
dsp.place(L, R, 0.5, pad([220., 261.6, 329.6], 2.0)) # 和弦垫

mix = np.stack([L, R], axis=1)
mix = dsp.normalize(dsp.soft_clip(mix, 1.1))
io.write_wav("demo.wav", mix)
```

事件驱动的编曲示例见 `../../neural_overdrive/src/music/song.py`。
