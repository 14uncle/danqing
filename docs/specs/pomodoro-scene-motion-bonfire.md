# Spec: 番茄钟场景动效 — 篝火场景

## Objective

雨场景动效试点(`docs/specs/pomodoro-scene-motion.md`,2026-07-28 关闭,验收 8/8)确立了"潮汐式场景动效"范式:计时运行时世界环绕、暂停 500ms 沉降、剂量"一眼可见但不抢戏"。用户 2026-07-28 指示"篝火开工",将同一范式推广到第二场景——篝火。

篝火静态图自身的视觉语言:下方中央的暖橙径向光晕(火体在画框外,只见辉光)+ 中景散布的细碎静态火星点。动效必须顺着这套语言走,不发明新元素:

- **光晕呼吸**: 下中央暖光区域的亮度缓慢起伏(乘性调制,只呼吸已有辉光,不改色相);
- **火星余烬上浮**: 与静态图中已有的火星点同形态(细、小、暖色),从下方光晕区缓慢升起、轻微横摆、升高后淡出。

剂量继承雨试点终审校准:元素细(2~3px)、数量少、速度慢、亮度低、形态与静态图一致。

用户: 番茄钟使用者(单一用户即作者本人)。成功 = 篝火场景在计时运行时"活了"(余烬上浮、光晕呼吸一眼可见),暂停时 500ms 内沉降回静态图,且不抢余光、不破性能门槛。

## Tech Stack

- Rust + wgpu 30 + winit 0.30(同主仓);动效全部在 `src/render/background.wgsl` 程序化生成,零新资产。
- 复用雨试点已建的通道: `BackgroundFrame.time` / uniform 动效槽位 / `motion.rs` 策略层 / `MotionEnvelope` 500ms 沉降。

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

### 框架能力(第二次使用 →  uniform 扩容)

雨试点时 16B uniform 的两个 pad 位恰好装下 `[rain_intensity, time]`,当时按 YAGNI 砍掉了 `SceneEffect` 枚举。篝火是第二个效果类型,但交叉淡化期间雨与火可以同时非零(from 雨淡出 + to 火淡入),效果间互斥的"选择子"模型不成立——**每效果一个独立强度标量**才是正确形状:

- uniform 16B → 32B: `[opacity, fade, rain_intensity, time, fire_intensity, pad, pad, pad]`;缓冲仍极小,无性能关切。
- `BackgroundFrame` 新增 `fire_intensity: f32`(默认 0)+ `with_fire()` 链式 builder;`with_motion(time, rain_intensity)` 签名不变。
- `fire_intensity == 0` 时 shader 输出与静态逐像素一致(暗启动,与雨同纪律)。
- 时间共享: 火效复用同一 `time` uniform 与 8s 取模;`RAIN_WRAP_SECS` 更名 `MOTION_WRAP_SECS`(第二用例使原名失效)。火效所有频率/速度取 1/8 Hz 整数倍,保 8s 公共周期不破。
- 第三效果类型落地前不引入打包/枚举方案(记录,不实现)。

### 火效 shader(background.wgsl 新增段落,参数集中可调)

- **呼吸**: `flicker(t)` = 3 个正弦叠加(频率 2/8、3/8、5/8 Hz,相位错开),周期 4s/2.67s/1.6s 叠出有机起伏;mask = 以光晕中心 (uv ≈ 0.5, 0.95) 的径向平滑衰减;`color.rgb *= 1 + flicker × mask × gain × fire_intensity`(乘性,幅度 ±4% 量级起步)。
- **余烬**: 分列 hash(同雨的分列范式),每列相位随机、速度全列一致(保公共周期);`y = fract(uv.y×scale + t×speed + rnd)` 向上漂;x 向低频正弦轻摆(幅度数像素);2D 圆点成形(非丝状),随高度淡出。暖色 additive(线性空间),点亮度低、直径 2~3px @960px 窗宽,同屏 ~15~25 颗。
- 参数全部集中在 wgsl 常量段,调参只动该段(与雨段并列,互不改名改值)。

### 策略层(examples/pomodoro/motion.rs)

- `pub const BONFIRE_SCENE: usize = 0;` + 单测锁定 `SCENES[0].name == "篝火"` 且唯一(防生成器重排)。
- `fire_intensity(from, to, fade, envelope)` 与 `rain_intensity` 同权重合成;共享私有 `scene_weight` helper,公开 API 按效果分列(调用点读性优先)。
- `MotionEnvelope` 原样复用(500ms 沉降,雨火共用同一包络实例——同涨同落,潮汐契约)。

