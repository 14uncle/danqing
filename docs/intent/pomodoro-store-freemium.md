# Intent: 丹青-pomodoro 微软商店 freemium 上架

> 2026-09-01 interview-me 确认; 对 [pomodoro-free-release.md](pomodoro-free-release.md) 做渠道加法, 不推翻原决策

## 背景

- 桌景 (danqing-deskscape) spec 中途发现**没有付费点**,「变现岗位」出生证失效 (2026-08-28 启动, 见 [third-product-desk-scene.md](third-product-desk-scene.md))
- 农场需要一个已发布资产承担**首个变现验证**; pomodoro 是唯一候选 (已发布、有分发、有用户)
- 8/10 免费决策的发布渠道为 GitHub + 国内社区; 6 平台文章 (CSDN/知乎/掘金/抖音/小红书/公众号) 承诺「9 个场景全部免费」, 但触达≈0 (0 评论, 2026-09-01 确认) — 「食言」约束力已注销, 双轨制是为不给未来留话柄

## 决策 (双轨制)

- **微软商店渠道: freemium** — 免费版 2 场景 (篝火/海), 内购解锁完整版 (9 场景 + 统计 + 年度报告 + CSV/JSON 导出); 定价口径沿用 ¥18 / 首发 ¥7.9
- **GitHub 渠道: 维持全免费** — Release 继续发 full 版 (9 场景); 已发文章不删不改, 承诺在原渠道继续成立
- **实现**: Cargo features `free`(默认) / `full` / `store` + `src/license.rs` 启动时查商店 add-on 内购许可证 (Offer ID: `danqing-pomodoro-full`)
- 打包: `tools/build_freemium.ps1` (三版本) + `tools/build_msix.ps1` (商店 MSIX, 上传后商店自动签名)

## 成功标准

- 这波完成线: **提交审核并上架**
- 战略闸门: **卖出一份即成功**, 不计时 — 外检落地 (见 [[product-strategy-two-gates]])
- 性质: 管道验证 (代码→上架→收款), 不是收入验证

## 不做的事 (Out of scope)

- Steam 上架 (文案已备 danqing-pomodoro/docs/steam-store-copy.md, 另行)
- 桌景付费点重找 (另行)
- 删改已发文章 / 在文章渠道改口
- 数据同步后端 (延续免费决策的 scope)

## 关键风险与待办

- Partner Center 账号注册 + $19 + 身份验证是关键路径 (仅用户本人可办, 中国区可能数天等待)
- `build_msix.ps1` 的 `PublisherCN` 与 `license.rs` 的 `STORE_URL` 为占位符, 待 Partner Center 回填
- 内购检查必须是 **add-on 级** (`AddOnLicenses` 匹配 Offer ID): 应用级 `IsActive` 对免费应用恒为 true —— 2026-09-01 已修
- MSIX 侧载安装实测通过后上传
