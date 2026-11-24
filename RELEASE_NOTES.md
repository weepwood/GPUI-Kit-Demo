# GPUI Kit Demo v0.5.0 — 可测试性与开发者工具

- 新增 Developer 页（侧栏 SYSTEM 区 / `Ctrl+5`）：运行环境、渲染与点击计数、配置文件路径与体积、当前状态 JSON 预览与复制、二次确认的清理操作。
- 内置元素检查器露出水面：debug 构建默认可用，`Ctrl+Shift+I`（macOS `Cmd+Alt+I`）即可拾取元素树；Developer 页提供一键开关。
- 拆出 library crate：`src/lib.rs` 承载工作台，`src/main.rs` 只调用 `run()`，界面现在可以被测试直接挂载。
- 新增 30 个无头 UI 用例（导航 / 任务 / 偏好 / Library / Developer / 布局），全部通过真实点击、快捷键与语义属性断言；替换掉「只跑通编译」的检查。
- 引入测试基础设施：`Fixture`（固定 1220×800 窗口 + 临时配置目录）、`scroll_to`、`click_and_focus`、布局不变量断言；storage 支持路径注入，测试不再污染真实用户配置。
- 工程门禁加强：CI 从 `cargo fmt`（只格式化不校验）改为 `--check`，并新增 `clippy --all-targets -D warnings` 与 `test --features gpui-kit/test-support`；新增 `.cargo/config.toml` 别名、`docs/TESTING.md`、`docs/DEVTOOLS.md`、`CONTRIBUTING.md`。
- 版本号统一到 0.5.0（Cargo.toml、Inno Setup、Info.plist）。
- 依赖兜底：上游 `proptest 1.12.0` 需要尚未发布的 `rand_xorshift 0.5`，仓库用 `[patch.crates-io]` + `vendor/rand-xorshift` 让测试构建可解析（上游发布后可删）。

---

# GPUI Kit Demo v0.4.0 — Workbench polish

- 全局快捷键：`Ctrl+1..4` 页面导航、`Ctrl+N` 新建任务并聚焦输入、`Ctrl+B` 折叠侧栏、`Ctrl+,` 打开偏好设置；Library 页展示的绑定全部真实可用。
- 任务支持完成态：勾选完成显示删除线与弱化，计数随 `Saved tasks · n (m done)` 联动；旧版纯字符串任务文件自动迁移为 `{title, done}`，无数据丢失。
- Home 页显示最近 3 条真实任务预览与完成度；窗口启用原生毛玻璃背景（Windows Acrylic / macOS Vibrancy），可在 Preferences 中即时切换。
- 页面切换与侧栏折叠带语义动效（可在偏好中关闭）；统一界面 4px 间距节奏。
- 窗口标题显式设置，最小窗口尺寸 720×480；版本号统一为 0.4.0（Cargo.toml、Inno Setup、Info.plist 一致），关于页读取编译期版本。
- `Cargo.lock` 纳入版本控制，保证各平台构建依赖一致。

---

# GPUI Kit Demo v0.3.0 — Workbench refinement

- 移除常驻聊天式输入区，任务编辑回到 Tasks 页面；首页改为真实本地状态与功能导航。
- 减少模板化标语、虚构的启动清单与重复状态标签，重新定义导航、信息密度、排版、交互状态。
- 浅色与深色统一语义色阶，增加用于状态和导航的克制暖色强调，并校验文本与表面的对比度。
- 组件预览输入与任务编辑输入独立；修复 Preferences 页侧栏折叠状态未持久化的问题。
- 首次启动使用空任务列表，不再自动插入示例待办；原有用户保存的数据继续保留。
- Windows x64 安装器和 ZIP、macOS Apple Silicon 和 Intel app ZIP 自动构建；安装器、Info.plist 与 Cargo.toml 的版本保持一致。
- 发布流程提供 SHA256 校验文件。

这是未签名的预发布版。请在真实 Windows 与 macOS 设备上检查 DPI、键盘交互、安装与系统安全提示。