### 接线(examples/pomodoro/main.rs)

`background_frame` 追加 `.with_fire(motion::fire_intensity(from, to, fade, self.motion_gain))`;`tick` 包络推进逻辑不变。

## Boundaries

- **Always**: 提交前 `cargo fmt --check` + 两个 clippy + 全部测试绿 + 五轴评审;火效参数集中在 wgsl 常量段;纯逻辑(policy)留在 example 侧。
- **Ask first**: 新增依赖;改性能门槛;动效推广到海/山/森林;改静态场景图资产。
- **Never**: 改 `scenes.rs`(生成文件);火效引入新资产文件;为动效改变重绘频率(可见 60fps / 隐藏零渲染的架构事实不动);在 widget/layout/event/text 引入平台依赖。

## Success Criteria

1. 篝火场景计时运行时,帧差证据:余烬区裁框两帧(≥300ms 间隔)moved_ratio > 0 且肉眼可见余烬上浮+光晕呼吸;对照场景(山)同法 ≈ 0。
2. 形态与静态图语言一致:余烬为 2~3px 暖色圆点(非丝非流星),呼吸只调制已有辉光、不改色相;剂量"一眼可见但不抢戏"(用户目测裁定)。
3. 暂停 → 500ms 内火效沉降回静态(包络单元测试 + 运行时目测);暂停中恢复从当前值续接,无跳变。
4. 雨效行为零回归(雨场景参数与表现不变,`rain_intensity` 路径不动)。
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

1. **效果概念**: 光晕呼吸 + 火星余烬上浮, 不画火焰形状(静态图本身无火体, 火在画框外)。——决议: 采纳, 终审通过。
2. **锚点**: 光晕中心 uv ≈ (0.5, 0.95), 余烬发射带 uv.x 0.35~0.75、自底部升起至 y≈0.35 淡出(对齐静态图火星散布带)。——决议: 采纳, 终审未再调整。
3. **uniform 形状**: 每效果一标量(32B), 不引入效果选择子枚举。——决议: 采纳 (交叉淡化期间双效果并存, T2 `rain_and_fire_coexist_on_crossfade` 测试固化)。
4. **时序**: 火效与雨共享 `time` + 8s 公共周期;火效频率全部落在 1/8 Hz 整数倍网格上。——决议: 采纳 (`MOTION_WRAP_SECS` 更名落地)。
5. **沉降**: 雨火共用同一 `MotionEnvelope` 实例(同涨同落),不为火单设包络。——决议: 采纳 (T3 接线落地)。

## 验收记录 (2026-07-29, 8/8 通过)

1. ✅ 帧差证据 (裁框左下余烬带, 0.4s 间隔): 篝火运行 mean=1.145 / moved 0.013% > 0; 对照山场景 mean=0.000 / moved 0.000%。
2. ✅ 形态与静态图语言一致: 余烬为热黄圆点 (终审裁定直径 7px), 呼吸乘性只起伏已有辉光; 用户目测终审 "go"。
3. ✅ 暂停 500ms 沉降: 包络单测 (1.0→0.5→0.0) + 运行时 P1-P2 帧差 0.000 (逐像素回静态); 恢复从当前值续接无跳变。
4. ✅ 雨效零回归: 雨段常量未动, 雨测试全绿, 冒烟期雨场景雨丝表现正常。
5. ✅ 隐藏零渲染成本: 里程碑全程 `src/window/` 零变更 (`git diff eb96c55^..HEAD -- src/window/` 为空), 无新增 `request_redraw`。
6. ✅ benchmark PASS: 暖机 best 587.7ms ≤ 1s, median WS 178.5MB ≤ 360MB (run 1 1397.5ms 为已记录的冷启动伪影)。
7. ✅ 门槛全绿: fmt / clippy×2 / test×2 (222 lib+集成, 114 pomodoro) + 五轴评审无 findings。
8. ✅ 用户人工终审通过 (2026-07-29 "go"; 剂量校准路径: 暗启动自检 1 轮 → 5px 第 2 轮 → 用户指定 7px 终审修正)。

调参轮次决算: 自检 1 轮 + 目测 1 轮 + 用户指定数值终审修正 1 次, 未触发撤退条款。
