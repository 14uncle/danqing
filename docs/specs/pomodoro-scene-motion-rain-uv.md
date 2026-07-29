# Spec: 番茄钟场景动效 — 雨场景 UV 位移化改造

## Objective

海场景终审(2026-07-29)确立机制裁定:亮度调制读作"光在车上跑、路没动",要让背景图本身动起来必须动采样坐标(UV 位移),详见 [[scene-motion-uv-displacement]] 教训。用户同日指示:"雨、篝火,背景图也要跟着动,按这个效果来修改。"本里程碑为改造第一站——雨。

雨静态图自身的视觉语言:整幅蓝灰缓变天空(上浅下深,梯度极平缓)+ 满屏已画好的细雨丝(`\` 形朝右下,与既有 `RAIN_SLANT=0.12` 对齐,丝细、量散、低对比)。改造目标:让画中雨丝沿下落方向被"风阵"剪切推移——采样坐标本身动,而非在世界之上再叠加东西:

- **雨幕剪切**: 位移场沿雨丝下落方向(vec2(SLANT, 1))作用于采样坐标,幅度随垂直于雨丝的坐标(`uv.x - uv.y*SLANT`,即雨段既有列坐标)正弦变化、随时间同向行进——读作阵雨阵风扫过,画中雨丝随之推移;
- **既有叠加雨丝保留**: 程序化 `rain_overlay` 是"世界之上漂浮的粒子"语义(雨丝本就悬浮于世界之上),是连续下落读感的主力,原样保留,不动其常量。

剂量继承雨/篝火/海终审校准:元素细、速度慢、幅度低、形态与静态图一致。

用户: 番茄钟使用者(单一用户即作者本人)。成功 = 雨场景计时运行时,画中雨丝本身在推移(非仅叠加丝在落),暂停 500ms 内沉降回静态图,且不抢余光、不破性能门槛。

## Tech Stack

- Rust + wgpu 30 + winit 0.30(同主仓);动效全部在 `src/render/background.wgsl` 程序化生成,零新资产。
- **复用既有通道,零 Rust 变更**: `rain_intensity` 与 `time` uniform、`motion.rs` 策略层、`MotionEnvelope` 500ms 沉降、main.rs 接线在海里程碑前已全部就位;本里程碑预期只改 `background.wgsl`(雨段新增位移场 + 采样分支追加雨项)。

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

### 雨幕剪切位移场(background.wgsl 雨段新增,参数集中可调)

- **位移方向**: 沿雨丝方向 `vec2(RAIN_SLANT, 1.0)`(`\` 形朝右下;SLANT 小,免归一化,长度并入增益)。
- **位移场形状**(与海 `sea_swell` 同构,2 层同向行进正弦):
  - 横坐标复用雨段既有列坐标 `x = uv.x - uv.y * RAIN_SLANT`(等值线与雨丝平行,位移天然沿丝变化);
  - `w1 = sin(2π(2.0x + 0.3·uv.y) - t·W·2)`,`w2 = sin(2π(3.5x - 0.5·uv.y) - t·W·3 + 1.9)`,权重 0.6/0.4;相位含小 y 项使阵风口浪尖不成直线;两层同向行进(反向叠加成驻波,海场景调参轮 2 教训);
  - 时间频率 {2/8, 3/8} Hz 整数倍,保 8s 公共周期(`MOTION_WRAP_SECS`)。
- **幅度**: `RAIN_SHEAR_GAIN ≈ 0.008`(纵向 uv,960×640 窗 ≈ ±5px,约为画中丝长 1/5——可见推移不断丝)。
- **天空亮度安全性**(海"亮带"教训的定量复核): 天空梯度极平缓(全幅亮度跨度 ~0.3),±5px 垂直位移引起的亮度调制 <0.5%,不可读;画中雨丝是高对比高频元素,位移读感由它们承载。
- **采样分支**(fs_main,与海项并列叠加): `sample_uv += vec2(RAIN_SLANT, 1.0) * rain_shear(in.uv, u.time) * u.rain_intensity`。位移随 `rain_intensity` 缩放——暂停沉降逐像素回静态,暗启动纪律不破;雨↔海交叉淡化期间两位移项可同时非零,与海↔火同模型。
- 边缘: 位移 ±5px,ClampToEdge 作用于平缓梯度,不可读(与海同结论)。

### 不动的部分

- 雨叠加层(`rain_overlay` 三层雨丝)常量不动;海段、火段常量不动;`Uniforms` 布局不动;`background.rs` / `motion.rs` / `main.rs` 预期零变更。

## Boundaries

- **Always**: 提交前 `cargo fmt --check` + 两个 clippy + 全部测试绿 + 五轴评审;位移参数集中在 wgsl 雨段常量区,调参只动该段;策略层仍在 example 侧纯逻辑(本里程碑无新增策略)。
- **Ask first**: 新增依赖;改性能门槛;篝火 UV 位移化(下一里程碑,另行开工);改静态场景图资产;若实现中发现必须动 Rust 侧(预期不会)。
- **Never**: 改 `scenes.rs`(生成文件);引入新资产文件;为动效改变重绘频率(可见 60fps / 隐藏零渲染的架构事实不动);在 widget/layout/event/text 引入平台依赖。

## Success Criteria

1. 雨场景计时运行时,帧差证据:画面区裁框两帧(≥300ms 间隔)moved_ratio 显著大于纯叠加丝基线(改造前同法测一组作对照)且肉眼确认"画中雨丝本身在推移";对照场景(山)同法 ≈ 0。
2. 形态与静态图语言一致:推移沿雨丝下落方向,天空无可见亮带扫动,画面边缘无拉花;剂量"一眼可见但不抢戏"(用户目测裁定)。
3. 暂停 → 500ms 内沉降回静态(运行时暂停前后帧差 ≈ 0);暂停中恢复从当前值续接,无跳变。
4. 火/海/雨叠加丝零回归(三段常量未动;`rain_intensity=0` 时采样原坐标,逐像素一致),既有测试全绿。
5. 窗口隐藏时零渲染成本(架构事实,无新增 `request_redraw`)。
6. benchmark 门槛 PASS(暖机启动 ≤1s、常驻 WS ≤360MB)。
7. 提交门槛全绿(fmt / clippy×2 / test×2)+ 五轴评审通过。
8. 用户人工终审通过。

## Retreat Discipline

任一触发即 `git revert` 整里程碑,不留 feature flag:

- benchmark 回归(启动或内存超门槛且排除冷启动伪影后仍超);
- 终审不过且调参 >2 轮仍不达标(用户主导的方向修正不计入,同篝火"7px"与海"UV 位移"先例);
- 五轴评审发现架构性反流(框架层渗入场景策略等)。

## Open Questions

1. **叠加雨丝是否保留原剂量**: 拟原样保留(粒子语义合法,连续下落读感主力;位移场只负责"世界本身动")。若终审目测"雨太密/太乱",再降叠加层增益。——待用户确认。
2. **位移波形选择**: 拟正弦阵风(与海 `sea_swell` 同构,定点处为往复摆动、空间上阵风前沿行进)。备选: 流图式连续下落(双采样 + 三角权重交叉),连续单向但画中丝会被叠出鬼影重影,复杂度与风险都高。——拟正弦阵风,待用户确认。
3. **幅度初值**: `RAIN_SHEAR_GAIN = 0.008`(±5px @640 窗高)。丝长 20~30px,推移 1/5 丝长可见而不断丝;调参只动此常量。
