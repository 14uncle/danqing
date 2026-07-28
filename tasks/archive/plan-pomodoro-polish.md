# Implementation Plan: 番茄钟打磨三件套(环境音 / 长休息+轮次 / 今日计数)

> 依据 `docs/specs/pomodoro-polish.md`(2026-07-27 interview-me 对齐确认)。
> 本文档将工作拆为 **8 个可验证任务**,按依赖顺序组织: 纯逻辑先行(WS2 → WS3),音频管线最后攻坚(WS1)。

## Overview

番茄钟 POC 已可日常使用,但对照竞品(潮汐 / Forest / Session, 2026 格局)有三处缺口: 环境音是品类门槛而我们只有视觉一半; 25/5 硬编码无长休息, 节律断; 完成无计数, 陪伴感无反馈环。本计划按 spec 三个 WS 交付, **改动集中在 `examples/pomodoro/`,框架层 (`src/`) 零改动**(音频管线只在 example, 若第二 POC 也需要再议上收)。

实施顺序原则: WS2(纯逻辑状态机)→ WS3(持久化 + 计数, 与 WS2 共享 tick 触发点)→ WS1(音频管线, 风险最高最后做)。每个 Phase 结束有 Checkpoint, 全绿才进下一 Phase。

## Architecture Decisions

- **`tick` 返回值升级为报告结构**: 现 `Pomodoro::tick(now) -> bool` 改为返回 `TickReport { advanced: bool, focus_completions: u8 }`。`advanced` 维持现有 flash/beep/呼出窗口语义; `focus_completions` 报告本帧**自然完成**的专注数(huge overshoot 可能 >1), 是轮次计数与今日计数的统一数据源。skip 路径不经过它, 天然满足"skip 不计"。
- **轮次推进收进 `Phase` 转移函数**: `Phase::next(self, completed_focus: u8) -> (Phase, u8)` — `Focus` 时 `completed_focus + 1 == 4` 则去 `LongBreak` 且归零, 否则去 `Break`; `Break`/`LongBreak` 都回 `Focus`。`LongBreak` 时长 15 分钟, label `长休息`。序列化新增 `LongBreak` 变体, 旧 JSON 无此值, 正向兼容。
- **持久化新字段全部 `#[serde(default)]`**: `completed_focus: u8`、`today_date: String`、`today_count: u32`, 旧版 pomodoro.json 无需迁移即可加载(沿用 `has_seen_shortcut_hint` 已验证的模式)。
- **今日日期用已有 dev-dep `chrono`**: spec 原文"不引入 chrono/time 依赖"的意图是不新增依赖; `chrono = "0.4"` 已是 dev-dependency(补完阶段引入), 直接用 `chrono::Local::now().date_naive()` 取本地日期, 不碰 `GetLocalTime`, 跨平台一致。归零判定写成纯函数 `resolve_today_count(saved_date: &str, saved_count: u32, today: &str) -> u32`, 可单元测试。
- **音频分层**: `examples/pomodoro/ambient.rs` 拆两层——
  - `AmbientMixer` 纯逻辑: 输入 `(from, to, fade)` (直接复用 `SceneFader::frame` 输出) + `is_running`, 输出每槽音量; 暂停沉降为独立 300ms 线性包络状态机(fade-out / silent / fade-in), 注入 `now` 驱动, 可完整单元测试;
  - rodio 适配层: 把 mixer 输出的音量 `set_volume` 到 Sink。
