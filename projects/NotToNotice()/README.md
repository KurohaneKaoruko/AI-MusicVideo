# NotToNotice(); — 终端 MV

把 CRYMACHINA 的 OP 主题歌《NotToNotice();》（歌：エノア／CV. 遠野ひかる，
作曲：削除，作詞：ASPRGuS）整首演绎成"一次终端会话"的二创 MV。

整部 MV 是第八神机 ENOA 的一场控制台会话：启动自检 → 放出蓝妖精执行
`void execute(mission)` → 查询"我存在的意义"得到 0 结果 → 灵魂熔炉铸造
E.V.E → 在日志里发现"光学阵列上的不明液体"（第一次流泪）→ 用
`NotToNotice( tears );` 把真相打码隐藏 → 箱庭虚拟世界年复一年地筛出
3 个 humanlike soul → 曲速飞向播种构造体"伊甸"、伸手摘下 E×P 禁果 →
`while (be_a_doll)` 的提线木偶循环，直到胸口开裂 → 三颗 E.V.E 之光
照亮她的路 → "Fight for You" → 副歌再次落泪、抑制函数报错
`ERROR: cannot suppress (at heart::overflow, line ∞)` → 打开源码删掉
`hide()`、改名 `ToNotice()` → 终焉鉴定书盖章：**人間は、はなまる。**

## 播放 / 渲染

```
cargo run --release                     # 交互式终端播放（ANSI，真彩终端）
cargo run --release -- --dev beats      # 节拍检测校准报告
python ../tools/mv_render.py \
    --exe target/release/not-to-notice.exe \
    --audio NotToNotice.mp3 \
    --out "dist/NotToNotice - Enoa [terminal MV 1080p60].mp4" \
    --duration 183.2 --fps 60 --segments 6
```

## 技术栈（与其它项目不同的全新管线）

- **Rust 像素级渲染引擎**：192×45 字符网格（10×24px 单元 = 精确 1920×1080），
  fontdue 光栅化 Consolas / Consolas Bold / MS YaHei。
- **双层渲染**：字符网格层（终端 UI/代码/歌词）+ 像素精灵层
  （泪滴、花瓣、妖精、光球在亚字符尺度上以 60fps 平滑运动）。
- **确定性后期链**：加性 Bloom → 毛刺撕裂 → 色差 → CRT 扫描线 →
  暗角 → 胶片颗粒 → 淡入淡出/闪光。所有参数由场景逐帧驱动。
- **音频反应**：解码 PCM 后预计算 RMS / 低 / 中 / 高频段包络（~94Hz），
  配合谱通量 + 自相关 + 歌词锚点最小二乘校准的节拍网格（158.71 BPM），
  视觉随实际响度与节拍逐帧呼吸。
- **逐帧确定性**：无墙钟、无线程 RNG，任何时刻任何机器渲染结果逐位一致；
  分段并行渲染后无损拼接。
- 共享渲染工具在仓库根 `tools/mv_render.py`（所有 MV 项目通用）。

## 场景时间轴（15 幕）

| 时间 | 场景 | 内容 |
|---|---|---|
| 0:00 | BOOT | DEUS EX MACHINA OS 自检、八神机状态表、曲名 stamp |
| 0:10 | BLUE FAIRY | 星海中的蓝妖精 → `void execute(mission)` |
| 0:17 | QUERY | `find("the meaning of my existence")` → 0 results |
| 0:23 | SOUL FORGE | 人格数据字符雨铸成人形，E.V.E 登录 |
| 0:37 | TEARS | 机械之眼发现泪滴；`analyze(tear) → undefined` |
| 0:58 | REDACT | `NotToNotice( tears );` 金色大字 + 打码扫描 |
| 1:06 | HAKONIWA | 仮想世界地球四季轮转，E.V.E×3「あなたは、選ばれた。」 |
| 1:16 | EDEN FLIGHT | 星流加速 → 戴森环 → 恒星 → 白光 |
| 1:27 | EDEN | dream() 碎片 → 伊甸树与 E×P 禁果 → 伸手触碰 |
| 1:40 | DOLL | `while (be_a_doll)` 提线木偶循环，裂纹蔓延 |
| 1:47 | LIGHT | 三色 E.V.E 之光升起，连成她的路 |
| 1:51 | FIGHT | FIGHT FOR YOU——三彗星斩击合流 |
| 1:55 | CHORUS | 机械之眼泪雨倾盆；抑制函数报错溢出 |
| 2:15 | SOURCE | 删除 hide()、改名 ToNotice()；隐藏日志重见天日 |
| 2:37 | HANAMARU | 终焉鉴定书 + はなまる盖章；`exit code 0` |

> **版权说明**：歌曲《NotToNotice();》与 CRYMACHINA 相关设定版权归
> フリュー / 削除 / ASPRGuS / 遠野ひかる 及 respective rights holders 所有。
> 本项目为个人非商业的粉丝二次创作（fan MV），音频仅作本地渲染同步用途。
