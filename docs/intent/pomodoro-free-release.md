# Intent: 丹青-pomodoro 免费发布

> 2026-08-10 interview-me 确认; 2026-08-10 v0.1.0 首次发布

## 愿景

丹青-pomodoro 作为免费产品发布,走通"代码→发布→社区反馈"完整闭环。这是第一次个人项目练手——AI 编码、产品发布、社区维护、个人品牌——全部免费是因为学费比收入重要,真正的变现留给下一个产品。

## 决策

- **全部功能免费**,零门槛,不设付费分层
- **废弃** `docs/specs/companion-flagship-pricing.md` 中的付费方案
- **去掉"旗舰"品牌词**——免费产品叫旗舰容易让人觉得有付费陷阱
- 发布渠道: **GitHub + 国内社区**(V2EX、稀土掘金)试水
- 成功标准: **走通完整流程** + 收到真实用户反馈
- 引擎复用给下一个产品(剪贴板等),pomodoro 是学费不是收入来源

## 发布记录

- **v0.1.0** (2026-08-10): https://github.com/14uncle/danqing-pomodoro/releases/tag/v0.1.0
  - 独立仓库: https://github.com/14uncle/danqing-pomodoro
  - Windows x64 便携包(30MB, 含 9 场景图 + 环境音)
  - 发布渠道: V2EX + 稀土掘金
  - v0.1.0 修复: 报告面板标题行布局(移除空 UiBox, commit c7bc91c)

## 不做的事(Out of scope)

- 付费变现(买断/订阅)
- 数据同步后端
- "旗舰"品牌词
- 剪贴板作为第二件产品(顺延)
