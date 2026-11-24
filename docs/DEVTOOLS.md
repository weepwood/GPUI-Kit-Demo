# 开发者工具

三类工具：**应用内 Developer 页**、**元素检查器**、**工程命令与 feature 开关**。

## 1. Developer 页（Ctrl+5 / 侧栏 SYSTEM 区）

发布版和 debug 版都在，因为它是这套工作台的一部分，也顺便演示了「数据展示」类组件。

| 卡片 | 内容 |
| --- | --- |
| Runtime | 版本（编译期 `CARGO_PKG_VERSION`）、build 类型、平台/架构、视口尺寸与缩放、帧计数、运行时长、点击与保存计数 |
| Storage | 配置文件绝对路径、文件体积、`Reveal in file manager`、`Clear tasks` / `Reset preferences` / `Clear everything` |
| State | 当前 `Settings` 的 JSON 预览（900 字符）+ `Copy as JSON`（写系统剪贴板） |
| Debugging tools | 检查器说明与开关按钮、演示触发器（busy 周期、切换主题、折叠侧栏）|

三个清理按钮都是**二次点击确认**：第一次点变成红色的 `Click again: ...`，再点才执行；离开页面或点另一个按钮会解除待确认状态。这样误触不会丢数据，也顺带演示了破坏性操作的确认模式。

对应测试：`tests/developer.rs`（7 个用例，覆盖读数、JSON 跟随、二次确认、复制反馈、触发器与帧计数）。

## 2. 元素检查器（Ctrl+Shift+I / macOS Cmd+Alt+I）

GPUI Kit 的检查器在 **debug 构建里自动挂载**（`gpui_kit::init` → `gpui_component::init` → `inspector::init`，源码里由 `#[cfg(any(feature = "inspector", debug_assertions))]` 控制），快捷键绑定在 `ctrl-shift-i`。按下去以后进入拾取模式，鼠标悬停即可读取元素树的边界与状态，用来回答「这个东西为什么没渲染 / 为什么跑到那里去了」。

- 想从 UI 打开：Developer 页 → `Toggle the inspector`（调用 `Window::toggle_inspector`，同样只在 debug 生效）。
- release 版没有这个能力，`toggle_inspector` 在非 debug 构建里不存在，所以调用点用 `#[cfg(debug_assertions)]` 门控。

## 3. 工程命令

`.cargo/config.toml` 里定义：

| 别名 | 展开 | 用途 |
| --- | --- | --- |
| `cargo t` | `test --features gpui-kit/test-support` | 单元 + 无头 UI 全量 |
| `cargo t-ui` | 同上 + `--tests` | 只跑 `tests/` |
| `cargo check-all` | `clippy --all-targets --features gpui-kit/test-support -- -D warnings` | 门禁级 lint |
| `cargo fmt-check` | `fmt --all -- --check` | 只校验，不改文件 |

CI（`.github/workflows/build.yml`）在两个平台都跑 `fmt --check` → `clippy -D warnings` → `cargo test --features gpui-kit/test-support`，然后才构建 Release 产物，所以测试与 lint 是发布门禁。

## 4. 可选的编译期开关

| feature | 作用 | 何时用 |
| --- | --- | --- |
| `gpui-fast`（本项目透传 `gpui-kit/gpui-fast`）| 换用 gpui-fast 渲染后端 | 想对比另一套引擎时；未纳入 CI |
| `gpui-kit/inspector` | 让 release 版也编译检查器 | 需要给外部演示版做元素检查时 |
| `gpui-kit/profiler` | 接入 `gpui/profiler`（tracing / Chrome trace） | 排查掉帧与重绘热点；默认不开 |

```bash
cargo run --features gpui-kit/inspector
cargo run --features gpui-kit/profiler
```

## 5. 关于 `vendor/rand-xorshift`

`gpui` 的 `test-support` feature 依赖 `proptest`，而 `proptest 1.12.0` 声明的 `rand_xorshift ^0.5` 还没有发布到 crates.io，导致整张依赖图无法从零解析（已被上游确认是发布事故）。仓库里用 `[patch.crates-io]` 指向 `vendor/rand-xorshift`：一个按 `rand_core 0.10` 实现 XorShift128 的小 crate，只为让解析通过，并且只在测试构建里被编译。

- 保持原状的唯一硬性要求：**不要**在没有这个 patch 的情况下执行 `cargo update`，否则会直接解析失败。
- 上游发布 `rand_xorshift 0.5` 之后，删掉 `vendor/rand-xorshift` 与 `Cargo.toml` 里的 `[patch.crates-io]` 即可，其余代码不用改。
- 普通构建（不带 `test-support`）不会用到它，`warning: patch ... was not used in the crate graph` 属于预期提示。
