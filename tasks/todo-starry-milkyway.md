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

- [x] **Task 2: tools/export-stars.py + assets/stars.bin** ✅ 2026-08-03
  - 产出: `tools/export-stars.py`(纯 stdlib, 缓存 tools/.cache/ 已 gitignore) + `assets/stars.bin`(40,466 B, 8B 头 "DQST" v1 + 6,743 星 × 6B)
  - 自检全过: 解析 9,096/9,110(14 条非恒星记录缺字段跳过); 织女(0.605,0.031)/牛郎(0.730,0.191) 分列带两侧且 l/b/星等解析抽验命中; 银心落 (0.587,0.320) 上三分之一; UV 外剔除 2,353 颗并打印
  - 观测姿态常量(占位, Task 8 回填): L_CENTER=-45°, THETA=60°, FOV 260°×150°, SHIFT_Y=-0.03
  - 设计微调: "保留 u8" 落实为 B-V 色指数量化(0xFF=缺失), 服务星点暖色分布; bin 加 8B 自描述头(魔数+版本+计数), 利于 Task 3 截断防御

- [x] **Task 3: Rust 星表解析模块 + 单测** ✅ 2026-08-03
  - 产出: `examples/pomodoro/starfield.rs` — decode(魔数/版本拒读/计数取小/截断跳过) + 星等→亮度(二次+0.02 地板)/半径(2.6px~1px)/B-V 染色映射, 7 单测全绿
  - 锚点测试(织女/牛郎 星等+UV 抽验)与内嵌 bin 硬耦合——Task 8 重导后必红, 强制同步(特性)
  - `#![allow(dead_code)]` 占位(Task 4 接线时移除); tint 注释已修正(冷星不染蓝)

### Checkpoint: Phase 1 — starfield 单测全绿 ✅;锚点星投影人工核对通过 ✅(Task 2 自检)

## Phase 2: 渲染通路

- [x] **Task 4: 星野纹理烘焙 + 绑定点接线** ✅ 2026-08-03
  - 产出: engine `BackgroundConfig::with_starfield`(RGBA 字节+尺寸) + pipeline group 3 恒绑(1×1 全黑 fallback) + `upload_rgba_texture`(**Rgba8Unorm 线性格式**——bake 是线性权重非 sRGB 图像, 评审修正); example 侧 `bake_starfield_rgba`(6,743 星二次软点 splat, 与场景图同画布 1536×1024) + 启动接线
  - **烘焙耗时实测 13.4ms (debug)** — 远低于 <100ms 目标; release 更快, 精确值见启动日志 `星野烘焙: ... 耗时`
  - wgsl 调试直出行已标 TODO(Task 5) 届时移除; uniform 布局未动(护栏全绿); 新增 5 测(bake×4 + config×1)
  - **遗留(GUI 目测)**: 星夜纹理通路目测延后——验证时用户番茄钟正在运行(多实例会抢同一状态文件+全局热键), 留待 Phase 2 checkpoint 用户目测一并确认

- [x] **Task 5: shader 重写 star_field / star_twinkle** ✅ 2026-08-03
  - 产出: `star_field` 改为采样星野纹理(真实星表, B-V 暖色随纹理); `star_twinkle` 改为 96×54 脉冲场**调制采样**(脉冲逻辑逐字保留: {2,3,4}/8 Hz + 双极 ±0.42); `star_band()` 山脊遮挡两函数共用; 旧常量 SF_COLS/ROWS/ON/BIG/WARM/ASPECT + star_cell/star_color 全部退役, 无残留引用; Task 4 调试直出行已移除
  - **顺带修复一处已提交 bug** (Task 4 commit e77d639): 纹理格式修改时 `str.replace` 首处命中误改了 `load_texture`(光晕/噪声 PNG 被切成线性格式会洗白), `upload_rgba_texture` 反而没改成——两处已对调归位(scene/load=Srgb, starfield=Unorm), 评审抓获
  - 验证: naga Validator (ValidationFlags::all()) 通过(含模块作用域前向引用确认); 全量回归全绿; **wgsl 运行时编译+目测仍留待用户 checkpoint**(番茄钟运行中不便开第二实例)

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
