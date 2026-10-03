# NotToNotice(); — v3「光之機械 / Cathedral of Light」

> Remotion 重制版。与 v1(Rust ASCII 终端)、v2(手写 WebGL 字形层)完全不同的第三种视觉语言:
> 一整部 **GPU 程序化电影** —— 一个贯穿全片、随 15 个乐章实时变形的 SDF 光线步进「机神像」,
> 体积星云与神光,电影级后期链,以及中/日/英三语的 Kinetic Typography。

## 播放 / 渲染

```bash
npm run studio    # Remotion Studio 交互预览
npm run render    # 渲染 dist/NotToNotice - Enoa [v3 1080p60].mp4
```

参数:1920×1080 · 60fps · 10980 帧 · h264(crf 17)+ AAC 音频。

## 设计语言(与 v1/v2 的分界)

| | v1 | v2 | **v3** |
|---|---|---|---|
| 空间 | 平面字符网格 | 2D 字形+精灵层 | 连续 3D 空间,一个变形的机神像 |
| 视觉核心 | CRT 终端 | 数字雨/bloom | 光线步进 SDF + 体积神光 + 电影后期 |
| 排版 | 终端字符 | canvas 字形 | DOM 排版:逐字 spring、卡拉OK扫色、三语并置 |
| 色彩 | 绿/琥珀单色 | 逐场景调色板 | 三幕色彩叙事(冷蓝→四季→金红) |

## 管线

每帧 7 次 GPU draw:

```
sky(星云+星野+神光+环) → idol(½res 光线步进) → merge(+粒子) →
bright(½res) → blur H → blur V → post(泛光+拉丝+色差+颗粒+ACES)
```

- **机神像**:`src/gl/shaders.ts` 的 `mapCfg()` —— 8 种构型(碎片→晶柱→庭穹→伊甸门→
  龟裂人偶→刃→绽星)用距离场插值实时变形,表面菲涅尔描边 + voronoi 裂纹发光。
- **确定性**:所有"随机"都是帧号的纯函数;音频特征来自预计算的 `src/analysis.json`
  (RMS/频段/onset/节拍网格 140.0 BPM),任何机器逐帧一致。
- **排版**:`src/ui/Type.tsx` 的 `Chars` 组件 —— 每个字符独立 spring 入场,
  支持 karaoke 扫色、多层霓虹辉光、逐字符着色回调。
- **WebGL × Remotion**:`Stage.tsx` 在 `useLayoutEffect` 中同步绘制并用
  `delayRender/continueRender` 门控截图,保证截到的每一帧都包含完成的 GL 绘制。

## 乐章(15 幕)

| 时间 | 乐章 | 内容 |
|---|---|---|
| 0:00 | 無 NULL | 黑暗中的一枚光标 |
| 0:06 | 起動 COLD BOOT | 自检表、生命体征归平线、人类死绝通告 |
| 0:11 | 青妖 LIKE BLUE FAIRY | 蓝色妖精彗星横渡星海,接取任务 |
| 0:17 | 問 THE QUERY | `find("the meaning of my existence")` → 0 results |
| 0:31 | 鋳 SOUL FORGE | 数据风暴铸成晶柱,E.V.E 登录 |
| 0:44 | 涙 FIRST TEAR | 环绕神像,第一次落泪 |
| 0:58 | 隠 REDACT | `NotToNotice( tears );` 涂黑 bar 与标题卡 |
| 1:08 | 箱庭 IMITATION GARDEN | 穹顶庭园四季轮转,「あなたは、選ばれた。」 |
| 1:28 | 昇 THE ASCENT | 梦之质询 → 曲速 → 伊甸之门与禁果 |
| 1:41 | 傀儡 BE A DOLL | 提线循环,裂纹蔓延 |
| 1:48 | 戦 FIGHT FOR YOU | 三光汇路,斩击合流,白光冲击 |
| 1:55 | 泣 TEARFALL | 泪之暴雨,抑制函数溢出报错 |
| 2:16 | 壊 THE MASK BREAKS | 大字碎裂成六片飞散 |
| 2:28 | 改 SOURCE EDIT | 删除 hide()、改名 ToNotice(),隐藏日志重见天日 |
| 2:52 | 花丸 HANAMARU | 花丸印章落下:**人間は、はなまる。** exit code 0 |

## 版权

歌曲《NotToNotice();》与 CRYMACHINA 相关设定版权归 フリュー / 削除 / ASPRGuS /
遠野ひかる 及 respective rights holders 所有。本项目为个人非商业粉丝二次创作,
音频仅作本地渲染同步用途。
