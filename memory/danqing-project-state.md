---
name: danqing-project-state
description: 丹青(danqing)Rust UI 框架里程碑状态(M1~M3 + 阶段 1 + 阶段 2 + 打磨三件套 2026-07-28 + 五场景动效 2026-07-30 全检关闭)、计划文件位置与已定决策、2026-08-01 著作型旗舰十年战略 + 山/森林动效迭代 lessons learned
metadata:
  node_type: memory
  type: project
  originSessionId: 4cad717e-4f66-4be2-bc93-bc02c8f26405
  modified: 2026-08-11T14:34:31.615Z
---

丹青(danqing,F:\github\danqing):Rust 跨平台自绘 UI 框架,winit 0.30 + wgpu 30,保留模式组件树。M1 目标:开窗+图元/文本绘制+键鼠事件+showcase;M2 目标:焦点系统+单行文本输入+剪贴板+IME;M3 目标:滚动容器 Scrollable + 多行文本域 TextArea + 渲染裁剪 + 鼠标拖拽选区。阶段 1(设计系统 + 品牌视觉)**已于 2026-07-23 终审关闭**(提交 8f05bde;玉色 accent、破框朱砂 LOGO、毛玻璃浅色主题落地,spec 十条验收 10/10)。2026-07-22 accent 定为深青绿/玉色 #0F766E(丹青矿物色,朱砂仅作 logo 品牌点睛);2026-07-23 美学方向转向潮汐式场景沉浸、首个 POC 定为专注(番茄钟),见 [[danqing-strategic-positioning-efficiency-tools]]。**阶段 2(番茄钟 POC)已于 2026-07-23 全检关闭**(7 任务 + 人工终审,末提交为 todo 收口 commit;spec Success Criteria 7/7 + 用户终审"漂亮有质感")。**下一步候选:第二 POC 剪贴板历史管理器**(效率工具族,美学剂量低于专注陪伴族,见 [[danqing-strategic-positioning-efficiency-tools]]),或用户另行指定。已知观察已决策:雨场景主按钮 base-on-accent 对比 ~2.9:1 低于 3:1,用户 2026-07-23 明确"不需要调深",接受现状不再处理。场景打磨(SSAA 抗锯齿+雨丝可见化+海面正弦浪层,提交 ace9a9d)同日经用户 1080p 最大化四场景截图人工终审通过。关键架构:App trait 已有 `tick()`(每帧心跳)与 `background_frame()`(场景→渲染通道)默认方法;背景管线双纹理 mix 交叉淡化;SceneFader 纯逻辑在 example;Center 逐轴 tight + `fill_max()` 显式占满;fill 子项交叉轴宽松是刻意的(定高色块案例)。

- 规格统一在 `docs/specs/`:M1 `docs/specs/spec.md`(已批准),M2 `docs/specs/spec-m2.md`(M2 已实现),M3 `docs/specs/spec-m3.md`(M3 已实现),山/森林 `docs/specs/pomodoro-scene-motion-mountain-forest.md`;计划统一在 `tasks/`:M1 `tasks/plan.md`,M2 `tasks/plan-m2.md`,M3 `tasks/plan-m3.md`,山/森林 `tasks/archive/plan-pomodoro-scene-motion-mountain-forest.md`;进度勾选 `tasks/todo.md`(M1 关闭)、`tasks/todo-m2.md`(M2 关闭)、`tasks/todo-m3.md`(M3 关闭)。
- 已完成:M1 全部 14 个任务已关闭,M2 5 个任务已关闭——2026-07-16,M3 8 个任务已关闭——2026-07-18。`cargo test`/`clippy -D warnings`/`fmt --check`/`build --release` 全绿,showcase 人工运行首帧渲染无错误。
- 已定决策:值类型放 `src/layout.rs`;App 用 Elm 风格 `update(msg)+view()`;回退字体已改为 assets/ 目录提交(OQ1 决策已被取代),见 [[danqing-assets-directory-convention]];showcase 是唯一持续生长的示例(以用代测);M2 新增 `arboard` 剪贴板、焦点默认按组件树深度优先、单行 TextInput;M3 新增 `Scrollable`、`TextArea`、渲染 clip stack、焦点命中裁剪;五场景动效"每效果一标量 uniform"(非互斥选择子,交叉淡化可同时非零)。
- 公开 API 一律 `lib.rs` re-export;`widget/`、`layout.rs`、`event.rs` 必须纯逻辑;公共类型写中文文档注释;提交前 `cargo fmt` + `cargo clippy -- -D warnings` + `cargo test` 全绿。
- 构建环境有坑,见 [[windows-gnu-toolchain-lld-fix]]。

