# Plan: Overlay 模态浮层组件 (框架下沉·簇C)

> Spec: `docs/specs/SPEC-overlay.md` (已批准 2026-09-09)。来源: `docs/intent/framework-sinking.md` 簇C (6 份手写拷贝, 全场最多)。
> 簇A/B 先例 (plan-anim/plan-job) 的结构与纪律全部沿袭。

## 决策 (D1–D8)

- **D1 组件形态采 log 的组件化门控** (settings.rs SettingsOverlay 是六语义最全形态), 不采 pomodoro 声明式 (门控散在调用方正是 6 份拷贝的根源)
- **D2 遮罩吞事件统一化** (spec 决策2): 开态卡片外点击一律 Consumed, `on_scrim_click` opt-in 发消息; pomodoro 迁移有观感变化 (现网遮罩不吞), 验收姿势钉死
- **D3 Esc 不进组件** (spec 决策1): pomodoro 无 / log 有且焦点优先清空 (S3 教训); 组件 doc 写范式, 产品侧自理
- **D4 关态 layout 返回 `(constraints.max().width, 0)`** — log 实证形态, 不占空间不扰底层布局
- **D5 内容槽纯注入**: Overlay 只管 scrim + 居中 + 门控 + 渲染层 + 遮罩点击; 玻璃卡片结构 (UiBox+Padding+surface+radius_lg) 产品自建 —— 三产品卡片样式各异, 收进来是过度泛化
- **D6 绑定/消息走既有范式**: `bind_open<S>` = Switch 式 Any 闭包; `on_scrim_click<M>` = MsgFactory 模式; 内容子树存 `Node` (组件模板范式)
- **D7 零 commit 纪律**: 未获用户指示不 commit/push; 迁移分仓分别提交、message 注明关联
- **D8 迁移顺序与闸门**: pomodoro (无前提) → log (前提闸: 在途批次 async-open+text-selection 已提交, 与簇B T6 同闸) → clipboard (用户裁决是否动; 双层叠加是 z 序验证场)

## 依赖图

```
T1 (overlay.rs 骨架 + open 门控) ──→ T2 (开态交互 + 渲染层) ──→ T3 (showcase 演示卡)
                                                                        │
                                                                        ▼
                                          CP1 (三件套绿 + 实机 + commit/push 裁决)
                                                                        │
                                                                        ▼
                                              T4 (pomodoro 三面板迁移, 前提: push)
                                                                        │
                                          [闸: log 在途批次已提交]        │
                                                  │                     ▼
                                                  ▼                   CP2
                                              T5 (log 迁移)
                                                  │
                                          [裁决: 动不动 clipboard]
                                                  ▼
                                              T6 (clipboard 迁移) ──→ CP3
```

## 风险

| 风险 | 缓解 |
|---|---|
| 遮罩点击判定依赖卡片实际矩形, 布局时序错位会误吞/误穿 | layout 期 Cell<Rect> 缓存卡片 area (switch.rs 先例); 单测锁定命中域 |
| pomodoro 迁移观感变化 (遮罩吞事件) | spec 决策2 已裁; T4 人工姿势含「点面板外不穿透」 |
| 双层叠加 (clipboard 确认层盖设置层) z 序错乱 | 两 Overlay 树内兄弟序 + 各自 push_layer; T6 人工姿势验证 |
| log 闸未开就动工 | D8 前提闸: log 在途批次不提交不动工 |
| 产品侧面板互斥/焦点回归逻辑误删 | 迁移只换浮层机构, 状态逻辑逐行保留; 既有测试 (面板互斥/焦点回归) 零改动绿为判据 |
