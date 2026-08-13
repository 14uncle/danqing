---
name: danqing-assets-directory-convention
description: "丹青二进制视觉资产统一放 assets/ 并提交版本控制,build.rs 不再生成资产"
metadata: 
  node_type: memory
  type: project
  originSessionId: a53524db-941c-4e0e-a644-7c28395b946f
  modified: 2026-07-24T07:09:38.556Z
---

丹青项目约定：字体、LOGO、背景图等二进制视觉资产统一放在仓库根目录 `assets/` 下，并提交到版本控制。

目录结构：

- `assets/fonts/` — 内嵌 OFL 黑体(思源黑体 GB2312 子集,加载链首选),原 ZCOOL XiaoWei 回退字体已于 2026-07-23 移除(提交 `8abd38d`)。
- `assets/logo/` — 多尺寸 PNG / ICO。
- `assets/background/` — 渐变背景图、噪声纹理等。

此前这些资产由 `build.rs` 在构建时下载或生成到 `OUT_DIR`。2026-07-20 起改为仓库内提交，`build.rs` 被移除，`Cargo.toml` 也不再需要 `[build-dependencies]`。

代码加载方式：

- 内嵌字体：`src/text/font.rs` 使用 `include_bytes!("../../assets/fonts/fallback-font.ttf")`。
- 窗口图标：`src/window.rs` 运行时读取 `assets/logo/logo_256.png`。
- 背景图：`examples/showcase.rs` 等通过 `BackgroundConfig::with_image("assets/background/gradient.png")` 配置。

**Why:** 消除首次构建对网络和 `build.rs` 的依赖，保证 CI 和离线环境可复现；同时让视觉资产受版本控制管理，便于追溯与替换。

**How to apply:** 新增字体 / LOGO / 背景图时直接放入 `assets/` 下对应子目录，随代码一起提交；不要再在 `build.rs` 或 `OUT_DIR` 中生成资产。

相关记忆：[[丹青项目状态]]、[[丹青战略定位]]