## Next Step

**番茄钟打磨三件套已于 2026-07-28 全检关闭并归档**(tasks/archive/{plan,todo}-pomodoro-polish.md):场景环境音(rodio 0.22 懒初始化+双槽+静默降级,5 场景 CC0 OGG 响度统一 -28 LUFS)、长休息+轮次(4 轮 Focus→15min LongBreak,skip 不计)、今日完成计数(跨日归零持久化)。benchmark 双门槛 PASS(693.8ms / 177.5MB),人工终审用户确认全过。踩坑: rodio 0.22 repeat_infinite 对 symphonia 解码器秒空无声,自实现 LoopingDecoder,见 [[rodio-022-repeat-infinite-bug]](2026-08-27 已迁至 danqing-pomodoro/memory/)。

**2026-07-28 雨场景动效试点已关闭并归档**(tasks/archive/{plan,todo}-pomodoro-scene-motion.md,spec docs/specs/pomodoro-scene-motion.md 验收 8/8):雨场景程序化雨丝(background.wgsl 三层 hash 叠加,uniform 复用 16B pad 位,零新资产),计时运行下落、暂停 500ms 沉降(motion.rs MotionEnvelope,视觉独立时长不复用音频 300ms)。终审调参三轮:方向对齐静态图(雨落朝右下,\ 形)、去流星感(尾羽 x2.5/亮度降/速度减半,公共周期 8s)、线宽 2-3px(列密度 480/360/320)。benchmark PASS(891.4ms/180.6MB)。

**2026-07-29 篝火场景动效已关闭并归档**(tasks/archive/{plan,todo}-pomodoro-scene-motion-bonfire.md,spec docs/specs/pomodoro-scene-motion-bonfire.md 验收 8/8):光晕呼吸(3 正弦乘性 ±8%)+ 余烬上浮(7px 热黄圆点 ~24 颗,用户终审指定点径)。uniform 16B→32B 每效果一标量(雨/火并存,交叉淡化可同时非零,非互斥选择子),RAIN_WRAP_SECS 更名 MOTION_WRAP_SECS,火效频率全落 1/8 Hz 整数倍保 8s 公共周期。剂量教训:暗启动首版(亮度 0.5/±4%)在亮橙辉光上对比不足不可读,自检 1 轮提亮 + 目测 1 轮(5px)+ 用户指定(7px)收口。benchmark PASS(587.7ms/178.5MB)。

**2026-07-29 海场景动效已关闭并归档**(tasks/archive/{plan,todo}-pomodoro-scene-motion-sea.md,spec docs/specs/pomodoro-scene-motion-sea.md 验收 8/8):波带 UV 纵向位移起伏(采样坐标本身动,±0.015 uv,天空 mask 为 0、近水动得多)+ 波光碎点原地明灭(乘性提亮 ~14 颗)。uniform 32B 填 pad 不扩容。剂量教训重大:亮度乘性调制三轮加码(±4%→±35%)均读作"明暗对象沿静态波形移动的车,路没动",用户裁定换 UV 位移机制一轮通过,见 [[scene-motion-uv-displacement]]。benchmark PASS(858.3ms/178MB)。

