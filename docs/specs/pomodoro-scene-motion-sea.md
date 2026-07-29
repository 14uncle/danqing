# Spec: 番茄钟场景动效 — 海场景

## Objective

雨场景试点(2026-07-28 关闭)与篝火场景(2026-07-29 关闭,验收 8/8)已确立"潮汐式场景动效"范式:计时运行时世界环绕、暂停 500ms 沉降、每效果一标量 uniform 并存、剂量"一眼可见但不抢戏"。用户 2026-07-29 指示"剩下的场景继续做动效,一个一个来",本里程碑为第三场景——海。

海静态图自身的视觉语言:上方约 2/3 是大面积近白到浅青的极简天空,下方约 1/3 是三叠柔和的波带剪影(低对比、边缘柔软)。整体是亮色场景(调色板 base 168,221,232,深字亮底)。动效必须顺着这套语言走,不发明新元素:

- **波带涌动**: 波带剪影本身缓慢起伏——UV 纵向位移作用于采样坐标,天空不动、近水动得多(终审裁定,见 Open Question 4;初版"亮度乘性调制"被目测否决: 读作光斑沿静态波形移动,波形本身没动);
- **波光碎点**: 波带区域内散布的细碎光点缓慢明灭(同静态图低对比语言,形小、量少、速度慢,乘性提亮——亮场景下 additive 会被近白底吃掉)。

剂量继承雨/篝火终审校准:元素细、数量少、速度慢、幅度低、形态与静态图一致。

用户: 番茄钟使用者(单一用户即作者本人)。成功 = 海场景在计时运行时"活了"(波带涌动 + 波光碎点一眼可见),暂停时 500ms 内沉降回静态图,且不抢余光、不破性能门槛。

## Tech Stack

- Rust + wgpu 30 + winit 0.30(同主仓);动效全部在 `src/render/background.wgsl` 程序化生成,零新资产。
- 复用雨/篝火已建的通道: `BackgroundFrame.time` / uniform 动效槽位 / `motion.rs` 策略层 / `MotionEnvelope` 500ms 沉降(雨火海共用同一包络实例,同涨同落)。

## Commands

```bash
cargo test --lib --tests                 # 框架纯逻辑测试
cargo test --example pomodoro            # 番茄钟纯逻辑测试
cargo clippy -- -D warnings              # 工作区静态检查(不覆盖 example)
cargo clippy --example pomodoro -- -D warnings  # example 静态检查(必须单独跑)
cargo fmt --check
powershell -NoProfile -File tools/benchmark.ps1 -Example pomodoro -Runs 3  # 性能门槛
# 运行观测: cargo run --release --example pomodoro
# 抓帧: tools/print-window.ps1 <hwnd> <out.png>(输出须 Windows 全路径)
# 帧差: Python PIL 裁框灰度 diff(mean abs diff + moved_ratio>8)
```

## Design

### 框架能力(第三次使用 → uniform 槽位填充,不扩容)

篝火时 uniform 已从 16B 扩到 32B 并预留 3 个 pad 浮点位,海效强度恰好填入第一个 pad——**布局不变,仍是 32B**:

- uniform: `[opacity, fade, rain_intensity, time, fire_intensity, sea_intensity, pad, pad]`。
- `BackgroundFrame` 新增 `sea_intensity: f32`(默认 0)+ `with_sea()` 链式 builder;`with_motion` / `with_fire` 签名不变。
- `sea_intensity == 0` 时 shader 输出与静态逐像素一致(暗启动,与雨/火同纪律)。
- 时间共享: 海效复用同一 `time` uniform 与 8s 取模(`MOTION_WRAP_SECS`);海效所有频率/速度取 1/8 Hz 整数倍,保 8s 公共周期不破。
- 三个 pad 位填掉一个,剩两个;第四效果(山/森林)落地时继续填 pad,填完再议扩容(记录,不实现)。

### 海效 shader(background.wgsl 新增段落,参数集中可调)

- **涌动(终审裁定: UV 纵向位移)**: 采样坐标本身按位移场起伏——`sample_uv.y = uv.y + swell(uv,t) × sea_intensity`,from/to 两图同一偏移(交叉淡化两端一致)。位移场 = 2 层同向行进正弦(2/8、3/8 Hz,空间 2/3.5 周,相位含小 y 项使波峰不成直线)× 纵向 mask(天空区为 0,越靠下水层位移越大)。位移随强度缩放,暂停沉降逐像素回静态。
- **碎点**: 分列 hash(同雨/余烬的分列范式),每列相位随机;位置在波带区域内不动(原地明灭),亮度按低频正弦 smoothstep 缓起缓落(频率取 1/8 Hz 整数倍);2D 软圆点宽羽化边缘(宽高比修正),乘性提亮;点直径 ~5px @960px 窗宽,同屏 ~14 颗。
- 参数全部集中在 wgsl 常量段,调参只动该段(与雨/火段并列,互不改名改值)。

### 策略层(examples/pomodoro/motion.rs)

- `pub const SEA_SCENE: usize = 1;` + 单测锁定 `SCENES[1].name == "海"` 且唯一(防生成器重排)。
- `sea_intensity(from, to, fade, envelope)` 与 `rain_intensity` / `fire_intensity` 同权重合成(共享私有 `scene_weight` helper,公开 API 按效果分列)。
- `MotionEnvelope` 原样复用(雨火海共用同一包络实例——同涨同落,潮汐契约)。
- 交叉淡化期间三效果两两并存(雨↔海、海↔火),标量模型天然覆盖,补并存单测。

