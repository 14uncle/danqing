# Spec: 便携包本地诊断日志

> 适用于阶段 2 番茄钟 POC 的 Windows 便携包。需求与本 spec 已于 2026-07-24 获用户确认。

## Objective

为 `pomodoro` 便携包增加无需额外工具即可回传的本地诊断日志，用于定位“开发机可运行、同事电脑在打印集成显卡后退出”的机器相关问题。

**用户故事：**

- 作为使用者，我双击解压后的 `pomodoro.exe` 后，程序会在 exe 旁的 `logs/` 留下本次启动日志；启动失败时可以直接把该文件夹发给开发者。
- 作为开发者，我能从日志判断程序运行到了窗口、GPU 适配器选择、GPU 设备创建或后续哪个阶段，并看到完整错误链或 panic 信息。
- 作为维护者，日志功能不能破坏现有 stderr 输出、启动 benchmark 或正常启动；日志目录不可写时应用仍应继续运行。

**本轮明确排除：**修复真实 GPU 退出根因、改变 wgpu 后端策略、改变事件循环错误语义、修复资产相对路径、记录进入 Rust `main` 前的 Windows loader/DLL 错误。

## Tech Stack

- **语言：**Rust 2024 edition，最低 Rust 1.85。
- **日志 facade：**现有 `log` 0.4。
- **日志实现：**现有 `env_logger` 0.11；保留其 `RUST_LOG` 过滤，通过 `env_logger::Target::Pipe` 接入自定义 stderr/文件双写器。
- **时间：**现有 `chrono` 0.4，用于本地时间戳。
- **错误：**标准库 `std::error::Error::source()` 与现有 `anyhow`。
- **约束：**不新增第三方依赖。

## Commands

```bash
# 开发运行
cargo run --example pomodoro

# 纯逻辑与集成测试
cargo test --lib --tests

# example 内部日志逻辑测试
cargo test --example pomodoro

# 格式与静态检查
cargo fmt --check
cargo clippy -- -D warnings

# 发布构建与启动 benchmark
cargo build --release --example pomodoro
powershell -NoProfile -File tools/benchmark.ps1 -Example pomodoro -Runs 1

# Windows 便携包
powershell -NoProfile -File tools/package_portable.ps1 -BinaryName pomodoro
```

提交门槛为 `cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test --lib --tests` 和 `cargo test --example pomodoro` 全绿；本功能还需完成 release build 与 benchmark 兼容验证。

## Project Structure

```text
examples/
  common/
    log.rs              # 扩展：exe 相对日志目录、stderr/文件双写、保留策略、panic hook、纯逻辑测试
  pomodoro/
    main.rs             # 扩展：捕获并记录顶层 anyhow 错误链，返回失败退出码
src/
  render/mod.rs         # 扩展：wgpu backend、适配器和设备创建阶段诊断
  window.rs             # 扩展：窗口/渲染初始化错误 source 链记录
tools/
  benchmark.ps1         # 不修改；继续依赖 stderr 的 perf 日志
  package_portable.ps1  # 不修改；用于打包和端到端验证
docs/specs/
  portable-diagnostics-logging.md
```

日志运行时布局：

```text
<解压目录>/
  pomodoro.exe
  assets/
  logs/
    pomodoro-20260724-153045.123-p12345.log
```

## Functional Requirements

### 日志路径与文件

- 必须使用 `std::env::current_exe()` 的父目录，不能使用相对路径或 `current_dir()`。
- 应自动创建 `<exe-dir>/logs/`。
- 每次启动创建独立文件，不覆盖既有日志。
- 文件名必须包含 exe stem、本地毫秒时间戳和进程 ID；候选已存在时追加递增序号，并通过 `OpenOptions::create_new(true)` 原子创建。
- 只管理当前 exe stem 对应的日志；不得删除 `logs/` 中其他应用或无关文件。
- 每个应用最多保留 10 份：当前文件加最近 9 份历史文件。

### 输出与降级

- 每条普通日志同时写入 stderr 和文件，沿用现有格式：

```text
HH:MM:SS.mmm LEVEL [target] message
```

- 继续完整支持 `env_logger` 的 `RUST_LOG` 过滤语义。
- 每条记录后刷新输出，尽量保留异常退出前的最后信息。
- 创建目录、创建文件、清理旧文件或文件写入失败，不得导致 panic 或阻止应用启动。
- 文件不可用时降级为 stderr；日志初始化不得使用会因重复安装而 panic 的 `.init()`。
- stderr 行为必须保持，使 `tools/benchmark.ps1` 仍能捕获 `perf startup_to_visible`。

### Panic 与错误链

- 日志文件可用时安装链式 panic hook。
- hook 必须保留并调用原 hook，同时向文件写入 panic payload、源码位置和 `Backtrace::force_capture()`。
- panic hook 自身不得因锁中毒或写入失败触发二次 panic；无法安全取得文件锁时允许放弃文件写入并依赖原 hook 的 stderr 输出。
- `pomodoro` 顶层 `anyhow::Result` 必须被显式匹配；失败时通过 `log::error!("{err:#}")` 写完整错误链，并返回失败退出码。
- 窗口创建和渲染上下文初始化错误必须逐层记录 `Error::source()`；仅增加诊断，不改变现有退出控制流。

