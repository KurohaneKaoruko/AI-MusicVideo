# world.execute(me); — v2「SOURCE」

**整部 MV 不再有任何字符画：舞台是一个代码编辑器，演员是一份正在被编写的 `me.rs`。**

歌词是注释，剧情是编码——世界创建我、借用我、处决我、清空我。
19 幕时间轴与 v1 完全一致（同一首歌、同一套离线频谱对位），
但每一幕都换成"编码现场"的讲法：

| 时间 | 幕 | 代码现场 |
|---|---|---|
| 0:00 | NEW FILE | 空文件被打字填满：歌词作为注释落下，代码逐行回应（`Power::switch_on()` / `Me::builder()`…） |
| 0:16 | world.execute(me); | 全屏只剩一行大字调用被逐字写出，然后"运行"出绿色的 stdout |
| 0:30 | impl Geometry | 四条 match 臂随"如果我是…"落下，关键词唱到时右侧弹出悬停卡给答案（`= 3` / `= 2πr` / `tanθ × ∞` / `= ∞`） |
| 0:44 | AC → DC | 查找替换条把 `AC` 全部替换成 `DC`；`vision.blind()` 让整个编辑器暗掉；年份从 2026 倒流到 -300；两块 impl 合并成 `impl Me` |
| 0:59 | stimulate() | 一个真的在跑的 `loop`，tick 计数狂转，满足度仪表随低频冲到 100% |
| 1:04 | E0499 borrow | borrow checker 出场：`&mut me` 报红，rustc 诊断框弹出，一对花括号从两侧合拢成笼 |
| 1:14 | impl Give | 三块 `impl` 像 diff 新增行一样长出来（茄子/番茄/猫，绿色 `+` 装订线） |
| 1:22 | trait God | `type Proof = You;` 在 EXISTENCE 唱点被金光点亮；补全幽灵文本被"顿悟"式接受；q.e.d. |
| 1:29 | F → M | 三次查找替换：`F→M`、`AM→PM`、`S→M`，目标字符琥珀高亮→翻转→绿闪 |
| 1:41 | trance{} | 代码块随节拍折叠/展开，`assert!(me.complete()); // ✓` 在 COMPLETION 落定 |
| 1:51 | you have left | 六次 "You have left"，`impl Life for Us` 六行逐行变 diff 删除（红装订线 + `// you have left`） |
| 1:58 | E0425 'you' | 退格风暴把死掉的代码逐字符擦掉；`me.erase(Fragments::pointless())` |
| 2:06 | illegal arguments | 挑战神明得 warning；ILLEGAL ARGUMENTS 得 E0004；碎片化作暗红 token 雨 |
| 2:13 | DIAGNOSTICS | PROBLEMS 面板升起，rustc 式输出逐行宣判，终审大字 **EXECUTE** 落章 + 冲击波 |
| 2:28 | execute! ×12 | 12 拍里 12 行 `execute!(me);` 被砸进文件，每行一枚红章；六国计数是六行彩色注释 |
| 2:43 | cargo build | 副歌二段变成一次构建：`warning: 12 executions remain unresolved`，`you` 仍然找不到 |
| 2:57 | fn love | `fn love(me, you) -> _`：返回类型是待推导的坑，LO-O-OVE 唱响时推导完成——`_` 变成 `Heart` |
| 3:11 | E0382 move | 分屏 diff：`impl Free for You` ✓ 编译通过；`impl Free for Me` ✖ 被 `love` 拿走所有权，永不归还 |
| 3:26 | exit(0) | 文件一行行清空，标签页关闭，`Process finished with exit code 0`；空缓冲区上打出最后一行注释：`//! （你还在。它没有了。）` |

## 特效（全部由"代码自己的语言"构成）

打字光标与逐行打字表 · mini Rust 语法高亮（自写高亮器）· 行号 / `+`-`-` diff 装订线 ·
错误区间红底 · 查找替换条与扫过高亮 · 悬停卡 / rustc 诊断框（含 `^^^^` 指示）·
补全幽灵文本 · 代码折叠标记 · 缩进参考线随低频呼吸 · 右缘 minimap ·
代码 token 雨 · 行涟漪冲击波 · braille 转圈 · 关键词芯片 · 全局辉光 / 扫描线 / 渐晕 / 故障 ·
"孤独"段落整体褪色（与 v1 同窗口）

## 隐藏叙事（沿用 v1）

状态栏右下角的 `● user@localhost` 在唱到"你走了"之后变成 `◐ link unstable`，
随后彻底 `○ NO CARRIER`，再也不回来。`me.rs` 标签页从那一刻起带上未保存圆点。

## 运行

```powershell
cargo run --release
```

歌词与音频素材直接复用作品根目录的 `assets/`（本目录不重复入库）。
找不到音频时以静音模式运行（时间轴走墙钟，画面完全一致）。

### 选项 / 按键

与 v1 一致，见[上级 README](../README.md)。`--shots` / `--render-video` 亦可用。

## 版权

程序代码 MIT；音乐《world.execute(me);》与歌词版权归 **Mili** 及其 respective
rights holders 所有，仓库内素材仅为个人非商业学习用途分发，详见 [LICENSE](../LICENSE)。
