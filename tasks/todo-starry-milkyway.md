# Todo: 星夜银河升级

- @author 十四叔
- @date 2026/08/03
- Plan: `tasks/plan-starry-milkyway.md` · Spec: `docs/specs/pomodoro-scene-starry-milkyway.md`

## Phase 0: 授权地基(fail-fast)

- [x] **Task 1: 星表授权尽调 + BSC 数据源落地** ✅ 2026-08-03
  - 结论: BSC5 干净可用(低风险,全文见 spec Open Questions);HYG(CC BY-SA 4.0)已排除
  - 数据源: `http://tdc-www.harvard.edu/catalogs/ybsc5.gz` + `ybsc5.readme` 格式说明;交叉验证 HEASARC BSC5P
  - 附带发现: 目录自带 GLON/GLAT 银道坐标与 B-V 色指数,Task 2 免坐标转换

### Checkpoint: Phase 0 — 授权结论落字;不干净则降级重评审

## Phase 1: 数据管线

- [ ] **Task 2: tools/export-stars.py + assets/stars.bin**
  - Description: 新增预处理脚本(对齐 `export-scenes.py` 惯例): 下载/读取 BSC → 银道坐标 (l,b) → 投影到 UV(银经→x、银纬→y + 旋转常量,默认角度先取 60° 占位,Task 8 回填) → 写出 `assets/stars.bin`(每星 6B: x u16 / y u16 / mag u8 / 保留 u8,~53KB)。脚本打印自检: 星数、星等分布、织女(Vega)/牛郎(Altair)投影坐标。
  - Acceptance: 脚本跑通产出 stars.bin;织女/牛郎分列银河带两侧(b 符号相反),坐标与已知星图一致(人工核对);UV 外星点已剔除并打印剔除数
  - Verify: 运行脚本看自检输出 + 人工核对锚点星
  - Dependencies: Task 1
  - Files: `tools/export-stars.py`(新)、`assets/stars.bin`(生成入库)
  - Scope: M

- [ ] **Task 3: Rust 星表解析模块 + 单测**
  - Description: 新增 `examples/pomodoro/starfield.rs`(纯逻辑,无 GPU): `include_bytes!` 加载 stars.bin,解码为星点迭代器(UV + 星等),星等→亮度/半径映射函数,边界防御(截断/非法记录跳过)。文件头 `@author 十四叔` / `@date`。
  - Acceptance: 单测覆盖——解码星数与 bin 长度一致、星等映射单调(亮星更亮更大)、截断数据不 panic、织女/牛郎抽验(容差内)
  - Verify: `cargo test --example pomodoro starfield` 全绿;`cargo clippy --example pomodoro -- -D warnings`
  - Dependencies: Task 2
  - Files: `examples/pomodoro/starfield.rs`(新)、`examples/pomodoro/main.rs`(挂 mod,1~2 行)
  - Scope: M

### Checkpoint: Phase 1 — starfield 单测全绿;锚点星投影人工核对通过

## Phase 2: 渲染通路

- [ ] **Task 4: 星野纹理烘焙 + 绑定点接线**
  - Description: `src/render/background.rs` 纹理 bind group 扩一槽(星野纹理,与场景大图 `create_scene_texture` 同机制的字节上传 API);example 侧启动时用 starfield 模块把 9,110 星 splat 成 1280×720 单通道位图(亮星带小光晕,6.5 等星 = 1px 弱点)并上传,常驻。测量并记录烘焙耗时。
  - Acceptance: 星夜场景渲染采样到星野纹理(可先以调试亮度直出验证);烘焙耗时实测 <100ms(记录数值);`cargo test --lib uniform_buffer_size` 等护栏全绿(uniform 未动)
  - Verify: `cargo test --lib --tests` 全绿 + `cargo run --release --example pomodoro` 目测星野出现
  - Dependencies: Task 3
  - Files: `src/render/background.rs`、`examples/pomodoro/starfield.rs`、`examples/pomodoro/main.rs`
  - Scope: M