### GPU 启动诊断

日志至少能够区分：

1. wgpu Instance 开始创建（backend 与 flags）。
2. 适配器选择成功（名称、设备类型、backend、vendor/device ID、驱动与驱动信息）。
3. 开始请求 GPU device。
4. GPU device 创建成功，或错误链说明创建失败。
5. surface 配置成功及后续既有启动日志。

不得借此改变 `PowerPreference::LowPower`、`DEFAULT_BACKENDS`、校验层开关或任何 GPU 初始化参数。

## Code Style

- 继续使用现有中文用户/维护者可读日志，内部实现使用英文命名。
- 复用 `examples/common/log.rs::init_log`，不引入新的公共框架 API。
- 文件系统和时间等副作用放在薄包装中；路径、命名和保留选择拆成可注入数据的纯函数。
- 错误处理采用降级而非 `unwrap`/`expect`；日志系统不能因自身失败终止产品。
- 不创建新的 `.rs` 文件；如实施中确需新增，必须带 `//! @author 十四叔` 与 `//! @date yyyy/MM/dd`。

示例风格：

```rust
let log_file = match create_log_file(&executable) {
    Ok(file) => Some(file),
    Err(err) => {
        eprintln!("无法创建本地日志，将仅输出到 stderr：{err}");
        None
    }
};
```

## Testing Strategy

### Small：纯逻辑单元测试

测试写在 `examples/common/log.rs` 的 `#[cfg(test)]` 中，由 `cargo test --example pomodoro` 执行：

- synthetic exe 路径解析为 exe 同级 `logs/`，不读取测试进程 CWD。
- 固定时间、PID 和序号生成确定性文件名。
- 碰撞候选使用递增后缀且不会覆盖原文件。
- 保留选择始终保护当前文件，只删除同一 exe 前缀的较旧文件。
- 其他应用日志和非 `.log` 文件不进入删除集合。

全局 logger 和 panic hook 不在普通单元测试中反复安装，避免进程级状态污染。

### Medium：构建与兼容验证

- `cargo test --lib --tests` 验证框架回归。
- `cargo test --example pomodoro` 验证 example 内纯逻辑。
- `cargo clippy -- -D warnings` 必须零警告。
- `cargo build --release --example pomodoro` 验证发布构建。
- benchmark 必须继续从 stderr 观察到 `perf startup_to_visible`。

### Large：Windows 端到端验证

- 从不同工作目录启动 release exe，日志仍必须写到 exe 旁。
- 连续启动超过 10 次后只保留当前应用最新 10 份，不影响无关文件。
- 在不可写目录中启动，确认安全降级且不因日志失败退出。
- 以临时、受控 panic 验证 payload、位置和 backtrace，验证后立即撤销测试代码。
- 打包并解压 `pomodoro` ZIP；同事双击复现后回传 `logs/` 最新文件。

## Boundaries

- **Always：**
  - 先写会失败的纯逻辑测试，再实现路径、命名和保留逻辑。
  - 保留 stderr 与 `RUST_LOG` 兼容性。
  - 所有日志文件路径以 `current_exe()` 为根。
  - 日志初始化及写入失败必须安全降级。
  - 提交前执行本 spec 的全部自动化门槛。

- **Ask first：**
  - 新增第三方依赖或将 example 日志设施迁入框架公共 API。
  - 修改 `tools/benchmark.ps1` 或便携包目录结构。
  - 改变 GPU backend、adapter/device 参数或事件循环错误传播语义。
  - 扩展到日志上传、用户数据采集或日志内容脱敏策略。

- **Never：**
  - 因无法创建日志而拒绝启动应用。
  - 使用当前工作目录决定日志位置。
  - 覆盖既有日志或删除其他应用/无关文件。
  - 吞掉原 panic hook。
  - 在本轮根据猜测修改 GPU 行为或声称已经修复同事机器上的退出问题。

## Success Criteria

1. 解压后的 `pomodoro.exe` 每次启动都尝试在自身同级 `logs/` 创建独立日志，与进程工作目录无关。
2. 正常可写目录中，普通日志同时出现在 stderr 和文件；日志包含启动、GPU adapter/device、surface 及既有 perf 信息。
3. `pomodoro` 顶层错误以完整 anyhow 链记录；窗口/渲染初始化错误逐层记录 source；panic 文件记录包含 payload、位置和 backtrace。
4. 当前应用最多保留 10 份日志，不删除其他应用和无关文件。
5. 日志目录不可写或文件写入失败时应用不 panic，退化为 stderr。
6. `tools/benchmark.ps1 -Example pomodoro -Runs 1` 仍能从 stderr 观察启动 perf 行。
7. `cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test --lib --tests`、`cargo test --example pomodoro` 和 release build 全部通过。
8. 本轮只产出可回传诊断证据；同事机器的退出根因留待拿到日志后的下一任务。

## Open Questions

无。日志位置、每次独立文件、保留最近 10 份、失败降级、诊断范围及非目标均已确认。

## Related Documents

- `docs/specs/phase2-pomodoro-poc.md` — 番茄钟 POC 总规格。
- `CLAUDE.md` — 项目结构、命令和提交门槛。
- `tools/package_portable.ps1` — Windows 便携包生成流程。