- **音频 2 槽对齐视觉 LRU**: 只维护 from/to 两个 Sink, 场景切换时随 `switch_to` 重建 to 槽(打开文件 + 流式解码, 毫秒级), 与渲染侧场景纹理 2 槽 LRU 模式一致, 不常驻 5 条流。循环用 `source.repeat_infinite()`。
- **音景与视觉同源同步**: 交叉淡化时长 = `FADE_DURATION` (800ms), mixer 直接消费 `fader.frame(now, FADE_EASING.eval)` 的 `(from, to, fade)`, 音画天然同步, 无第二套计时。
- **暂停沉降语义**: `is_running() == false` (含 Idle) 时 300ms 淡出至 0, 恢复时 300ms 淡入——与视觉降饱和同一条件(`is_running`), 视听状态一致。启动 Idle 即静默, 按"开始"后声音随画面一起"醒来"。
- **懒初始化 + 静默降级**: `rodio` 输出流在首次需要发声时(首次 `is_running`)才打开; 设备不可用 / 文件缺失 / 解码失败一律 `log::warn` + 降级为无声, 视觉功能不受影响。`rodio` 仅进 `[dev-dependencies]`。
- **音源资产**: `assets/audio/` 5 个 OGG Vorbis (每个 ≤2MB), 来源 Freesound CC0 筛选 / OpenGameArt CC0, `assets/audio/ATTRIBUTION.md` 逐段记录出处与许可。无缝循环: ffmpeg 裁剪 + 循环点首尾 50ms 微 crossfade 处理。
- **场景音源路径平行数组**: `SceneSpec` 在框架层(`src/theme.rs`)不加字段; `ambient.rs` 内 `const SCENE_AUDIO: [&str; 5]` 与 `SCENES` 索引对齐, 编译期 `assert` 长度一致(测试护栏)。
- **完成提示音不变**: 阶段流转仍 `MessageBeep`(系统通道, 与环境音不冲突), 不引入铃声资产。

## Dependency Graph

```
Task 1  timer.rs: Phase::LongBreak + 轮次计数 + TickReport (纯逻辑)
 ├─ Task 2  持久化 completed_focus + UI 副标轮次/长休息
 └─ Task 3  今日计数纯逻辑 (日期归零判定)
     └─ Task 4  持久化 today_* + 计数接线 + UI「今日 N」

Task 5  音源资产 (5 × OGG ≤2MB + ATTRIBUTION)      ┐ 可并行
Task 6  ambient.rs AmbientMixer 纯逻辑 + 测试       ┘
 └─ Task 7  rodio 接入 + 2 槽 Sink + 懒初始化 + 降级 (依赖 5, 6)
     └─ Task 8  终验: benchmark 双门槛 + 人工终审 + 封档 (依赖 1-7)
```

关键路径: 1 → 3 → 4 → (5/6) → 7 → 8。
并行车道: Phase 3 内 Task 5(选音源, 人工为主)与 Task 6(纯逻辑)互不依赖, 可并行; WS1 整体与 WS2/WS3 无代码耦合(只差 main.rs tick 一行接线), 但按风险排序仍放最后。

## Task List

### Phase 1: WS2 长休息 + 轮次

- [ ] **Task 1: `timer.rs` — `Phase::LongBreak` + 轮次计数 + `TickReport`**
  - **Description:** `Phase` 新增 `LongBreak` 变体(时长 15×60s, label `长休息`, `Serialize`/`Deserialize` 派生不变); `Phase::next(self, completed_focus: u8) -> (Phase, u8)` 实现轮次推进(Focus 完成第 4 个 → LongBreak 且计数归零; 否则 → Break; Break/LongBreak → Focus 计数不变); `Pomodoro` 新增 `completed_focus: u8` 字段, `new()`/`reset()` 归零; `tick` 返回类型从 `bool` 改为 `TickReport { advanced: bool, focus_completions: u8 }`(自然越过 Focus 终点才累计, 循环处理多次越过时累加); `skip` 语义更新: 切相位走 `Phase::next(self.phase, self.completed_focus)` 但**不写回** `completed_focus`(skip 不推进轮次); `restore` 增加 `completed_focus` 参数; `main.rs` 适配 `tick` 新返回值(`report.advanced` 维持 flash/beep/`phase_advanced` 现状)。
  - **Acceptance criteria:**
    - [ ] 连续自然完成 4 个 Focus → 进入 LongBreak (15:00), 之后回 Focus 且轮次重新计数
    - [ ] 第 1~3 个 Focus 完成后仍为 5:00 Break
    - [ ] skip 出 Focus 不推进 `completed_focus`; skip 出 Break/LongBreak 行为一致
    - [ ] reset 回 Focus Idle, `completed_focus` 归零
    - [ ] huge overshoot 跨多个 Focus 时 `focus_completions` 正确累加
    - [ ] 既有 41 个 timer 测试适配新返回值后全绿 + 新增 ≥6 个测试(4 轮进长休 / 长休时长 / skip 语义 / reset 清零 / overshoot 累加 / Phase 转移矩阵)
  - **Verification:** `cargo test --example pomodoro timer` 全绿; `cargo clippy -- -D warnings` 零警告。
  - **Dependencies:** None
  - **Files:** `examples/pomodoro/timer.rs`, `examples/pomodoro/main.rs`(仅 tick 返回值适配)
  - **Scope:** M

