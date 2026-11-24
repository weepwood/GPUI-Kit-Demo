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
