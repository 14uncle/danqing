# SPEC: 框架下沉第二批(TOML / 剪贴板写侧 / 主题委托宏 / 基线 helper / 小件包)

- 来源:三新仓复盘 `danqing/docs/intent/retro-trio-reuse-2026-10-10.md` S2/S4/S5/S6 + 小件包(2026-10-10 用户批准打包一批进 spec)
- 判据:沿用 `danqing/docs/intent/framework-sinking.md` 四条——通用 / 第二消费者 / 质量 / 框架未有
- 落点:全部进 danqing 框架(除 decode_bytes 进 danqing-encoding);**不建新 crate**
- 状态:**草稿,待用户批准**

## Objective

复盘发现的中小件跨产品重复,一批收编。单件都小(15~130 行),但合计全家重复 ~700 行,且每一件都是「下一个产品还会再写一遍」的形态。本批收编后,danqing-log / danqing-dropshelf / danqing-luoban / danqing-plainsight 四仓迁移消费。

非目标:RowList 组件族(S3,设计重,单独 spec 殿后);rfd 对话框封装(S7,裁决=维持产品直用,本批不含);AsyncJob 迁移(S8,是消费侧迁移非新下沉,排期单列)。

## 收编清单

### 件 1:persist::toml 子模块(S2)

- 重复证据:danqing-log/src/config.rs:1-391 手写朴素 TOML ↔ dropshelf/src/store.rs:121-250 同模式 ~130 行
- 收编:`danqing/src/persist.rs` 增设 `toml` 子模块(feature `persist` 内),最小子集读写(bool/int/string 转义/string-list/section 交集语义——plan 阶段读两边定精确交集)
- 迁移:dropshelf store.rs、log config.rs
- 判据:通用 ✓ / 第二消费者 ✓(2 仓)/ 质量 ✓(两边都有测试)/ 框架未有 ✓

### 件 2:剪贴板写侧(S4,旧账 E3 补完)

- 缺口证据:框架 clipboard.rs 纯读原语;luoban 自填 `set_html` 双路写 + `copy_html_verified`(写后读回校验+重试)+ `normalize`(clipboard.rs:33-38/55-75/78-80,~45 行+测试)。旧下沉盘点簇 E3「多格式剪贴板写入+to_crlf」只落了 to_crlf 半边
- 收编:`danqing/src/clipboard.rs` 加写侧(读侧驻地,天然一家)
- 迁移:luoban 删写侧自填 + 读侧重复 ~15 行(复盘 ①#7 一并消)
- 判据:通用 ✓ / 第二消费者 ✓(luoban + 历史 clipboard 产品需求在档)/ 质量 ✓(luoban 版带测试) / 框架未有 ✓
- **纪律**:测试严禁真剪贴板(幻影粘贴教训,2026-09-09);写侧测试走注入假源

### 件 3:theme delegate! 宏(S5)

- 重复证据:dropshelf app_theme.rs:28-75 `delegate!` 逐 token 委托壳 ↔ log LogTheme 同款(app_theme.rs:5 注释自认),~48 行/份
- 收编:`danqing/src/theme.rs` 提供导出宏(名定 plan 阶段,候选 `delegate_theme!`)
- 迁移:dropshelf、log
- 判据:通用 ✓ / 第二消费者 ✓(2 仓)/ 质量 ✓ / 框架未有 ✓

### 件 4:TextBatch 垂直居中基线 helper(S6)

- 重复证据:同一公式 `(line_h - texts.line_height(px))/2 + texts.ascent(px)` 全家 3 仓 7 处:plainsight text.rs:125-126/table.rs:293-294/358-360/view/mod.rs:44-46;dropshelf shelf_list.rs:258-259/group_picker.rs:111-112;log analysis_panel.rs:562(`vcenter_base`)
- 收编:`TextBatch` 加方法(名定 plan 阶段,候选 `vcenter_baseline`/`baseline_vcenter`)
- 迁移:四仓 7 处(plainsight 仓内 helper 已在复盘快修批先收口,届时换框架件)
- 判据:通用 ✓ / 第二消费者 ✓(3 仓)/ 质量 ✓ / 框架未有 ✓

### 件 5:小件包

- `time::epoch_millis` / `millis_to_epoch`(或 plan 定名):dropshelf store.rs:258-271(14 行)+ luoban app.rs:269-274 各手搓一份 → danqing time.rs
- `encoding::decode_bytes(bytes) -> (Encoding, String)`:整段字节→检测→UTF-16 转码→解码→剥 BOM 组合 API,收编 luoban encoding.rs:16-31 的组合层(~15 行)→ danqing-encoding crate
- plainsight `raw_nul_ratio`(state.rs:446-456)引擎外露 raw 头采样:**本期不做**,引擎 UTF-16 门已加固(2026-10-10 红队),产品侧保险带保留,观察

## Commands(框架仓)

```bash
cargo test --lib --tests
cargo test --features persist
cargo clippy -- -D warnings
cargo fmt
```

## Boundaries

- **Always**:公开 API 一律经 danqing/src/lib.rs re-export;中文文档注释;文件头 @author/@date;三件套全绿;**联动落地链路顺序不能反**:danqing 提交并 push → 产品仓 `cargo check` 最小重解 → 提交 lock
- **Ask first**:每件的产品仓迁移完成后的 push(逐项点头)
- **Never**:破坏框架既有公开 API;改 token 默认值;剪贴板写侧测试碰真剪贴板;`cargo update -p danqing` 宽边重解

## Testing Strategy

- 件 1:toml 读写 round-trip 测试(迁移两仓既有用例+交集边界:引号转义/空串/Unicode)
- 件 2:写侧经注入 fake 测校验重试逻辑(成功/首败重试成/连败报错);读回比对 normalize 后等价
- 件 3:宏展开编译测试 + 双主题委托 token 抽查
- 件 4:基线公式数值测试(奇偶行高两例)
- 件 5:epoch 互转饱和边界;decode_bytes 五编码矩阵(UTF-8/BOM/UTF-16LE/BE/GBK,迁移 luoban 既有用例)

## 迁移顺序(plan 细化,原则:框架先落地,产品后跟进)

1. danqing 侧四件 + encoding 侧一件,各带测试,一次 push(用户点头)
2. 产品仓按依赖顺序迁移:dropshelf(件 1/3/4/5)→ luoban(件 2/4/5)→ plainsight(件 4)→ log(件 1/3/4;**log 在 v1.x 发布窗口期,排期用户定,可挂到发布后**)
3. 每仓迁移后三件套绿 + 本地 commit 注明关联,push 逐项点头

## Success Criteria

- 全家重复净删 ~700 行;四件框架件 + 一件 encoding 件各带测试
- 四仓迁移后行为不变(显示档位/配色/基线像素级一致)
- 复盘报告 S2/S4/S5/S6 标落地

## Open Questions

1. toml 子模块交集范围:两仓现有格式都只用平铺 key=value 还是含 section?(plan 阶段读码定,spec 不预设)
2. delegate 宏命名与委托面:log LogTheme 是否委托了 Theme 全部方法(含 source_palette)?plan 阶段定
3. log 侧迁移排期:随批走还是挂 v1.x 发布后?(建议挂发布后,与 S8 log 侧同窗口)