- [ ] **Task 2: 持久化 `completed_focus` + UI 副标轮次/长休息**
  - **Description:** `state.rs` `PomodoroState` 加 `#[serde(default)] pub completed_focus: u8`; `snapshot_state`/`from_state` 接线(传给 `Pomodoro::restore`); 新增旧 JSON(缺字段)加载测试。`main.rs` `countdown_block` 副标: Running 时 `专注 · 篝火 · 第 2/4 轮`(轮次 = `completed_focus + 1`, 仅 Focus 相位显示轮次段, Break/LongBreak 显示 `休息 · 篝火` / `长休息 · 篝火`); 暂停时 `⏸ 已暂停 · 篝火` 不变。
  - **Acceptance criteria:**
    - [ ] 轮次进度跨重启恢复(完成 2 轮后关闭重开, 副标仍显示第 3/4 轮)
    - [ ] 旧版 pomodoro.json (无 `completed_focus`) 正常加载, 默认 0
    - [ ] 副标在 Focus/Break/LongBreak 三相位下文案正确
    - [ ] 序列化往返测试覆盖新字段
  - **Verification:** `cargo test --example pomodoro` 全绿; 手动跑一轮看副标。
  - **Dependencies:** Task 1
  - **Files:** `examples/pomodoro/state.rs`, `examples/pomodoro/main.rs`, `examples/pomodoro/timer.rs`(`restore` 签名)
  - **Scope:** S

### ⏸ Checkpoint 1: 长休息 + 轮次就绪
- [ ] `cargo test --example pomodoro` 全绿; `cargo fmt --check` + `cargo clippy -- -D warnings` 零警告
- [ ] 手动验: skip 连按 7 次走一遍 4 轮相位流转, 副标轮次正确
- [ ] 提交 Phase 1

### Phase 2: WS3 今日完成计数

- [ ] **Task 3: 今日计数纯逻辑 — 日期归零判定**
  - **Description:** `state.rs`(或新 `today.rs`)加纯函数: `today_string() -> String`(chrono `Local::now().date_naive().to_string()`, YYYY-MM-DD)与 `resolve_today_count(saved_date: &str, saved_count: u32, today: &str) -> u32`(日期不同归零, 相同保留, 空串视为不同)。单元测试覆盖: 同日期保留 / 跨日归零 / 空串 / 首次启动。
  - **Acceptance criteria:**
    - [ ] `resolve_today_count` 四种输入组合全部正确
    - [ ] 不新增任何 Cargo 依赖(chrono 复用)
  - **Verification:** `cargo test --example pomodoro` 全绿。
  - **Dependencies:** None(逻辑独立, 但与 Task 4 同 Phase 顺序做)
  - **Files:** `examples/pomodoro/state.rs` 或 `examples/pomodoro/today.rs`(新)
  - **Scope:** S

