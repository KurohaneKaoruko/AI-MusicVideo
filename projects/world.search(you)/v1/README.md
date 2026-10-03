# world.search (you) ; — 终端 MV

一个把 Mili 的《world.search (you) ;》整首歌演绎成"一次全世界的搜索程序"的 Rust CLI 播放器。
整部 MV 就是一次检索的执行：启动自检 → 比对记忆 → 逐个打开候选文件 → 万物皆是你 →
查询改写 → 进化重组 → 记忆回放 → `exit code 0`。

```
$ cargo run --release
```

## 它是什么样的

- **常驻"程序"界面**：状态栏 / 分段进度条（按场景着色）/ 节拍圆点 / 中英双语歌词条，
  中间舞台由 18 个场景按歌词时间轴切换。
- **真·节奏同步**：启动时离线解码整首 MP3，谱通量 + 自相关检测节奏，
  再用歌词锚点做最小二乘校准出节拍网格（结果缓存在音频旁）；
  视觉脉冲、场景内动画全部查表对齐。
- **meta 叙事**：整首歌被处理成一次 `world.search(you);` 的执行——
  每个候选都是一个文件：`world/you.table`、`world/you.eggplant`、`world/you.cat`、
  `world/you.steak`……属性面板逐项打勾，唱到"完美"盖下大印章；
  副歌时查询改写成 `world.search(my old self)` → `0 results`，
  `ls world/` 里所有 `you.*` 全部 `[NOT FOUND]`。
- **歌词特效**：逐字显现 + 卡拉OK扫光；副歌"searching for"带回声残影；
  桥段 `a piece of ****` 是真正的打码闪烁，`"Do-o-o-o-ope"?` 的字母会物理拉伸；
  "old self" 那句用红蓝色差+颤抖呈现。
- **结尾**：`world.search(you);` → `0 results for "you"` / `∞ results for "warmth"`，
  一颗缓慢跳动的心。

## 运行

需要真彩终端（Windows Terminal / iTerm2 / 大多数现代终端），建议 ≥ 100×30。

```
cargo run --release
```

音频默认从 `reference/` 读取任意 `.mp3`（本仓库自带歌曲与歌词参考）。

> **版权说明**：仓库内的音频、歌词等素材版权归 **Mili** 及其 respective rights holders
> 所有，仅为个人非商业学习用途随仓库分发；详见 [LICENSE](LICENSE)。
> 如版权方提出要求，将立即移除。

### 选项

```
-a, --audio <路径>   指定音频文件
    --no-audio       静音播放（时间轴走墙钟）
    --seek <秒>      从指定位置开始
    --fps <数字>     渲染帧率上限
-c, --capture <列表> 离屏渲染若干时间点（开发校验）
    --cols/--rows    离屏渲染尺寸（默认 120x36）
    --video <规格>   逐帧流式输出（渲染视频用）
    --dev beats/audio 节拍检测校验 / 音频设备自检
```

### 按键

| 键 | 功能 |
|---|---|
| `SPACE` | 暂停 / 继续 |
| `←` / `→` | 快退 / 快进 5s |
| `+` / `-` | 音量 |
| `F` | 帧率显示 |
| `H` | 帮助浮层 |
| `Q` / `ESC` | 退出 |

## 场景时间轴（18 幕）

| 时间 | 场景 | 内容 |
|---|---|---|
| 0:00 | BOOT | 光标独白 → 作曲署名 → 像素大字标题逐列显现 → 启动日志 |
| 0:14 | memory.sort | 记忆卡片冒泡排序；泡泡升起，"till the bubble pops" 破裂；存入 past/ |
| 0:40 | you.table | 桌子组装；手肘高度标尺；短腿摇晃；PERFECT 印章 |
| 0:54 | you.eggplant | 紫色朋友：摇摆、指甲划痕逐拍出现、脸 |
| 1:08 | searching | 雷达扫描臂 + 光点 + 一颗反复被找到的心 |
| 1:22 | you.cat | 蓝重点色小猫：meow / sss!! / `purr: NOT FOUND` |
| 1:36 | you.steak | 菲力牛排滋滋冒汽；切面检查面板（55°C 三分熟） |
| 1:49 | you.flower | 花苞 → 绽放 → "偶尔才开"开合 → 盛放 + 花瓣雨 |
| 2:03 | you.human | 双手温度粒子；乱发；臭臭的呼吸；PERFECT |
| 2:17 | searching | 更急的扫描 + 故障切片 |
| 2:31 | u-um… | 问号之雨；候选词逐个划掉；桌面溶解；远处的身影；LOVE 大字落地 |
| 2:59 | everywhere | 万物漂浮且不断变形；搜索光束点亮它们；查询改写为 `my old self` → 0 results |
| 3:27 | merge/evolve | 爱之桶（90%/25%/60%/10%）→ 切成细胞 → 重组进化出嵌合体 |
| 3:53 | memory: flower | 花朵在棕褐色记忆窗口里重放（胶片颗粒） |
| 4:07 | memory: human | 同一双手的温度，隔着一层回忆 |
| 4:21 | searching (dim) | 调暗的最后一次扫描 |
| 4:35 | outro | 小奶狗摇尾巴 ruff! → 番茄灌满果汁 |
| 4:50 | — fin — | `world.search(you);` → `0 results / ∞ results` → 一颗心 |

## 项目结构

```
world.search(you);/         ← 仓库根 = cargo 项目根
├── src/                    ← 源码（scenes/ 下按叙事幕分文件）
│   ├── main.rs             启动 / 主循环 / 离屏渲染 / 视频流输出
│   ├── audio.rs            symphonia 解码 + cpal 播放 + 主时钟
│   ├── beats.rs            节拍检测（谱通量 + 自相关 + 歌词锚点校准）
│   ├── gfx.rs              字符画布：Cell/Grid/图元 + ANSI 序列化
│   ├── art.rs              5 行像素字模 + 全部字符画精灵
│   ├── lyrics.rs           中英双语歌词数据 + 歌词渲染器
│   ├── fx.rs               转场与特效（wipe/glitch/CRT/curtain/grain）
│   ├── hud.rs              常驻界面：状态栏/进度条/帮助
│   ├── term.rs             raw mode / 备用屏 / 输入
│   ├── capture.rs          离屏帧导出（HTML / PNG / 视频流）
│   └── scenes/             18 个场景 + 时间轴 + 共享 UI 组件
├── reference/              ← 歌曲音频 + 原版 LRC（随仓库分发，见 LICENSE）
├── tools/                  ← 帧转 PNG / 视频合成脚本（Python）
├── dist/                   ← 渲染成品（gitignore）
├── Cargo.toml
└── README.md
```

## 开发工具

离屏渲染与实时播放共用同一份场景代码，从任意时间点直接成像：

```
cargo run --release -- --capture 13,108,203 --cols 110 --rows 34   # → .preview/*.html/.txt
cargo run --release -- --dev beats                                 # 节拍检测 vs 歌词锚点
cargo run --release -- --dev audio                                 # 音频设备自检
cargo run --release -- --bench 400                                 # 渲染性能基准
python tools/frames_to_png.py <名字> .preview/png 6                # 帧转 PNG（需 Pillow）
```

## 渲染成视频

整首歌逐帧离线渲染（严格 60fps、确定性输出，与实时播放共用同一渲染管线），
6 路并行分段编码后无损拼接，混入歌曲音频，输出 1080p60 H.264：

```
pip install pillow imageio-ffmpeg
python tools/render_full.py     # → dist/world.search (you) ; - Mili [terminal MV 1080p60].mp4
```