**2026-07-29 雨场景改造已关闭并归档**(tasks/archive/{plan,todo}-pomodoro-scene-motion-rain-rework.md,spec docs/specs/pomodoro-scene-motion-rain-rework.md 验收 8/8):静态雨图去丝(export-scenes.py 雨配置去 streaks,rain.png 206KB→89KB,其余 4 场景字节不变)+ 程序化雨幕独挑(有雨列门槛 0.70/0.72/0.85 ~290 丝,丝宽保 2~3px)+ 暂停雨钟定格可见(rain_intensity 去包络,新增 rain_clock 由包络推进下落时间,uniform 第 7 槽 pad0→rain_time)。机制教训重大:UV 位移对沿自身轴的元素不成立(画中丝沿线滑动除丝头外无可读变化),摆动剪切/双相位流图两版试错后用户裁定"静态图去丝,运行时动态渲染",见 [[scene-motion-uv-displacement]] 修订。benchmark PASS(786ms/179.8MB)。

**2026-08-01 山/森林场景动效已人工终审通过并归档**(tasks/archive/{plan,todo}-pomodoro-scene-motion-mountain-forest.md T5 已勾,spec docs/specs/pomodoro-scene-motion-mountain-forest.md 验收 8 注✅):山单层暖粉融入暮色(alpha 0.45 + 雾色 (0.92,0.65,0.62) + mask 0.50-0.88 集中山脊上空 + speed 0.0625+ u.rain_time),**森林单层雾**(fore_mist 单 mist_pattern,SPEED 0.0625 / SCALE 2.0 / ALPHA 0.25,**副层 LAYER_B2 已去掉**——终审时用户纠正,勿再用"2 层"描述)。uniform 32B→36B→48B(删 pad0 + 加 mountain_intensity + forest_intensity + 保留 rain_time)。代码评审通过(0d4fcdb 删死代码 3 函数 89 行,448→359 行)。benchmark startup 762ms/WS 274MB。**终审另一修正: 无 1-5 场景快捷键**——场景切换仅 ◀/▶ 按钮,全局热键只有显隐/暂停/退出 3 个。

**山/森林迭代 lessons learned(2026-07-30)** — 视觉迭代 18+ commit,踩了几个值得记的坑:

1. **wrap-clean 数学易错(commit c325833)**: speed=0.0625 看似 wrap-clean (8×0.0625×k=k/2 整数),但 sin(k/2·π)≠0 因为 π 是无理数,实际 sin(3)=0.14 仍有 5% 跳变。**正确修法是 u.rain_time**(非 wrap,持续累加,f32 25 min 安全,78c9cf3)。后续所有"持续漂移"类动效直接用 u.rain_time,不纠结 wrap-clean 数学。
2. **alpha 阈值经验(山, 单层暖粉)**: 0.10-0.15 不可见(用户嫌"看不清楚")→ 0.20-0.30 微妙可见(用户嫌"看不清楚")→ 0.30-0.45 明显可见(本次目标)→ 0.45-0.55 强势但易读作"云团"→ ≥0.55 读作"独立云团"。alpha 上限 0.50,超过破"暮色融入"语义。
3. **sum-of-sines vs value noise 选择**(森林, 大尺度连续视觉): sum-of-sines 流畅波纹但有周期性(LCM 重复 + 传送带感),value noise 无周期但**cell 边界可见方块感**让"雾"读作"马赛克"。雾选 sum-of-sines,雪/小颗粒选 value noise。
4. **多动效遮罩宽度经验**(山/森林, "融入背景"类型): 屏占比 55% (0.40-0.95) 太宽弥漫, 38% (0.50-0.88) 聚焦融入, 40% 临界易读作"条带"。≤屏高 40% 为宜。
5. **UV 位移 vs 静态烘焙 vs 程序化(山/森林 雾)** — 雨场景改造范式:静态图去烘焙 + 运行时程序化全取代(forest.png 去 mist 字段),遵循 [[scene-motion-uv-displacement]] 的"沿轴 UV 位移对离散元素不成立"教训。
6. **18 commit 视觉迭代的耐心成本**: 同一区域 18+ commit,用户每次反馈"还要调",需在 5-7 轮内识别根本问题(本次问题反复是"周期性 / 视觉" 反复出现,根因是 sum-of-sines 周期性),5 轮后还看不出根因 → 切换抽象层次(从调参换到换函数/换范式)。**视觉迭代应预算 5 commit,超过 5 commit 同主题还无收敛 → 提级反思根本问题**。

