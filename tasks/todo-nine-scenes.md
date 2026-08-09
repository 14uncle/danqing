# Task List：沉浸世界九场景

## 阶段1：场景定义 + 基础设施

- [x] Task 1: scenes.rs 添加4个新 SceneSpec
  - Acceptance: SCENES 数组长度为9，断言通过
  - Verify: `cargo test --example pomodoro`
  - Files: `examples/pomodoro/scenes.rs`

- [x] Task 2: ambient.rs 扩展 SCENE_AUDIO 为9条
  - Acceptance: SCENE_AUDIO.len() == SCENES.len()，测试通过
  - Verify: `cargo test --example pomodoro scene_audio`
  - Files: `examples/pomodoro/ambient.rs`

## 阶段2：Shader 动效

- [x] Task 3: background.wgsl 添加铁匠铺 shader
  - Acceptance: blacksmith_sparks 函数正确，编译通过
  - Verify: `cargo clippy -- -D warnings`
  - Files: `src/render/background.wgsl`

- [x] Task 4: background.wgsl 添加洞穴 shader
  - Acceptance: cave_drip 函数正确，编译通过
  - Verify: `cargo clippy -- -D warnings`
  - Files: `src/render/background.wgsl`

- [x] Task 5: background.wgsl 添加夜市 shader
  - Acceptance: nightmarket_lanterns 函数正确，编译通过
  - Verify: `cargo clippy -- -D warnings`
  - Files: `src/render/background.wgsl`

- [x] Task 6: background.wgsl 添加火车 shader
  - Acceptance: train_motion 函数正确，编译通过
  - Verify: `cargo clippy -- -D warnings`
  - Files: `src/render/background.wgsl`

## 阶段3：动效策略 + 主程序接入

- [x] Task 7: motion.rs 添加4组场景动效函数
  - Acceptance: blacksmith_intensity/cave_intensity/nightmarket_intensity/train_intensity 函数存在
  - Verify: `cargo test --example pomodoro`
  - Files: `examples/pomodoro/motion.rs`

- [x] Task 8: main.rs 接入4个新场景强度
  - Acceptance: background_frame() 包含新场景强度
  - Verify: `cargo clippy -- -D warnings`
  - Files: `examples/pomodoro/main.rs`

## 阶段4：环境音

- [x] Task 9: export-ambient.py 添加4个新场景音频生成
  - Acceptance: 4个新 OGG 文件生成，RMS 合理
  - Verify: `python3 tools/export-ambient.py`
  - Files: `tools/export-ambient.py`

- [x] Task 10: export-scenes.py 添加4个新场景配置
  - Acceptance: 4个新 PNG 文件导出
  - Verify: `python3 tools/export-scenes.py`
  - Files: `tools/export-scenes.py`

## 阶段5：验证

- [x] Task 11: 全量测试 + clippy + 手动验证
  - Acceptance: `cargo test --lib --tests` 全绿，clippy 零警告
  - Verify: 运行 pomodoro，9个场景可切换，听音辨识
  - Files: —
