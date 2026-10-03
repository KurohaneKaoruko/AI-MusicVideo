# AI-MusicVideo

用代码合成音乐、再把整条时间轴渲染成终端风 MV 的实验场。
本仓库为单一仓库：共享工具库 + 全部作品，统一在根目录做版本管理。

## 目录结构

```
AI-MusicVideo/
├── tools/                 共享工具库（跨作品复用）
│   ├── synthlib/          音乐合成工具库（音源/节拍/效果器/混音/WAV）
│   ├── charvideo/         终端字符动画渲染库（画布/5x7字体/导出 PNG、MP4、ANSI）
│   ├── mv_render.py       通用分段并行渲染脚本（供各作品的渲染器 exe 使用）
│   └── mv_verify.py       成片校验脚本（探测流信息 + 抽帧）
└── projects/              作品目录（每个作品一个子目录）
    ├── NotToNotice()/     作品：NotToNotice (Enoa)，v1 Rust / v2 / v3 Remotion
    ├── world.execute(me)/ 作品：world.execute(me); (Mili)，Rust，维护期
    ├── world.search(you)/ 作品：world.search (you); (Mili)，v1 / v2 Rust
    └── ...                 新作品按相同约定新建目录
```

## 作品目录约定

每个作品目录内部结构统一：

```
projects/<name>/
├── README.md        作品说明
├── reference/       音源（mp3）、歌词（lrc）等原始素材
├── v1/ v2/ v3/      各版本源码（自包含：Cargo / Remotion 工程各自独立）
└── dist/            成片输出（各版本最终 MP4 集中放在这里；体积大，不入库）
```

- 单版本维护的作品（如 world.execute(me);）源码直接放在作品根目录，不设版本子目录。
- `dist/`、构建产物（`target/`、`node_modules/`）、预览缓存（`.preview/` 等）一律不入库，
  详见根目录 [.gitignore](.gitignore)。
- 便携版 ffmpeg 不入库，渲染前自行放置（作品内 `tools/ffmpeg/` 或 `PATH` 均可）。

## 新增作品

新建 `projects/<name>/`，按上面的目录约定放置素材与源码即可；
音乐合成可复用 `tools/synthlib`，字符渲染可复用 `tools/charvideo`。

- 工具库用法见 [tools/synthlib](tools/synthlib/README.md) 与
  [tools/charvideo](tools/charvideo/README.md)。
