---
name: review-before-commit
description: 代码提交前必须执行一遍 /agent-skills:code-review-and-quality 技能评审
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 0112d934-8ade-4fe0-ac65-6d9ff4e7edb8
  modified: 2026-07-28T05:41:13.479Z
---

代码提交(commit)前,必须先执行一遍 `/agent-skills:code-review-and-quality` 技能对待提交 diff 做五轴评审。

**Why:** 用户 2026-07-28 明确要求。该流程已在性能调优提交中实战验证 — 独立评审抓到主流程漏掉的「解码失败每帧重试」Required 项。

**How to apply:** 任何 `git commit` 之前,先对 `git diff` 过一遍 code-review-and-quality(可派独立 code-reviewer 代理交叉评审);Critical/Required 项修复并复验门槛后才能提交。与 [[verify-immediately-before-commit]] 互补:那个管「门槛紧贴 commit 重验」,这个管「提交前必评审」。
