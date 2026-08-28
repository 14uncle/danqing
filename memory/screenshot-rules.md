---
name: screenshot-rules
description: 截图流程与存放路径规则——先问用户要截图，自行截图放 target/tmp
metadata: 
  node_type: memory
  originSessionId: 465f4e7f-ba6b-4038-9993-f69126f7f23c
  modified: 2026-08-04T03:59:52.157Z
---

1. 需要截图时先问用户要，用户不提供再自行启动程序截图。
2. 截图一律放 `target/tmp/`，不放项目根目录，避免污染 git 工作区。

**Why:** 根目录截图文件会出现在 git status 里，干扰提交判断；路径散乱也难清理。
**How to apply:** 用 `target/tmp/` 作为截图临时目录，截图前 `mkdir -p target/tmp`。