- [ ] **Task 4: 持久化 `today_*` + 计数接线 + UI「今日 N」**
  - **Description:** `PomodoroState` 加 `#[serde(default)] pub today_date: String` + `#[serde(default)] pub today_count: u32`; `PomodoroApp` 新增 `today_count: u32` 字段; `from_state` 用 `resolve_today_count` 归零恢复; `tick` 中 `report.focus_completions > 0` 时先取 `today_string()` 比较(跨日归零)再累加; `snapshot_state` 写入 `today_string()` 与 `today_count`; 副标追加 `· 今日 N`(N ≥ 1 才显示该段, 拼接在轮次段之后); 旧 JSON 兼容测试。
  - **Acceptance criteria:**
    - [ ] Focus 自然完成 → 今日 +1 并随 1Hz 节流持久化; 重启后仍在
    - [ ] skip 完成 Focus 不计数
    - [ ] 跨本地日期启动计数归零(可改系统日期或构造 state 手动验)
    - [ ] 副标 N=0 不显示「今日」段, N≥1 显示
    - [ ] 旧版 pomodoro.json (无两字段) 正常加载
  - **Verification:** `cargo test --example pomodoro` 全绿; 手动验(可用测试构造 state 文件)。
  - **Dependencies:** Task 1(共用 TickReport), Task 3
  - **Files:** `examples/pomodoro/state.rs`, `examples/pomodoro/main.rs`
  - **Scope:** S

### ⏸ Checkpoint 2: 今日计数就绪
- [ ] `cargo test --example pomodoro` 全绿; `cargo fmt --check` + `cargo clippy -- -D warnings` 零警告
- [ ] 手动验: 完成一个 Focus(或构造 state), 副标显示「今日 1」
- [ ] 提交 Phase 2

### Phase 3: WS1 场景环境音(重头戏)

- [ ] **Task 5: 音源资产 — 5 × CC0 OGG + ATTRIBUTION**
  - **Description:** 从 Freesound(CC0 筛选)/ OpenGameArt(CC0) 为 5 场景各选一段环境音: 篝火(柴火噼啪)、海(海浪拍岸)、雨(稳定降雨)、山(山风/远风)、森林(鸟鸣+树叶); ffmpeg 转 OGG Vorbis (q≈4, 单声道或立体声按源), 每段 ≤2MB, 时长 1~3 分钟, 循环点首尾 50ms 微 crossfade 消接缝; 落 `assets/audio/{bonfire,sea,rain,mountain,forest}.ogg`(文件名与 `SCENES` 顺序对应); 写 `assets/audio/ATTRIBUTION.md`(每段: 标题/作者/来源 URL/许可/改动说明); `tests/assets.rs` 或 example 测试加资产存在性 + 体积护栏(≤2MB × 5)。
  - **Acceptance criteria:**
    - [ ] 5 段音源齐全, 每段 ≤2MB, OGG Vorbis 可解码
    - [ ] 循环播放听不出明显接缝(人工逐段审听)
    - [ ] ATTRIBUTION.md 五段记录完整, 许可均为 CC0/可再分发
    - [ ] 资产护栏测试绿
  - **Verification:** 人工审听 + `cargo test` 资产护栏。
  - **Dependencies:** None
  - **Files:** `assets/audio/*.ogg`(新), `assets/audio/ATTRIBUTION.md`(新), `tests/assets.rs`
  - **Scope:** M(人工挑选为主)

