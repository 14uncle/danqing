# 三新仓复盘：引擎复用与下沉候选(2026-10-10)

对象：danqing-dropshelf(5839 行)/ danqing-luoban(3523)/ danqing-plainsight(2687)
对照：danqing 框架(34285)+ danqing-encoding + danqing-logfile + danqing-log(跨产品比对用)
方法：5 路只读审计(框架编目 + 三仓逐模块审计 + log 授权编目),全部结论带 file:line 证据
判据：沿用 `framework-sinking.md` 下沉四条——通用 / 第二消费者 / 质量 / 框架未有

## 一句话结论

引擎复用主干健康(窗口/渲染/文本测量/编码/CSV/拖放/主题 token 基本全走框架),但有三处系统性漏风:
① `persist`/`update` feature 默认关,dropshelf 与 luoban 齐刷手搓了等价物;②授权栈已是全家第三份同构拷贝,框架没有授权模块;③ `AsyncJob` 收编后零消费(log 没迁移),plainsight 又写了第四份。下沉头号候选 = **授权栈**。

## ① 造轮子清单(框架已有,产品自造 → 应改用框架)

| # | 仓 | 自造了什么 | 框架已有 | 证据 |
|---|---|---|---|---|
| 1 | dropshelf | `write_atomic` 17 行 | `persist::atomic_save` | store.rs:353-369 ↔ danqing/src/persist.rs:78-86 |
| 2 | dropshelf | `default_dir` | `persist::config_dir` | store.rs:253-255 ↔ persist.rs:53-60 |
| 3 | dropshelf | 位置记忆防抖 ~20 行 | `persist::DirtyFlag` | main.rs:205-208/1063-1070/1082-1090 ↔ persist.rs:144-202 |
| 4 | dropshelf | `rel_time` 4 档 12 行 | `time::relative_time` 5 档 | shelf_list.rs:494-505 ↔ danqing/src/time.rs:13-35 |
| 5 | luoban | `default_autosave_dir` 手搓 %APPDATA% | `persist::config_dir` | app.rs:261-267 |
| 6 | luoban | autosave/导出非原子写 ×4 | `persist::atomic_save` | app.rs:201/205/209/210(**兼质量单**,见 §4) |
| 7 | luoban | 剪贴板读侧 ~15 行逐行同构 | `clipboard::ClipSource` | clipboard.rs:26-30/42/46 ↔ danqing/src/clipboard.rs:56-59/77/81 |
| 8 | luoban | `push_wrapped` 手写列折行 25 行 | `text::break_lines`(measure 回调可表达) | preview.rs:155-178 ↔ danqing/src/text/line_layout.rs:47(软性重复,语义有差) |
| 9 | plainsight | 后台扫描线程(thread+Mutex+catch_unwind) | `job::AsyncJob::launch_catched` | state.rs:187-199 ↔ danqing/src/job.rs |
| 10 | plainsight | `HEAD_SAMPLE` 常量 | `danqing_encoding::SAMPLE` | state.rs:13-14 ↔ danqing-encoding/src/lib.rs:43 |

根因共性:#1/2/3/5/6 全是 **persist feature 未开**(两仓 Cargo.toml 均无 features 声明),能力在仓外睡大觉。#9 是家族旧账——框架 job.rs 模块头自述「农场重复发明 4 处」收编,但 log 至今用本地副本(danqing-log/src/search.rs:16),新仓又绕开。

## ② 下沉候选(跨产品重复 → 框架/兄弟 crate)

### S1 授权栈【头号,判据四占】

现状:三份同构拷贝——pomodoro license.rs(580,商店侧鼻祖)→ log license.rs(735)+store_license.rs(246)+bin/keygen.rs(177)→ dropshelf license.rs(484)+store_license.rs(252)+bin/keygen.rs(179)。`find_main_window` 三仓逐字同构;`verify_key`/`build_payload`/`Payload`/`Entitlement`/`mask_email` 结构一致仅换常量;两仓 keygen 近乎双胞胎。

- 判据:通用 ✓(Ed25519 + base64url + WinRT broker 全通用)/ 第二消费者 ✓(已三仓)/ 质量 ✓(log 版含 ~365 行测试)/ 框架未有 ✓
- **形态建议:新兄弟 crate `danqing-license`**(零 UI 依赖,照 encoding/logfile 前例,可独立发布);WinRT store broker 部分加 feature 门(windows crate 依赖重)。产品侧只注入:`KEY_PREFIX`/`PRODUCT`/公钥/`OFFER_ID`/`PURCHASE_URL`/`Feature` 枚举(泛型或 trait 参数)。keygen 收编为该 crate 的参数化 bin(prefix/product 走 CLI 参数),私钥纪律不变(仓库外)。
- 下沉 spec 必须统一的三处语义分叉:
  1. `ExpirationDate` 读失败:log fail-open 朝付费(log store_license.rs:103-106,**已知洞**)vs dropshelf fail-closed(:103-110)→ 取 fail-closed,log 顺带修洞;
  2. `activate` 落盘:dropshelf 原子写 vs log 裸 `fs::write`(log license.rs:318)→ 取原子写;
  3. payload `expires` 执行口径(dropshelf license.rs:58-59 解析不执行)→ spec 定。
