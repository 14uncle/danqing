# Spec: 三新仓指南差距修复(2026-10-13 扫描批次)

> 存档理由: 本批次跨 danqing / dropshelf / luoban / plainsight / 农场根五处,
> 跨仓文档成例落 `danqing/docs/intent/`(参照 retro-trio-reuse-2026-10-10.md)。
> 依据: 2026-10-13 拿 PRODUCT-DEV-GUIDE.md 扫描三新仓的差距报告(会话产出)。

## Objective

2026-10-13 指南扫描发现: 三新仓建仓面(B1–B8)全合规, 但有一项**正在错着**——
B9 事故态: dropshelf/luoban 自绘标题栏静默画母 logo「破框朱砂」(danqing `LogoKind`
无两仓分支, dropshelf 未接 `logo_kind`, luoban `logo_name`/`logo_kind` 双缺),
即 B9 条款(当日从 plainsight 实录立项)描述的同款事故, 肉眼可见、测试抓不到。

本批次目标:

1. **B9 事故态清零**: danqing 加 Dropshelf/Luoban 两分支, 两仓升 rev 完成双接线+锁测试
2. **dropshelf 手搓清理**: `dirs::config_dir()` 手搓改走 `persist::config_dir`,
   `dirs` 依赖整砍(A5); 自绘件硬编码色板按用户裁决落地(A4)
3. **杂项收口**: luoban 删 `.cargo/config.toml` 残留(B5); plainsight `.gitignore`
   补三条目; 根 CLAUDE.md 仓库地图+路由收录三新仓(P3)

非目标(明确不做):

- P2 发布前闸项(关于页/更新检查/截图/框架复用表)**只入账不实施**——三仓均未发布,
  属发布时闸门; 入账载体 = 各仓 `tasks/release-checklist.md`(dropshelf 需新建)
- 不发布任何产品; 不动 plainsight 发布 hold(10-21); 不碰 danqing-license;
  不开 local-patch(升 rev 期间尤其保持默认关)

## 修复分组(Phase 0 轻量结论, 代 capability map)

| 组 | 内容 | 涉及仓 | 独立验收点 |
|---|---|---|---|
| G1 B9 接线 | danqing 两分支+几何锁 → push → 两仓升 rev → 双接线+接线锁 | danqing, dropshelf, luoban | 锁测试绿; 肉眼项记账 |
| G2 dropshelf 手搓 | config_dir→persist 砍 dirs; 色板下沉框架(裁决②) | dropshelf + danqing | 测试绿; `dirs` 依赖消失 |
| G3 杂项 | luoban 删残留; plainsight gitignore; 根地图 | luoban, plainsight, farm01 | 逐项核对 |

顺序: danqing 侧改动(LogoKind 两分支 + 色板 token 下沉)合并**一次提交一次 push**,
dropshelf/luoban 只升一次 rev(A6 链路顺序不能反); G3 无依赖可穿插。

## Tech Stack

- Rust edition 2024, 工具链 stable-x86_64-pc-windows-gnu(三仓 override 均已设, B1 ✅)
- danqing: `src/widget/title_bar.rs` `LogoKind` 枚举(现 Default/Pomodoro/Clipboard/Log/Plainsight);
  Plainsight 分支(2026-10-13 落)为本批成例——几何与产品 SVG 手工同步 + 几何锁测试
- 产品侧接线成例: plainsight `src/app/mod.rs:56,67,437`
  (`title_logo_kind()` 纯函数 + `.logo_kind(...)` 接线 + 接线锁测试双断言)

## Commands

```bash
# 各仓内(danqing / danqing-dropshelf / danqing-luoban / danqing-plainsight):
cargo test                              # 全绿
cargo clippy --all-targets -- -D warnings
cargo fmt

# 升 rev 手法(记忆 danqing-dep-lock-minimal-reresolve):
# danqing push 后直接 cargo check 驱动重解; 禁止 cargo update -p danqing
# (会把 windows 边错配炸 wgpu-hal, 2026-09-08 实测);
# 若已炸: cargo update -p gpu-allocator 外科修复宽边(2026-09-22)
```

## Project Structure(将触碰的文件)