- [ ] **Task 6: `ambient.rs` — `AmbientMixer` 纯逻辑 + 单元测试**
  - **Description:** 新建 `examples/pomodoro/ambient.rs`: `SCENE_AUDIO: [&str; 5]` 平行数组(与 `SCENES` 索引对齐, 测试护栏长度与文件存在); `AmbientMixer` 纯逻辑: `frame_volumes(&self, from: usize, to: usize, fade: f32, running: bool, now: Duration) -> [(usize, f32); 2]`, 音量 = 淡化权重 × 暂停包络; 暂停包络状态机: running 边沿触发 300ms 线性 fade-in/fade-out(运行→暂停淡出, 暂停→运行淡入), 稳定态输出 0.0/1.0; 目标音量常量 `AMBIENT_VOLUME: f32 = 0.6`(固定, 无 UI)。单元测试: 淡化插值端点、暂停沉降边沿与时长、淡化中途切运行态、单场景(from == to)。
  - **Acceptance criteria:**
    - [ ] 静止 Running: 当前场景音量 = 0.6, 另一槽 = 0
    - [ ] 淡化中点: 两槽音量按 fade 权重分配(fade=0.5 时各占一半)
    - [ ] 暂停后 300ms 线性到 0; 恢复后 300ms 线性回 0.6
    - [ ] 淡化中途暂停/恢复, 输出连续无跳变(包络与淡化独立相乘)
    - [ ] 单元测试 ≥6 个全绿
  - **Verification:** `cargo test --example pomodoro ambient` 全绿。
  - **Dependencies:** None
  - **Files:** `examples/pomodoro/ambient.rs`(新)
  - **Scope:** M

- [ ] **Task 7: rodio 接入 — 2 槽 Sink + 懒初始化 + 静默降级**
  - **Description:** `Cargo.toml` `[dev-dependencies]` 加 `rodio`(当前稳定版; **注意 0.21+ API 大改**: `OutputStreamBuilder::open_default_stream()` / `Sink::connect_new(&stream)`, 以 docs.rs 当前版为准, 勿用旧 `OutputStream::try_default`); `ambient.rs` 加 rodio 适配层: `AmbientPlayer` 持有输出流 + from/to 两个 `Sink`, `apply(frame_volumes, scene_changed)` 每帧 `set_volume` + 场景切换时重建 to 槽(`Decoder` 流式 + `repeat_infinite()`); 懒初始化: 首次 `is_running()` 且未初始化时才打开输出流, 失败置 `disabled` 旗标永久降级; 每步失败(打开流/开文件/解码)`log::warn` 不 panic; `main.rs` `PomodoroApp` 持有 `AmbientPlayer`, `tick` 末尾一行接线(消费 `fader.frame` + `is_running`); 启动即 Idle 静默, 不阻塞启动路径。
  - **Acceptance criteria:**
    - [ ] 按"开始"后当前场景环境音淡入可闻; 场景前/后切换音景随画面 800ms 交叉淡化
    - [ ] 暂停 300ms 内淡出至无声, 恢复淡入; 与视觉降饱和同步
    - [ ] 音频设备被独占/拔出耳机: 不 panic, warn 日志, 视觉完整
    - [ ] 音源文件缺失(临时改名验证): 不 panic, warn 日志
    - [ ] `cargo test --lib --tests` + `cargo test --example pomodoro` 全绿(测试路径不触音频设备)
    - [ ] `cargo clippy -- -D warnings` 零警告; `cargo build --release --example pomodoro` 绿
  - **Verification:** 手动听感验收(5 场景 + 切换 + 暂停/恢复); 降级路径手动验(改名文件/占用设备)。
  - **Dependencies:** Task 5(资产), Task 6(mixer)
  - **Files:** `Cargo.toml`, `examples/pomodoro/ambient.rs`, `examples/pomodoro/main.rs`
  - **Scope:** L

### ⏸ Checkpoint 3: 环境音就绪
- [ ] 5 场景音景人工听感验收(循环接缝 / 交叉淡化 / 暂停沉降)
- [ ] 降级路径(设备占用 / 文件缺失)不 panic
- [ ] 提交 Phase 3

### Phase 4: 终验与收口

