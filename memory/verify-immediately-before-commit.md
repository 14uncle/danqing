---
name: verify-immediately-before-commit
description: 提交前必须重新验证——用户 IDE 自动保存可能在验证与提交之间改动文件
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 6fc1e2fe-ba55-44fa-a821-d74fb2620f19
---

2026-07-22 提交 4 个切片时,button.rs 在「最后一次 cargo test」与「git commit」之间被用户 IDE 写入一个游离字符 `2`(误敲 + 自动保存),导致损坏内容进入提交,事后返工重做 3 个提交。

**Why:** 用户常驻 JetBrains IDE 打开项目文件,验证通过 ≠ 提交内容已验证。

**How to apply:** 在这个仓库执行提交时,把 fmt/clippy/test 门槛放在 `git add` 之后、`git commit` 之前(或提交后立即对提交内容复验),不要信赖几分钟前的验证结果;拆分提交需要暂存工作区时,先用 `git diff HEAD > patch` + 文件副本双重备份。
