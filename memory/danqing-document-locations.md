---
name: danqing-document-locations
description: 丹青项目 spec/plan/todo 文档目录约定
metadata: 
  node_type: memory
  type: project
  originSessionId: ea3032c3-05bd-457d-ac6d-370ab793404b
  modified: 2026-07-31T01:13:14.604Z
---

丹青项目文档按类型分目录存放：

- **spec 文档**统一放到 `docs/specs/`
- **plan 文档**统一放到 `tasks/`
- **todo 文档**统一放到 `tasks/`
- **已关闭的里程碑 plan/todo** 归档到 `tasks/archive/`(2026-07-22 起,M1~M3、标题栏、widget 分类、视觉 remediation r1 等 12 份已归档)
- **分层上下文(按需加载)** 放到 `docs/CONTEXT/`(2026-07-31 起,含 architecture.md 详细文件地图、scenes-guidelines.md 场景动效开发范式;CLAUDE.md 减至 ~100 行,细节下沉到此目录)

**Why:** 保持项目文档结构一致，避免 spec 与 plan/todo 散落在 `docs/` 根目录和 `tasks/` 中，方便快速定位；与 [[danqing-project-state]] 中 tasks/plan.md、todo.md 的约定对齐。

**How to apply:**

- 新建 spec（如阶段 1）→ `docs/specs/<name>.md`
- 新建 plan/todo → `tasks/<name>.md`
- 历史迁移：
  - `docs/spec.md` → `docs/specs/spec.md`
  - `docs/spec-m2.md` → `docs/specs/spec-m2.md`
  - `docs/spec-m3.md` → `docs/specs/spec-m3.md`
  - `docs/plan-m2.md` → `tasks/plan-m2.md`
  - `docs/plan-m3.md` → `tasks/plan-m3.md`
