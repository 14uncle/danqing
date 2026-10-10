# 人工验收总清单 · danqing 框架(下沉流水线专项)

> 全家惯例: 人工实机/过目项集中记账一处, 逐项打勾回填; 发现缺陷另开行记。
> 产品级验收在各产品仓(dropshelf/luoban/plainsight/log 各有本文件); 本清单只收
> **框架下沉流水线**产生的人工项(B 批 + S1 授权栈)。
> 状态: ⬜ 待验 / ✅ 通过 / ❌ 不过。

## B 批 · 框架下沉第二批(2026-10-10 五段全闭, plan: `tasks/plan-sinking-batch2.md`)

### Checkpoint 1 · 框架侧五件过目

对照 commit(dev 已推): T1 `9ceb1cb` / T2 `7eb1617` / T3 `259c2f7` / T4 `b3d72ae` / T5 `cf3ff03` / 复评修复 `9c25929`。

- ⬜ `persist::toml` 公共 API 形态过目(`src/persist/toml.rs`);语义口径三条确认: 读写按**并集**(非交集) / `bool()` 取 log 宽松口径(带引号 `"true"` 也认) / `str_list()` 括号非法 → `None` 保留旧值
- ⬜ **转义行为变化确认(本轮唯一读侧语义变化)**: 手写 `k = "C:\new"` 现按 TOML 标准读出换行(原为字面量 `\n`);写侧产物不受影响(Writer 现已转义 `\n\r\t`, 红队 F1 修复)。认可即勾
- ⬜ 剪贴板写侧 API 过目(`src/clipboard.rs`): `ClipSource::set_html` 默认实现返 `Unsupported` / `copy_html_verified` 对 `Unsupported` 零重试早退 / `ClipWriteError` 文案与 luoban 旧版逐字节一致
- ⬜ `delegate_theme!` 宏过目(`src/theme.rs`;33 个必填方法清单有覆盖锁测试)
- ⬜ `TextBatch::vcenter_baseline` 返回**偏移**(不含 y, 调用方补 `y +`)语义过目(`src/render/text.rs`)
- ⬜ `time` epoch 毫秒三件套过目(`src/time.rs`: `epoch_millis`/`system_time_to_millis`/`millis_to_system_time`, 全饱和)

### Checkpoint 2 · 三仓迁移 diff 过目(三仓均已推)

- ⬜ dropshelf `d6f7ae4`: `store.rs` 净删 ~160 行;84 测试零改动存活;`config.toml` 逐字段 round-trip 等价(两条有意口径分叉见 plan T7「迁移必读」—— 存量文件不受影响)
- ⬜ luoban `42c0648`: net −66 行;95 绿(**100→95 是预期**: encoding.rs 随迁 5 条由框架 `decode_bytes_matrix` 承接, `decode_bytes` 在 luoban 零生产调用点故整模块删);`preview.rs:245` 顶对齐**未动**(非居中, 动则 1px 偏移)
- ⬜ plainsight `0a9556d`: 仓内 helper 删 + 4 处换框架件;47 绿零改动存活;lock 跳 `7813d1d3→9c25929a`(带入 dropshelf 引擎五肌肉批, windows 0.61 依赖组变化)

## S1 · 授权栈(crate: `danqing-license`, plan: `../danqing-license/tasks/plan-danqing-license.md`)

- ⬜ **闸 B**(T7 dropshelf 迁移落地后执行): dropshelf 便携版 key 激活人工走通 —— keygen 签 key → 激活 → 四功能门开(验收细则见 plan「闸 B」节)
- ⬜ **闸 C**(T8 log 迁移落地后执行, 排期挂 log v1.x 发布窗口): log 侧人工验收(细则见 plan「闸 C」节)

## 验收中缺陷记录

(逐条记: 现象 / 定位 / 处置)