- [ ] **Task 8: benchmark 双门槛 + 人工终审 + 封档**
  - **Description:** `cargo build --release --example showcase` + `powershell -NoProfile -File tools/benchmark.ps1`(启动 ≤1s、常驻 WS ≤360MB); 番茄钟 release 实机终审: 完整走 1 个 Focus → Break 流转, 听环境音淡化与暂停沉降, 看轮次/今日计数显示; spec `docs/specs/pomodoro-polish.md` 验收清单逐条勾选; `tasks/todo-pomodoro-polish.md` 全部勾选; plan/todo 归档 `tasks/archive/`(按现有约定); `CLAUDE.md` Current State 更新(三件套关闭, 下一步恢复为"第二 POC 剪贴板历史管理器或用户指定"); git commit(提交前重新验证门槛, 防 IDE 自动保存改脏)。
  - **Acceptance criteria:**
    - [ ] benchmark: 启动 ≤1s、常驻 WS ≤360MB(核显记账)
    - [ ] `cargo fmt --check` + `cargo clippy -- -D warnings` + `cargo test --lib --tests` + `cargo test --example pomodoro` + `cargo build --release` 全绿
    - [ ] spec 三个 WS 验收清单全部勾选
    - [ ] 文档归档 + CLAUDE.md 更新
  - **Verification:** 上述验收 + 人工终审。
  - **Dependencies:** Task 1-7
  - **Files:** `tasks/todo-pomodoro-polish.md`, `CLAUDE.md`, `tasks/archive/`(归档移动)
  - **Scope:** S

### ✅ Checkpoint Complete: 三件套封档
- [ ] 三个 WS 全部人工验收通过
- [ ] 全部命令绿 + benchmark 双门槛达标
- [ ] 文档归档完成, CLAUDE.md 指向下一候选

## Risks and Mitigations

| 风险 | 影响 | 缓解 |
|------|------|------|
| rodio 0.21+ API 与旧教程差异大 | 按旧 API 写无法编译 | 实施时以 docs.rs 当前稳定版为准(source-driven), 先写最小发声 spike 验证 API 再接 mixer |
| cpal/rodio 在 windows-gnu 工具链链接问题 | 编译失败 | cpal 是纯 Rust + windows-sys 绑定, 无自研 shim; Task 7 第一件事 `cargo check` 验证; 若 msvc 专用问题, 按 README 用 MSYS2 binutils 环境 |
| 输出流初始化拖慢启动 | 启动 ≤1s 门槛破 | 懒初始化(首次 running 才开流); benchmark 在 Task 8 实测, 若仍超标则移到后台线程 |
| 5 条音频流常驻内存 | 360MB 门槛压力 | 2 槽设计(from/to), 流式解码不整段载入; benchmark 实测 |
| CC0 音源质量参差(篝火/森林难找) | 听感不达标 | 每场景备 2~3 个候选人工审听; 实在无满意则该场景用更中性的素材(如风声底)并在 ATTRIBUTION 注明 |
| 循环点接缝明显 | 沉浸感破坏 | ffmpeg 首尾 50ms 微 crossfade; 人工审听逐段过 |
| `TickReport` 改动波及既有 41 个 timer 测试 | 测试大面积适配 | 返回值改结构体但语义保持, `advanced` 字段一对一替换; 适配属机械改动 |
| chrono `Local` 在极少数环境取不到本地时区 | 日期错乱 | chrono 内部有 fallback; 归零判定是纯函数, 异常输入(空串)也安全 |
| 音频线程 panic 拖垮主进程 | 崩溃 | rodio 解码错误走 `Result` 不 panic; 输出线程由 cpal 管理, 主路径不 await |
| 用户机器无声音输出设备 | 功能缺失感 | 静默降级是设计行为(spec), 日志可见; 视觉打磨(WS2/WS3)不受影响 |

## Open Questions

1. **rodio 具体版本与 API 形态**: Task 7 第一步确认 docs.rs 当前稳定版(0.21+ 用 `OutputStreamBuilder`), 写最小 spike 验证后再接 mixer。
2. **`AmbientMixer` 放 example 还是框架层**: 本期 example(spec 非目标明确"音频管线不进框架层"); 若剪贴板 POC 也需要提示音, 再议上收 `src/` 的时机。
3. **山场景的"环境音"定义**: 山风 vs 更抽象的氛围底噪, 选源时人工判断, 与场景图气质匹配优先。
4. **暂停沉降时长 300ms 的听感**: 实施后可调(常量), 人工终审时定夺是否需要更长(如 500ms)。