**下一步(用户 2026-07-30 反馈)**: 山/森林等用户人工终审后即收尾。 之后候选:第二 POC 剪贴板历史管理器(效率工具族,美学剂量低于专注陪伴族),见 [[danqing-strategic-positioning-efficiency-tools]];**未获用户指示不要启动新 POC**。

**2026-08-01 战略升级:著作型旗舰(十年意图)已确认**,见 [[danqing-flagship-strategy]]。方向从"做完剪贴板 POC"改为:把番茄钟经营成十年付费旗舰(专注陪伴系统 × 十年建造史)。意图 `docs/intent/companion-flagship.md`,路线图 `tasks/plan-flagship-roadmap.md`。

**2026-08-01 里程碑 0「旗舰化第一刀」A+B+C+D 全部完成**: A 山/森林终审通过、B 付费边界 spec 确认、C 数据层 MVP、D 建造实录三篇草稿,见 [[danqing-flagship-strategy]]。剪贴板降级为引擎复用验证,顺延。遗留:付费门禁/数据同步后端(非本期)、建造实录发布(用户,仓库外)。

**2026-08-01 里程碑 1「沉浸世界定位落地」启动, Task E 年度报告完成并提交**: 竞品定位校验(interview-me)裁决「桌面上的专注世界」→ 用体验层回应功能型番茄钟阵营,产出 `docs/ideas/pomodoro-competitor-memo.md`;年度报告 = 旗舰版(用户裁定,原始数据+基础统计免费,深度洞察付费)。Task E 落地 `stats.rs` year_summary/month_trend 纯读聚合 + `main.rs` 报告面板(本年汇总/场景分布/近 12 月趋势),commit 07657e6。路线图 Task E-I: E ✅ / F 深度定制 / G 实录发布 / H 同步 spec / I 沉浸世界补全(2026-08-01 执行后收敛,见下)。

**2026-08-02 里程碑 1 推进**: Task F 深度定制完成(环境音开关移入设置面板 + theme scrim/radius_xl token,commit 0b05593/91e0273)、Task H 数据同步 spec 确认(`docs/specs/companion-flagship-sync.md`,commit 376b25c: 同步仅 sessions、session_id 前置演进、订阅专属)、Task I 沉浸世界九场景扩展(铁匠铺/洞穴/夜市/火车,commit e2c9693)。最终阵容 9 场景: 篝火/海/雨/山/森林/铁匠铺/洞穴/夜市/火车。

**2026-08-01 框架 Box::layout bug 修复**(commit 548322c): 单维显式宽/高未收紧子级约束,含 fill 弹性项的子级按父约束上限扩张溢出(统计面板自里程碑 0 即带此 bug,被零数据掩盖;报告面板 + 12 条会话暴露,数值画到 x=1220 窗口外)。修复: Box::layout 把子级 max 钳到自身显式尺寸(min 一并钳制),回归测试 `box_with_explicit_width_constrains_child_to_width`。报告面板趋势 12 行改小字号容纳窗口高度(655→604)。**教训: 数值"画了但看不见"优先查约束/布局越界,headless glyph-bounds 诊断可定位**。

**2026-08-09 四新场景动效已提交**(commit e2c9693): 铁匠铺/洞穴/夜市/火车四个场景的 shader 动效 + Uniform 对齐修复。修复了 starry 退役字段导致的 uniform 数据错位(starry_intensity/starry_base 占位 0.0 对齐 shader)。火车场景最终版本:雨滴由上往下滑落(简单垂直运动,不追求物理模拟) + 车厢暖光呼吸。教训: 物理模拟(抛物线轨迹+划痕)在视觉迭代中效果不如简单垂直滑落,用户要求"改回简单版本"。

**2026-08-10 pomodoro v0.1.0 首次发布**: 独立仓库 https://github.com/14uncle/danqing-pomodoro (public), Windows x64 便携包(30MB)。从 danqing 框架 examples/pomodoro 提取为独立项目,全部功能免费。发布渠道: V2EX + 稀土掘金。意图落盘 `docs/intent/pomodoro-free-release.md`。所有"旗舰"引用已清理(代码+文档),付费 spec 已标注废弃。v0.1.0 修复: 报告面板标题行布局(移除空 UiBox)。
