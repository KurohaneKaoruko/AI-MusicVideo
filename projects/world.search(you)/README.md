# world.search (you) ; — 终端字符画 MV

把 Mili 的《world.search (you) ;》整首歌演绎成 **一次全世界的搜索程序的执行**。
整部片子就是这段程序的运行过程：在黑暗里醒来 → 索引全部记忆 → 逐个打开候选文件
（桌子 / 茄子 / 猫 / 牛排 / 花 / 人）→ 万物皆是你 → 丢掉了那个词 → 查询改写成别的
→ 把剩下的东西揉在一起进化 → 隔着回忆重放 → `exit code 0`。

```
cargo run --release --manifest-path v2/Cargo.toml
```

> 这一仓库从 v1 重做而来。v1 是「一个带少量特效的音乐播放器」，已归档到 `v1/`，仍可独立构建；
> `v2/` 是重新设计后的版本 —— 见 [版本](#版本v1--v2)。

---

## 它是什么样的

- **每一幕都重新构图**，不共用骨架。整片 18 幕各自拥有自己的环境场、自己的相机运动、
  自己的阅读路径：冷开场只有一枚光标；索引幕是 44 根记忆条的实时冒泡排序；雷达幕是
  同心扫描环；桥段是一条单点透视的长走廊，尽头站着一个人；副歌是 26 个样本漂在星河里。
- **双层渲染**：字符网格层（192×45，10×24px 单元 = 精确 1920×1080）负责这个世界自己的
  界面 —— 查询行、属性面板、取景框、印章；像素层负责环境（星河 / 雨 / 尘埃 / 地板透视 /
  天空线）、位图精灵、虚拟相机和整条后期链。两层共用同一份场景代码。
- **真·节奏同步**：启动时离线解码整首 MP3，谱通量 + 自相关检测节拍，再用歌词锚点做
  最小二乘校准出节拍网格；视觉脉冲、场景动画全部查表对齐。
- **确定性渲染**：没有墙钟、没有线程 RNG。任意时刻的成像逐位一致，所以离线渲染与实时
  播放看到的是同一帧。
- **后期链**：bloom（加性辉光）→ glitch（毛刺撕裂）→ chromatic aberration（色差）→
  scanlines（扫描线）→ vignette（暗角）→ grain（胶片颗粒）→ fade / flash。
  相机在「世界画完、界面与歌词画上之前」应用，因此读数与歌词始终清晰不抖。

## 在终端里播放

### 1. 运行条件

窗口必须至少 **192 列 × 45 行**（片子固定画在 192×45 的字符网格上，10×24px 单元 = 1920×1080）。
不够就先把窗口最大化，再 `Ctrl` + `-` 缩字号。

必须是真实的 Windows 控制台：**Windows Terminal**（或 PowerShell / cmd）。Git Bash / mintty
跑不起来，会提示改用 `winpty`。

窗口不够大时程序**不开演**，而是停在这样一张提示卡上（按 `Q` 退出）：

```
        terminal too small
this film is a fixed 192 x 45 character grid
window now: 155 x 41
maximise the window, or shrink the font
need at least 192 columns by 45 rows
              [q] quit
```

这张卡只在窗口尺寸变化时重绘。播放中途把窗口缩小也会切回这张卡，音频时钟继续走，
窗口拉回来就接着放。

### 2. 播放

```
# 项目根目录
cargo run --release --manifest-path v2/Cargo.toml
```

音频默认自动从 `reference/` 里找那首 `.mp3`，所以在根目录或 `v2/` 下直接跑都可以。
启动时要离线解码整首歌（约 1.7s），这段时间屏幕上会显示 `indexing the world… x%`；
解码一完成就开始播放，节拍分析在后台并行跑完。

节拍分析结果会缓存到音频文件旁边（`reference/*.beatcache`，已被 gitignore）：
首次约 1.2s，之后基本为零；与当前音频对不上会自动重算。

```
# 常用变体
cargo run --release --manifest-path v2/Cargo.toml -- --seek 200      # 从 3:20 开始
cargo run --release --manifest-path v2/Cargo.toml -- --no-audio      # 静音（走墙钟）
cargo run --release --manifest-path v2/Cargo.toml -- --fps 30        # 限 30fps（省电）
cargo run --release --manifest-path v2/Cargo.toml -- -a "路径/歌.mp3"
```

播放结束时回到原终端并打印一行收尾语；中途 `Q` / `ESC` 直接退出，不会留下乱掉的终端。

### 3. 键盘

| 键 | 功能 |
|---|---|
| `SPACE` | 暂停 / 继续 |
| `←` / `→` | 快退 / 快进 5s |
| `+` / `-` | 音量 |
| `F` | 帧率上限：关 → 60 → 30 循环（右上角显示当前值） |
| `H` | 帮助浮层 |
| `Q` / `ESC` | 退出 |

### 选项

```
-a, --audio <路径>    指定音频文件
    --no-audio        静音播放（时间轴走墙钟）
    --seek <秒>       从指定位置开始
    --fps <数字>      帧率上限，0 = 不限（默认 0）
-c, --capture <列表>  离屏成像若干时间点到 .preview/*.png（完整像素管线）
                      写法："12,45.5,90" | 区间 "a..b" | 网格 "a:b:step"
    --video <规格>    逐帧离线渲染 "start:end:fps"（确定性）
    --out <文件>      mp4 输出路径（ffmpeg 走 MV_FFMPEG 或 PATH）
    --geometry        打印渲染几何（192 45 10 24 1920 1080）后退出
    --bench <数字>    跑 N 帧完整管线，并分别报告终端帧在"像素层开/关"下的耗时
    --dev beats|audio|sheet|small  节拍自检 / 音频设备自检 / 精灵图总览 /
                      窗口过小时的提示卡预览
```

> 终端里播放的是**字符网格层**：环境与精灵的像素层只在离屏成像与视频里完整呈现，
> 这符合"一段程序在终端里跑"的设定。要看完整画面请走 `--video` 或直接看成品 mp4。
>
> 因此播放时像素层不参与绘制 —— 场景仍走同一份代码。离屏成像与 `--video` 不受影响。

## 场景时间轴（18 幕）

| 时间 | 幕 | 内容 |
|---|---|---|
| 0:00 | `world.search --target you` | 冷开场：一枚光标 → 查询被逐字打进虚空 → 取景光圈从 "you" 上绽开 → 启动日志 → 光圈收成一点 |
| 0:14 | `memory.sort` | 44 根记忆条实时冒泡排序；唱到 "till the bubble pops" 时泡泡真的炸开；存进 `past/` |
| 0:41 | `you.table` | 房间侧视：桌子被"组装"出来，属性面板逐项打勾，唱到"完美"盖下 PERFECT 印章 |
| 0:54 | `you.eggplant` | 厨房横带：紫色的朋友，摇摆、指甲划痕逐拍出现、一张脸 |
| 1:08 | `searching` | 雷达同心环 + 扫描臂 + 一颗反复被找到的心 |
| 1:23 | `you.cat` | 夜里空房间里一只蓝重点色小猫，窗光洒进来 |
| 1:37 | `you.steak` | 俯视热锅里的菲力，切面扫描（55°C 三分熟） |
| 1:50 | `you.flower` | 花从花苞长满整个画面高度：偶尔才开 → 盛放 + 花瓣雨 |
| 2:03 | `you.human` | 双手把画面捧在中间；乱发；臭臭的呼吸；一顶被推开的 PERFECT 印章 |
| 2:17 | `searching (unstable)` | 抖动、色差拉满的故障扫描 |
| 2:32 | `u-um…` | 单点透视的长走廊，尽头一个身影；问号之雨；候选词一个个被划掉；`****` 打码；`"Do-o-o-o-pe"?` 物理拉长；最后 LOVE 落地照亮整条走廊 |
| 2:59 | `everywhere` | 26 个样本漂在星河里不断变形；一条读取光束扫过它们；查询改写成别的 → `0 results` |
| 3:27 | `merge / evolve` | 爱之桶（90%/25%/60%/10%）→ 切成细胞场 → 重组进化出嵌合体 |
| 3:54 | `memory: flower` | 隔着胶片颗粒与雨窗，重放那朵花 |
| 4:07 | `memory: human` | 同一双手的温度，隔着一层回忆 |
| 4:21 | `searching (dim)` | 调暗的最后一次扫描 |
| 4:36 | `you.dog / you.tomato` | 暖到过曝的午后：小奶狗跑进来、停下、摇尾巴 `ruff!` → 番茄登场，灌满果汁 |
| 4:50 | `exit code 0` | 回到开场那片虚空（但现在是暖的）：`0 results for "you"` → `∞ results for "warmth"` → 一颗缓慢跳动的心 → `exit code 0` / `— fin —` |

## 项目结构

```
.
├── v2/                    ← 当前版本
│   ├── src/
│   │   ├── main.rs        启动 / 主循环 / 离线成像 / 视频编码 / CLI
│   │   ├── audio.rs       symphonia 解码 + cpal 播放 + 主时钟 + 频段包络
│   │   ├── beats.rs       节拍检测（谱通量 + 自相关 + 歌词锚点校准）
│   │   ├── gfx.rs         字符网格：Cell/Grid/调色板 + 缓动 + 确定性 Rng
│   │   ├── pix.rs         像素画布：图元 / 位图 / 文字栅格化 / 后期链
│   │   ├── cam.rs         虚拟相机（平移 / 推拉 / 抖动，双线性重采样）
│   │   ├── env.rs         环境场（星河 / 雨 / 尘埃 / 地板透视 / 天空线 / 光柱…）
│   │   ├── art.rs         位图精灵库（桌子 / 茄子 / 猫 / 牛排 / 花 / 手 / 心…）
│   │   ├── lyrics.rs      中英双语 LRC 数据 + 按幕排版的双模式渲染器
│   │   ├── hud.rs         叙事内的"搜索程序读数"（进度细线 / 查询 / 时间）
│   │   ├── capture.rs     一帧走完整管线的离线成像 + 精灵图
│   │   ├── win_pipe.rs    Windows 上给 ffmpeg 用的 CreatePipe 匿名管道
│   │   ├── term.rs        raw mode / 备用屏 / 输入
│   │   └── scenes/        18 幕（每幕一个文件）+ 时间轴 + 相机表 + 共享元件
│   └── tools/
│       ├── sheet.py       联络表（逐幕构图校验）
│       ├── row.py         单行拼图（同一幕的时序对比）
│       └── grab.py        从成品 mp4 抽帧核对
├── v1/                    ← 旧版（归档，可独立构建）
├── reference/             ← 歌曲音频 + 原版 LRC（随仓库分发，见 LICENSE）
└── LICENSE
```

## 开发工具

离屏成像与实时播放共用同一份场景代码：

```
# 出联络表：任选若干时间点，直接成像
cargo run --release --manifest-path v2/Cargo.toml -- \
    --capture "5,20,47,75,115,160,200,240,282,293"
python v2/tools/sheet.py v2/.preview v2/.preview/sheet.png 4

cargo run --release --manifest-path v2/Cargo.toml -- --bench 60      # 渲染性能基准
cargo run --release --manifest-path v2/Cargo.toml -- --dev sheet     # 精灵图总览
cargo run --release --manifest-path v2/Cargo.toml -- --dev small     # 窗口过小时的提示卡
```

## 渲染成视频

整首歌逐帧离线渲染（严格 60fps、确定性输出，与实时播放共用同一渲染管线），
多路并行分段编码后无损拼接，混入歌曲音频，输出 1080p60 H.264：

```
# 公用工具在上一层目录，与同工作区其它 MV 项目共用
python ../tools/mv_render.py \
    --exe v2/target/release/world-search-you.exe \
    --audio "reference/world.search (you) ; - Mili.mp3" \
    --out "v2/dist/world.search (you) ; - Mili [terminal MV v2 1080p60].mp4" \
    --duration 296 --fps 60 --segments 8
```

`mv_render.py` 与渲染器之间的接口契约（任意 MV 项目通用）：

| 参数 | 含义 |
|---|---|
| `--geometry` | 打印 `cols rows cell_w cell_h W H`，用于确定输出分辨率 |
| `--video START:END:FPS --out F` | 渲染 `[START, END)` 区间为 mp4 |

渲染器自身通过 `MV_FFMPEG` 环境变量或 PATH 寻找 ffmpeg。

## 版本：v1 / v2

| | v1（`v1/`） | v2（`v2/`） |
|---|---|---|
| 定位 | 常驻"播放器"界面 + 中间一个小画框 | 每一幕独立构图的全幅叙事 |
| 渲染 | 字符网格 + 少量像素特效 | 字符网格层 + 像素层（相机 / 环境 / 位图 / 后期链） |
| 界面 | 状态栏 / 进度条 / 节拍点常驻 | 只在叙事内出现"搜索程序"自己的读数 |
| 结局 | 一颗心 | 心 + `exit code 0`，并带完整淡出 |
| 窗口 | 任意尺寸 | 固定 192×45，不够就停在提示卡（见 [运行条件](#1-运行条件)） |

v1 保留在 `v1/` 目录中，可独立 `cargo run`。

> **版权说明**：仓库内的音频、歌词等素材版权归 **Mili** 及其权利人所有，
> 仅为个人非商业学习用途随仓库分发；详见 [LICENSE](LICENSE)。
> 如版权方提出要求，将立即移除。