- pomodoro 已上架且是 compile-time `full` 双轨变体 → **建议不动**,迁移列可选项。
- 安全面:公钥常量内嵌产品侧不变;verify 侧代码本就随产品公开,无新增泄漏面。

### S2 手写 TOML 配置【判据四占】

log config.rs:1-391 手写朴素 TOML(:9-10「不引 toml 依赖」)↔ dropshelf store.rs:121-250 同模式 ~130 行。建议进 `persist.rs` 增设 toml 子模块,两仓迁移。

### S3 RowList 行列表族【判据四占,但设计重】

dropshelf 仓内两份同范式(shelf_list.rs:597 + group_picker.rs:165;ROW_H/`row_at`/基线公式/theme 快照块逐字族);log 的 RowList 已有三个仓内消费者(workspace-sessions 时成第三);框架无此组件。另有更重的「行锚定虚拟视口」族(log LogView → plainsight text.rs/table.rs 明言「照 LogView 家训」,框架 Scrollable 像素偏移不适用 5M 行)。**建议本期只做轻 RowList;虚拟视口观察**(plainsight table.rs:46-78 的 `col_window`/`build_spans`/`hit_col` 是比 log 更干净的独立 helper,可作未来种子)。

### S4 剪贴板写侧【旧账 E3 半落地,框架肌肉缺口】

旧下沉盘点簇 E3「多格式剪贴板写入+to_crlf」:`to_crlf` 已落(text/),写侧始终未落。luoban 自填 `set_html` 双路写 + `copy_html_verified`(写后读回校验+重试)+ `normalize`(clipboard.rs:33-38/55-75/78-80,~45 行+测试)。建议收进 `danqing/src/clipboard.rs`(读侧驻地,天然一家);luoban 迁移后 ①#7 的读侧重复一并消。

### S5 delegate! Theme 委托壳【小件】

log 的 LogTheme 与 dropshelf app_theme.rs:28-75 同款宏壳 ~48 行/份。建议进 theme.rs。

### S6 垂直居中基线公式【小件】

全家 3 仓 7 处同式 `(line_h - texts.line_height(px))/2 + texts.ascent(px)`:plainsight text.rs:125-126/table.rs:293-294/358-360/view/mod.rs:44-46;dropshelf shelf_list.rs:258-259/group_picker.rs:111-112;log analysis_panel.rs:562(`vcenter_base`)。建议 `TextBatch` 加 helper。

### S7 rfd 对话框【撞纯度政策门,待裁决】

全家 3 仓 6 处直用 rfd(log×3 / dropshelf×1 / plainsight×2)。框架无对话框原语(rfd 仅是框架 dev-dep)。下沉=框架引外部 UI 依赖 → 选项:a) 薄封装进框架 feature 门;b) 维持产品直用 + 家族模板注释。**待裁**。

### S8 AsyncJob 迁移【不是下沉,是收编验收补票】

框架 job.rs 收编时自述「重复发明 4 处」;现实:log 仍用本地副本(search.rs:16),plainsight 新写第四份(state.rs:187-199,「共享状态轮询」形态,无代次语义,与框架 poll 拾取形态有差)。建议:log+plainsight 迁框架;plainsight 形态若不适配,spec 时给 job.rs 补变体或判合理自造。**流程教训:下沉验收应含「既有消费者迁移」闸,否则收编空转**——建议写进 framework-sinking.md 使用方式节。

### 小件包

- epoch 毫秒互转:dropshelf store.rs:258-271 + luoban app.rs:269-274 → `time.rs` 补 API
- luoban `decode_bytes` 组合层(~15 行,encoding.rs:16-31)→ danqing-encoding 补「整段字节→String」API
- plainsight `raw_nul_ratio`(state.rs:446-456)= 引擎缺口产品层补丁(UTF-16 转码后原始字节不可得)→ 可选:引擎外露 raw 头采样

### 已裁定不动 / 已证伪(防重复提议)

- `wheel_lines` 滚轮换算两份(log main.rs:1049 ↔ plainsight text.rs:44):普查 G10 已裁「框架不归一 delta」,**不动**
- luoban encoding.rs:合规薄封装,检测零自撸 ✓
- plainsight mmap/CSV/编码/文本测量/省略/token:全走引擎,零自造 ✓;`DocSwitch` vs `MultiPanel` 近失配(差「未启用侧不 sync」一点),观察
- dropshelf 主题纪律达标(零自定义色,delegate! 全委托)、五肌肉全消费 ✓
- StatusBar:log 内联 + plainsight view/status.rs 两种形态未稳,观察
- 仓内重复(各仓自行消化,不下沉):plainsight `temp_file`×3(state.rs:463-467/app/mod.rs:234-238/frame.rs:281-285);dropshelf 测试 sandbox 双胞胎(store.rs:450-456/transfer.rs:412-418)+ rand_suffix/temp_name 三处;luoban FakeClip×2(clipboard.rs:87-122/app.rs:368-399)+ epoch_millis 两路(app.rs:175-178 没调自己的 :269-274)+ CRLF 归一两种口味(clipboard.rs:78-80/ir.rs:109)+ 公式/图片占位文本三面口径(render.rs:243-246/docx.rs:167-168/ir.rs:276)

