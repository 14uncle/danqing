# Plan: 框架字体重构 — Sarasa Mono SC SemiBold (子集)；SSAA 已撤销

> spec: `docs/specs/font-cjk-mono.md`（已批准；2026-09-06 复盘 pivot：字重 Regular→**SemiBold**、SSAA→**不做**（fontdue AA 不锐化、改字重）、旧字体→**归档**）。
> 框架级改动，danqing 全家受益（log/pomodoro/clipboard/showcase 随字体自动变化，不写产品代码）。

## 架构决定

- **字体源**：`SarasaMonoSC-TTF-Unhinted-1.0.41.7z`（49.7MB，GitHub be5invis/Sarasa-Gothic v1.0.41）。选 Unhinted：fontdue 不做 TrueType hinting，Unhinted 轮廓更干净、体积更小。
- **子集**：fonttools（已装 4.63）从 `SarasaMonoSC-Regular.ttf` 按 `tools/gb2312-charset.txt` + ASCII 子集 → `assets/fonts/ofl-mono.ttf`。Sarasa 是**静态 TTF 非可变**，跳过 `varLib.instancer`（现 Noto 脚本该步去掉）。目标 <3MB（对齐现 `SIZE_LIMIT` 断言）。
- **超采样**：`atlas.rs::get_or_rasterize` 改 2×：`rasterize(ch, px*2)` → LANCZOS 缩回 `px` → `GlyphInfo` 的 `advance/width/bearing` 以**目标 px** 重导出（无 2× 泄漏）。图集 allocate 仍按目标 px（常驻不变）。
- **呈现**：框架所有文本走新 mono；Noto Sans SC 移入 `assets/fonts/archive/` 归档。

## Task List

### Phase 0: 获取字体
- [ ] **T1: 下载 + 解出 Regular TTF**
  - Acceptance: `SarasaMonoSC-Regular.ttf` 落在本地（temp 或 `D:\app`），并能被 fontdue 打开（`Font::from_bytes` 成功）
  - Verify: 解压后可读 `fontTools.ttLib.TTFont` 头
  - Files: 下载产物（不入库）；工具 = `py7zr`（`pip install py7zr`）或 7-Zip
  - 风险：49.7MB 下载 + 解压；`-k` 绕过 GitHub TLS 吊销检查（已实测可行）

### Phase 1: 子集
- [x] **T2: 子集脚本 → ofl-mono.ttf** ✅ (SemiBold 2.01MB)
  - Acceptance: `tools/subset-mono-font.py` 用 fonttools 子集 **SemiBold**.ttf → `assets/fonts/ofl-mono.ttf`；`<3MB`；含 GB2312 + ASCII + 常用中文标点；OFL 授权备注正确
  - Verify: 三件套后单测栅格化「你」「a」成功；`tests/assets.rs` 的 SIZE_LIMIT 断言过（改指向新字体名）
  - Files: `tools/subset-mono-font.py`, `assets/fonts/ofl-mono.ttf`, `assets/fonts/OFL.txt`, `tests/assets.rs`
  - 注: 字重 Regular→SemiBold（复盘纠偏, 见 spec §Crispness 决策; Sarasa 无 Medium 档）

### Phase 2: 换字
- [x] **T3: font.rs 换内嵌源 + 归档旧字体** ✅ `embedded_sans→embedded_mono`
  - Acceptance: `EMBEDDED_MONO_BYTES = include_bytes!("../../assets/fonts/ofl-mono.ttf")`；常量名/方法名（embedded_mono）更新；`ofl-sans.ttf` 移入 `assets/fonts/archive/`
  - Verify: `cargo test` 全绿；`tests/assets.rs` 指向 mono
  - Files: `src/text/font.rs`, `src/text/atlas.rs`, `examples/mem_probe.rs`, `assets/fonts/`, `tests/assets.rs`, `src/render/text.rs`(descent 测试)

### Phase 3: 锐化 —— **SSAA 撤销，改字重（SemiBold）**
- [x] **T4: (撤销) atlas.rs SSAA** —— 复盘纠偏不做（fontdue 解析式 AA 封顶目标 px，SSAA 不锐化）。改以 **SemiBold 字重**达成（T2 落地）。atlas.rs 无 SSAA 改动，仅调用点更名。

### Phase 4: 验证
- [x] **T5: 单测 + 三件套** ✅
  - Acceptance: ①子集字体栅格化 CJK「你」+ASCII「a」，advance>0 ②覆盖 `，。：「·+` ③descent 测试比真实 metrics ④三件套全绿（386 lib）
  - Verify: `cargo test --lib` 新增用例 + `cargo clippy -D warnings` + `cargo fmt`
  - Files: `src/text/font.rs`, `src/render/text.rs`, `tests/assets.rs`
- [x] **T6: 人工 showcase + log 对比（用户验收）** ✅ 用户「通过」
  - Acceptance: `cargo run --example danqing-showcase` base/form 页小号中文锐利（SemiBold）；正文/表格等宽对齐；旧 Sans 观感不存在
  - Verify: 截图对比（用户过目）

## Checkpoint: 模块验收（T6 后）
- [ ] 三件套绿（fmt + clippy -D warnings + 测试）
- [ ] spec 成功判据逐条对照
- [ ] 人工验收（用户过目）
- [ ] 进 review 阶段

## 风险与缓解
| 风险 | 影响 | 缓解 |
|------|------|------|
| 49.7MB 下载 + 7z 解压摩擦 | 高 | py7zr 装 / 用户手放；备选 Noto Sans Mono CJK |
| SSAA 后 advance 泄漏 2× | 高（布局全错） | T4 显式断言 GlyphInfo 为目标 px；单测锁 |
| 子集字体超 3MB / 缺中文字 | 中 | 脚本可复跑定字符集；`tests/assets.rs` SIZE_LIMIT 断言 |
| fontdue 对 TTF 兼容 | 低 | fontdue 0.9 支持 TTF；解出单 TTF 而非 TTC |

## Open Questions（plan 不改语义，需求时提）
- 无（spec 三小点已按「按你推荐」定）