- [ ] **Task 5: shader 重写 star_field / star_twinkle**
  - Description: `background.wgsl`: `star_field` 改为采样星野纹理 × `starry_base`;`star_twinkle` 脉冲逻辑不变(1/8 Hz 档位、`u.time`、±SF_TWINKLE_AMP),改为细网格脉冲场**调制采样结果**;旧常量 SF_COLS/SF_ROWS/SF_ON/SF_BIG/SF_WARM 退役(保留 SF_ASPECT/SF_BAND_BOT/SF_TWINKLE_AMP 中仍适用者)。meteor 不动。
  - Acceptance: 亮星位置来自纹理(非 hash 格);星闪发生在纹理亮星上;暂停 500ms 星闪沉降、星野定格(语义零回归);雨/火/海/山/森林常量未动
  - Verify: `cargo test --example pomodoro starry` 全绿 + 运行目测(运行/暂停)
  - Dependencies: Task 4
  - Files: `src/render/background.wgsl`
  - Scope: M

- [ ] **Task 6: star_haze 暗星雾 + 银纬 mask**
  - Description: `background.wgsl` 新增 `star_haze`: 细 hash 暗星点(极低亮度,不闪),密度按银纬解析 mask(b≈0 聚集,带宽/角度常量占位待 Task 8 回填)调制,挂 `starry_base` 常驻。与星野纹理叠加后密度分级肉眼可辨。
  - Acceptance: 带内密度明显 > 带外(目测);常量集中在 wgsl 常量段;既有测试全绿
  - Verify: `cargo test --example pomodoro starry` + 目测
  - Dependencies: Task 5
  - Files: `src/render/background.wgsl`
  - Scope: S

### Checkpoint: Phase 2 — 旧底图+新星野目测通过;五场景零回归

## Phase 3: 资产与对齐(Task 7 可与 Phase 1~2 并行)

- [ ] **Task 7: AI 底图重生 + 挑片**
  - Description: 按 spec 构图契约重画星夜底图: 银河光带左下→右上斜跨、最亮段压上 1/3、避开中央倒计时区、底部两层山脊剪影保留、base (22,26,52) 不变。**灰度防线**: 无可分辨亮星(允许星点雾);"可分辨"阈值由用户裁定。经 `tools/export-scenes.py` 重新生成。
  - Acceptance: 用户挑片通过(多轮为预期成本);底图无可分辨亮星(逐张人工检查);`scenes.rs` 由生成器产出(不手改)
  - Verify: 人工挑片 + `cargo run --release --example pomodoro` 目测
  - Dependencies: None(可与 Phase 1~2 并行)
  - Files: `assets/`(底图)、`tools/export-scenes.py`(如需调参)、`examples/pomodoro/scenes.rs`(生成)
  - Scope: M

- [ ] **Task 8: 光带常量回填 + 三层对齐**
  - Description: 测量新底图光带中心线,回填 wgsl 银纬 mask 角度/带宽常量(与 export-stars.py 旋转常量一致);目测迭代三层对齐(光带/亮星带/暗星雾带重合)与"深邃"剂量(暗星雾密度/光带亮度/星闪幅度配比)。
  - Acceptance: 三层带结构目测重合;银河一眼可辨;**同区域迭代 ≤5 commit**(超预算提级换范式);阈值裁定回填 spec
  - Verify: 运行目测 + commit 计数自查
  - Dependencies: Task 6、Task 7
  - Files: `src/render/background.wgsl`、`tools/export-stars.py`(角度常量)、`docs/specs/pomodoro-scene-starry-milkyway.md`(阈值回填)
  - Scope: S

### Checkpoint: Phase 3 — 三层重合;与倒计时大字区无遮挡冲突

## Phase 4: 收口

- [ ] **Task 9: benchmark + 全量回归 + 三件套**
  - Description: `powershell -NoProfile -File tools/benchmark.ps1`(须先 release 构建);全量测试;fmt/clippy/test 三件套零警告。
  - Acceptance: 暖机启动 ≤1s(含星表烘焙)、常驻 WS ≤360MB PASS;`cargo test --lib --tests` 与 `cargo test --example pomodoro` 全绿
  - Verify: benchmark 输出 + 三件套输出
  - Dependencies: Task 8
  - Files: 无(仅验证;如门槛不破则无需改代码)
  - Scope: S

- [ ] **Task 10: 用户终审**
  - Description: 与海/雨并排盲切(意图层验收方法学);spec Success Criteria 8 条逐条过;外部 3~5 人第一反应决定项在终审前定。
  - Acceptance: 用户显式 yes;spec 状态改为"已关闭";plan/todo 归档惯例处理
  - Verify: 用户显式 yes
  - Dependencies: Task 9
  - Files: `docs/specs/pomodoro-scene-starry-milkyway.md`(状态)
  - Scope: XS
