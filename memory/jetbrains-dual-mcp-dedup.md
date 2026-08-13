---
name: jetbrains-dual-mcp-dedup
description: "双 JetBrains IDE 导致 MCP 工具双份加载占 ~30k context;deniedMcpServers 在 2.1.220 非托管设置中不生效,唯一可靠解法是 claude mcp remove"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 1986b5b1-9630-4f91-967a-2acfd65fa12a
  modified: 2026-08-03T03:11:02.615Z
---

机器上装了 IDEA + RustRover,两个 IDE 的 Claude Code 插件会各自把 MCP server(`idea` / `rustrover`,HTTP transport,端口动态)注册到 **user scope**(`~/.claude.json` 顶层 `mcpServers`)。两套工具 schema 完全相同,两个 IDE 同时开着时每个会话双份加载,白白占 ~15k×2 tokens。只有一个 IDE 开着时,连不上的 server 工具不加载、不占 context。

实测(2026-08-03,Claude Code 2.1.220,kimi 代理):

- `deniedMcpServers` 放在项目 `.claude/settings.local.json` 和用户 `~/.claude/settings.json` **均被忽略**(debug 日志显示 idea server 照常 Successfully connected);该 key 很可能仅企业托管 settings(`C:\Program Files\ClaudeCode\managed-settings.json`)生效。
- `enabledMcpjsonServers`/`disabledMcpjsonServers` 只管 `.mcp.json` 项目级 server,管不到 user scope。
- 权限 deny 规则(如 `mcp__idea`)只挡调用,不挡 schema 加载,省不了 context。

**How to apply:** 可靠解法只有 `claude mcp remove idea -s user`(或 rustrover)。注意 JetBrains 插件下次从该 IDE 启动 Claude Code 时可能自动重新注册;备用方案是 `claude --mcp-config <file> --strict-mcp-config` 单会话白名单。

**已执行:** 2026-08-03 已在 user scope 删除 `idea`(保留 `rustrover`)。若日后 `claude mcp list` 又看到 idea,是 IDEA 插件自动重新注册的,按需再删。

**更优解(同日落地):** `ENABLE_TOOL_SEARCH=true` 开启 ToolSearch 延迟加载——MCP schema 不预加载、用到时经 ToolSearch 按需加载,理论上 MCP 工具 context 占用 ~30k → 接近 0。已实测 kimi 代理(api.kimi.com/coding)能透传 tool_reference 块,MCP 工具发现+调用端到端可用。已写入 `~/.claude/settings.json` 的 `env` 块持久化。注意:此开关对非一方 host 默认关闭是官方的谨慎策略,若某日 MCP 工具"找不到",先怀疑代理不再透传,撤掉该环境变量即可回退。