```
danqing/
  src/widget/title_bar.rs        # LogoKind + Dropshelf/Luoban 两分支几何 + 几何锁测试
  src/theme.rs                   # (裁决②) 列表件色板 token 下沉: Theme 已有等价物则零新增,
                                 # 无则新增组件 token; 与两分支同一次提交+push

danqing-dropshelf/
  src/main.rs                    # title_logo_kind() 纯函数 + .logo_kind() 接线;
                                 # save_paste_image (main.rs:716 起) dirs::config_dir() → persist::config_dir
  Cargo.toml                     # 删 dirs 依赖(全仓唯一用法在 main.rs:720, 改后零引用)
  Cargo.lock                     # 复钉 danqing 新 rev (纯 git 态, 无 [patch])
  tasks/acceptance-pending.md    # 标题栏 logo 肉眼验收记账(C3)
  tasks/release-checklist.md     # 新建: P2 发布前闸待办入账(F5 载体)

danqing-luoban/
  src/main.rs                    # WindowConfig 补 logo_name: "luoban" + title_logo_kind() + 锁测试
  Cargo.lock                     # 复钉 danqing 新 rev
  .cargo/config.toml             # 删除(磁盘残留, 全文注释无效果, 未入 git; B5 默认态=关)
  tasks/acceptance-pending.md    # 标题栏 logo 肉眼验收记账

danqing-plainsight/
  .gitignore                     # 补 tools/sdk-tools/ *.secret *.license.key 三条目(对齐另两仓)

farm01/CLAUDE.md                 # 仓库地图补三行 + 路由补三条(dropshelf/luoban/plainsight)
```

## Code Style

- 全家约定: 中文注释; 新 `.rs` 文件头 `//! @author 十四叔` + `//! @date 2026/10/13`(本批预计不新增 .rs 文件)
- LogoKind 分支几何: 照同文件 Plainsight 分支画法(玉色窗 + 产品专属朱砂位家族语法),
  与产品 SVG(`assets/logo/shelf.svg` / `luoban.svg`)手工同步
- 接线锁测试照 plainsight `app/mod.rs:437` 成稿:

```rust
#[test]
fn title_logo_kind_lock() {
    assert_eq!(title_logo_kind(), LogoKind::Dropshelf); // 或 Luoban
    assert_ne!(title_logo_kind(), LogoKind::Default);
}
```

## Testing Strategy

- **danqing**: 几何锁测试(照 Plainsight 分支成例; 若成例缺则本次一并补立)
- **产品侧**: 接线锁测试(上示)入各仓既有测试模块; `cargo test` 全绿
- **回归**: G2 改动后 dropshelf 既有 84 测试全绿, 行为不变(粘贴图片落盘路径不变——
  `persist::config_dir` 与 `dirs::config_dir` 同根, 断言行级别验证)
- **肉眼项**(两仓标题栏实际渲染): 机器验不了, 当场记账各仓 `tasks/acceptance-pending.md`(C3),
  不口头记

## Boundaries

- **Always**: 提交前三件套各仓全绿; 升 rev 只用 `cargo check` 驱动重解;
  lock 复钉后确认纯 git 态(`grep 'source = "path+'` 零命中); 联动改动分仓提交、message 注明关联
- **已授权(2026-10-13 spec 批准)**: danqing commit+push; 色板下沉框架(含 Theme trait 公共面新增 token)
- **Ask first**: 升 rev 若带出框架其他变动(rev 跨度内 danqing 有新提交)是否接受
- **Never**: 不发布任何产品/不动发布 hold; 不开 local-patch; 不动 danqing-license;
  不碰三仓既有 spec/plan 文档的既有内容(仅追加入账); 不改 `focus-history.json` 类数据格式

## Success Criteria

1. `LogoKind::Dropshelf` / `LogoKind::Luoban` 存在于 danqing 并已 push origin/dev,
   几何锁测试绿
2. dropshelf/luoban lock 复钉该 rev(纯 git 态), 双接线完成, 接线锁测试绿,
   两仓既有测试全绿(dropshelf 84 / luoban 86)
3. dropshelf `Cargo.toml` 无 `dirs` 依赖, 全仓 `grep dirs::` 零命中
4. 色板下沉落地: Theme 等价 token 换用 + 无等价者新增进 danqing, dropshelf 两自绘件
   全改走 token, 硬编码色零残留(main.rs:1189 `clear_color` 半透明有意设计除外)
5. luoban `.cargo/config.toml` 磁盘删除
6. plainsight `.gitignore` 三条目齐
7. 根 CLAUDE.md 地图三行+路由三条(状态如实: dropshelf/luoban 未发布在制、
   plainsight 发布 hold 10-21)
8. 两仓标题栏 logo 肉眼验收项入账各仓 acceptance-pending.md
9. P2 发布前闸待办入账: dropshelf 新建 release-checklist.md(含 E1/E2/D5/A7 待办),
   luoban/plainsight 既有 release-checklist.md 补登缺口项(plainsight 关于页缺仓库地址等)

## 裁决记录(2026-10-13 用户全批, 开放问题清零)

1. **danqing push**: 批准本 spec 即视为授权(链路: danqing 提交+push → 产品仓升 rev)
2. **dropshelf 色板**: 下沉框架——先查 Theme 等价 token 换用, 无等价的下沉新增进 danqing
   再消费; main.rs:1189 `clear_color` rgba 0.92 (hold-to-show 半透明) 为有意设计, 不在范围
3. **dropshelf release-checklist.md**: 借本批新建, P2 发布前闸待办入账
4. **存档位**: 认可 `danqing/docs/intent/`(跨仓工作文档成例)
