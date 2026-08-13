---
name: shader-duplicate-constant-check
description: "修改 WGSL shader 时必须先检查同名常量,避免重复定义导致编译失败"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 25f88a87-d5c0-4e14-a620-5e26017357d4
  modified: 2026-08-09T10:58:08.445Z
---

修改 WGSL shader 添加新效果时,必须先搜索是否已存在同名常量/函数,再决定是复用还是替换。

**Why:** 夜市蒸汽效果重写时,新增 `NM_STEAM_ALPHA` 但未删除旧定义,导致 `redefinition of 'NM_STEAM_ALPHA'` 编译错误。此错误在 wgpu 运行时才暴露(cargo build 通过),浪费调试时间。

**How to apply:**
1. 添加新常量前,用 `grep` 搜索同名常量
2. 旧常量被新逻辑替代时,必须删除旧定义
3. 保留旧常量注释说明用途变化,便于追溯
4. shader 编译错误只在运行时暴露——改完 shader 后必须实际运行验证,不能只看 cargo build
