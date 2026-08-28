---
name: danqing-rs-header
description: 丹青项目所有新 .rs 文件顶部必须添加作者与日期头注释
metadata: 
  node_type: memory
  type: project
  originSessionId: 39f8d8a0-14ca-4689-8b80-ab633a41cd73
---

在丹青（danqing）项目中，新建 `.rs` 文件时，文件头必须包含：

```rust
//! @author 十四叔
//! @date yyyy/MM/dd
```

其中日期替换为当天日期，格式 `yyyy/MM/dd`。

**Why:** 用户要求统一源码文件作者与日期标识，便于追溯。

**How to apply:** 创建新 `.rs` 文件后，立即在首行写入上述头注释；处理批量文件时可复用脚本逻辑。

补充（2026-07-22）：用户评估过 LLM 生成代码署人类作者名的风险，结论是头注释维持 `@author 十四叔` 不变（视为维护者/责任人署名），改为在 README 末尾「开发说明」一节披露"大量代码经 LLM 辅助生成、人工评审"。用户主要在意社区质疑与信誉，而非法律风险。

[[danqing-project-state]]