## ③ 系统性发现(流程层)

1. **persist/update feature 默认关 → 两仓齐踩**:dropshelf+luoban 都没开 persist,手搓 4 件等价物。选项:a) persist 改默认开;b) 新仓 checklist(打包节既有)加「feature 评估」一步。建议 b 保守。
2. **收编件零消费**:见 S8,下沉流水线加「消费者迁移」验收闸。
3. **plainsight 工作树脏态**:`.cargo/config.toml` patch 当前开着(仅 logfile)且 `Cargo.lock` 被 rewrite 成 path 态未提交(committed 态锁 logfile#dc017a0)——落地链路纪律的现前实例,需收口(关 patch、lock 复原或提交)。

## ④ 质量单(非复用问题,复盘顺抓)

- **luoban autosave 非原子写**(app.rs:201/205/209/210)vs F4「不丢稿」卖点直接冲突 → 随 ①#6 一并修(开 persist 用 `atomic_save`)。
- **luoban Theme token 零使用**:preview.rs:181-192 硬编码 8 组色+字号(注释自述有意镜像输出 HTML,部分可辩)+ app.rs 间距硬编码 gap(12/8/16/4)——违「token 走 Theme 默认值」约定,待裁:修 or 记例外。
- dropshelf ui.rs:155 `preset_button` 字号静态取 LightTheme 而颜色 bind 双主题(轻微不一致)。
- dropshelf CLAUDE.md 结构节「空骨架」标注已过时(代码已建)。

## ⑤ 建议行动序列(每项经你点头后,下沉类走 spec 五阶段)

**快修批 + feature 开启批 + plainsight AsyncJob 迁移已落地(2026-10-10,用户「按推荐 go」)**:
- dropshelf `411a7a3`(开 persist,删 write_atomic/config_dir/位置防抖手搓,rel_time→框架五档)+ `569c8ec`(CLAUDE.md 结构节)——84 测试绿,本地 commit 未 push
- luoban `ed0be53`(开 persist,autosave/导出/handoff 全换 atomic_save,配置目录走 config_dir,新增「不丢稿」锁测试)+ `d6a9d9e`(CRLF 单出口/FakeClip 合一/epoch_millis 单出口)——100 测试绿,本地 commit 未 push
- plainsight `14f25fc`(patch 收口+lock 复原)+ `82e2fcf`(HEAD_SAMPLE 换 SAMPLE/后台扫描换 AsyncJob::launch_catched)+ `b6298e7`(temp_file×3 合一/vcenter 收口/CLAUDE.md)——47 测试绿,本地 commit 未 push
- 农场 `CLAUDE.md` 新仓 checklist 加第 7 步(评估 danqing features)
- **未含**:dropshelf 的 build.rs/winresource/export-logo/acceptance-pending 四条改动(另一条发布链在途工作,原样保留未提交);log 侧 AsyncJob 迁移(log 在 v1.x 发布窗口期,search.rs 心脏区,排期用户定)

**已批 (2026-10-10 用户「S1+B 批准」)**:
- S1 授权栈:计划已批 → **建仓 `danqing-license` 成稿(T0–T5,`e843f24`)**;spec/plan 已迁入该仓(danqing 侧留指针防双份漂移);**闸 A 通过, T6 公开仓 `github.com/14uncle/danqing-license` 已建并 push (master 单分支)**;消费方迁移 T7 dropshelf / T8 log 未开始
- B 批(S2/S4/S5/S6+小件包):**Phase 1 框架侧五件 + Phase 2 三仓迁移全闭(2026-10-10)** —— danqing `52cbfc8..9c25929a`(dev 已推) + danqing-encoding `dcc58cb..573ad91`(master 已推);迁移: dropshelf `d6f7ae4`(84 绿) / luoban `42c0648`(95 绿, encoding.rs 整模块删 —— 零生产调用点) / plainsight `0a9556d`(47 绿), **三仓迁移 commit 均未 push**;log(T10)挂 v1.x 发布后。双路评审零 Critical,红队实锤 F1(写侧不转义换行会写出非法 TOML 丢数据)/F5(attempts 下限无锁)已修
- S3 RowList(设计重,殿后);S8 log 侧迁移排期;S7 rfd(裁决=维持产品直用)

## ⑥ 待裁决清单

- A. S1 授权栈:进 spec?形态=新兄弟 crate `danqing-license`(建议)还是 danqing 模块 feature 门?
- B. S2/S4/S5/S6 是否打包一批进 spec(中小件合一条流水线)?
- C. S3 RowList 本期做 or 观察?
- D. S7 rfd 下沉 or 维持产品直用?
- E. persist feature 默认开 or checklist 加评估步?
- F. 快修批+feature 开启批是否现在动手(涉三仓,不 push)?
- G. S8 AsyncJob 迁移排期(log 侧 search.rs 在心脏区,动它影响面大)?