### 接线(examples/pomodoro/main.rs)

`background_frame` 追加 `.with_sea(motion::sea_intensity(from, to, fade, self.motion_gain))`;`tick` 包络推进逻辑不变。

## Boundaries

- **Always**: 提交前 `cargo fmt --check` + 两个 clippy + 全部测试绿 + 五轴评审;海效参数集中在 wgsl 常量段;纯逻辑(policy)留在 example 侧。
- **Ask first**: 新增依赖;改性能门槛;动效推广到山/森林;改静态场景图资产。
- **Never**: 改 `scenes.rs`(生成文件);海效引入新资产文件;为动效改变重绘频率(可见 60fps / 隐藏零渲染的架构事实不动);在 widget/layout/event/text 引入平台依赖。

## Success Criteria

1. 海场景计时运行时,帧差证据:波带区裁框两帧(≥300ms 间隔)moved_ratio > 0 且肉眼可见涌动+碎点明灭;对照场景(山)同法 ≈ 0。
2. 形态与静态图语言一致:碎点为小型软圆点(非丝非流星),涌动只调制已有波带亮度、不移边缘、不改色相;剂量"一眼可见但不抢戏"(用户目测裁定)。
3. 暂停 → 500ms 内海效沉降回静态(包络单元测试 + 运行时目测);暂停中恢复从当前值续接,无跳变。
4. 雨/火效行为零回归(雨/火段常量未动,既有测试全绿)。
5. 窗口隐藏时零渲染成本(架构事实,无新增 `request_redraw`)。
6. benchmark 门槛 PASS(暖机启动 ≤1s、常驻 WS ≤360MB)。
7. 提交门槛全绿(fmt / clippy×2 / test×2)+ 五轴评审通过。
8. 用户人工终审通过。

## Retreat Discipline

任一触发即 `git revert` 整里程碑,不留 feature flag:

- benchmark 回归(启动或内存超门槛且排除冷启动伪影后仍超);
- 终审不过且调参 >2 轮仍不达标;
- 五轴评审发现架构性反流(框架层渗入场景策略等)。

## Open Questions

(全部已决议, 2026-07-29 终审关闭)

1. **碎点叠加方式**: 亮场景下 additive 易被近白底吃掉, 拟用乘性提亮(与涌动同路径)。备选: additive 但增益放大。——决议: 乘性提亮 (增益 0.30 突兀 → 0.14 + 宽羽化, 终审通过)。
2. **波带区域锚点**: 位移区软入 uv.y 0.55→0.72、碎点散布带 uv.y 0.72~0.98(对齐静态图三叠波带)。——决议: 采纳, 终审未再调整。
3. **碎点是否漂移**: 拟原地明灭(水波光点不位移,实现更简单、公共周期更稳)。备选: 极慢水平漂移。——决议: 原地明灭, 终审通过。
4. **涌动机制(终审新增, 最重要的一条)**: 亮度乘性调制读作"明暗对象沿静态波形移动的车", 路没动; 用户裁定改为 UV 纵向位移——采样坐标本身起伏, 波带剪影随波行进。原约束"不移边缘"由此推翻。——决议: UV 位移 (幅度 ±0.015 uv ≈ ±9.6px @640 窗高), 终审 "棒, 效果对了"。

## 验收记录 (2026-07-29, 8/8 通过)

1. ✅ 帧差证据 (波带区裁框 y400-640, 1s 间隔): 海运行 mean=1.447 / moved 3.27% > 0; 对照山场景 mean=0.000 / moved 0.000%。
2. ✅ 形态终审: UV 位移使波带剪影本身起伏 (推翻"车在路上跑"的亮度调制); 碎点柔化后不突兀; 用户目测终审 "棒, 效果对了"。
3. ✅ 暂停 500ms 沉降: 包络单测 (1.0→0.5→0.0) + 运行时 P1/P2 帧差 0.000 (位移随强度缩放, 逐像素回静态)。
4. ✅ 雨/火零回归: 雨/火段常量未动 (`sea_intensity=0` 时采样原坐标, 逐像素一致), 既有测试全绿。
5. ✅ 隐藏零渲染成本: 里程碑全程 `src/window/` 零变更, 无新增 `request_redraw`。
6. ✅ benchmark PASS: 暖机 best 858.3ms ≤ 1s, median WS 178MB ≤ 360MB (run 1/3 偏高为已记录的冷启动伪影)。
7. ✅ 门槛全绿: fmt / clippy×2 / test×2 (226+ lib 集成 9 套件, 120 pomodoro) + 五轴评审 Approve (1 nit 非阻塞)。
8. ✅ 用户人工终审通过 (2026-07-29 "棒, 效果对了"; 剂量路径: 亮度调制 3 轮目测不可读 → 用户裁定换 UV 位移机制 → 一轮通过)。

调参轮次决算: 亮度调制路径自检+目测 3 轮未达剂量目标, 但未触发撤退条款——用户主动裁定更换机制 (UV 位移) 而非继续调参, 换机制后一轮通过 (同篝火"用户指定 7px"先例: 用户主导的方向修正不计入调参轮预算)。

## 后续方向 (用户 2026-07-29 终审时指示)

雨、篝火场景的背景图也要"跟着动"——按海场景的 UV 位移范式改造 (世界本身动, 而非在世界之上叠加动效)。作为下一里程碑候选记录, 不在本里程碑范围。
