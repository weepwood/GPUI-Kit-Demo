# GPUI Kit Demo

<img width="2678" height="1692" alt="image" src="https://github.com/user-attachments/assets/48785e30-bc25-4724-834e-08bd7e7d3a5c" />


**A native Rust workbench for exploring desktop UI patterns.**

[![Build](https://github.com/weepwood/GPUI-Kit-Demo/actions/workflows/build.yml/badge.svg)](https://github.com/weepwood/GPUI-Kit-Demo/actions/workflows/build.yml)

基于 [GPUI Kit](https://github.com/longbridge/gpui-kit) 的轻量原生桌面示例。界面采用克制的中性色、清晰的内容层级和少量暖色强调；重点展示可以直接复用的原生交互，不模拟聊天产品或 AI 助手。

## 界面组成

- **Home**：任务数、完成度与最近 3 条真实任务预览；可一键跳转任务列表。
- **Library**：六节组件画廊（Actions / Entry / Selection / Indicators / Feedback / Color roles），全部示例绑定真实状态；含 18 个内嵌 Lucide 图标的展示网格。
- **Tasks**：输入、创建、勾选完成、删除本地任务；行距密度跟随 Library 中的演示设置，任务以 `{title, done}` 结构持久化并兼容旧版纯字符串格式。
- **Preferences**：浅色 / 深色模式、导航折叠、界面动效与原生毛玻璃背景开关，重启后保留；显示真实配置文件路径。
- **Developer**：运行环境（版本、build、平台、视口与缩放）、渲染/点击/保存计数、配置文件路径与体积、当前状态 JSON 预览与复制、二次确认的清理操作，以及元素检查器与动效触发器。
- **Status bar**：持续反馈本地保存状态，不占用内容区。

窗口默认启用系统原生模糊背景（Windows 11 Acrylic / macOS Vibrancy），侧栏与画布以半透明层级透出；可以在 Preferences 中一键关闭并持久化。页面切换与侧栏折叠带有克制的过渡动画，同样可在 Preferences 中关闭。

界面由 Rust + GPUI 原生渲染，无 Electron 或 WebView。任务、主题与侧栏偏好保存到系统配置目录的 JSON 文件中，不需要在线账号。

完整颜色、排版、文案和交互约束见 [设计系统说明](docs/DESIGN_SYSTEM.md)，检查记录见 [UI 审查记录](docs/UI_AUDIT.md)。

## 本地开发

要求 Rust stable（edition 2024）；Windows 推荐 VS 2022 C++ Build Tools、Windows SDK 与 MSVC 工具链；macOS 需要 Xcode Command Line Tools。

```bash
cargo run                 # 调试版默认带元素检查器
cargo t                   # 单元测试 + 无头 UI 测试（等价于 --features gpui-kit/test-support）
cargo fmt-check           # 只校验格式，不改文件
cargo check-all           # clippy --all-targets -D warnings
cargo build --release
```

别名定义在 `.cargo/config.toml`。UI 测试必须开启 `gpui-kit/test-support`，否则元素注册表为空、用例会明确报错提示。测试写法、可测试元素约定与常见坑见 [测试指南](docs/TESTING.md)；检查器、Developer 页与 `gpui-fast` 等开关见 [开发者工具](docs/DEVTOOLS.md)。

### 快捷键

| 按键 | 功能 |
| --- | --- |
| `Ctrl+1..5` | 切换 Home / Library / Tasks / Preferences / Developer |
| `Ctrl+N` | 跳转任务页并聚焦新建输入 |
| `Ctrl+B` | 折叠 / 展开侧栏 |
| `Ctrl+,` | 打开偏好设置 |
| `Ctrl+Shift+I` | 切换元素检查器（仅 debug 构建；macOS 为 `Cmd+Alt+I`）|

## 发布与下载

[GitHub Releases](https://github.com/weepwood/GPUI-Kit-Demo/releases) 提供平台安装包；[GitHub Actions](https://github.com/weepwood/GPUI-Kit-Demo/actions/workflows/build.yml) 提供对应构建记录与 Artifact。

| 平台 | 构建产物 |
| --- | --- |
| Windows x64 | `GPUI-Kit-Demo-Setup-Windows-x64.exe`、`GPUI-Kit-Demo-Windows-x64.zip` |
| macOS Apple Silicon | `GPUI-Kit-Demo-macOS-arm64.zip`（包含 .app） |
| macOS Intel | `GPUI-Kit-Demo-macOS-x86_64.zip`（包含 .app） |

Pull Request 触发两平台编译与测试；推送主分支后，工作流从 `Cargo.toml` 读取版本并尝试创建对应的 `vX.Y.Z` 预发布版。如果同版本 Release 已经存在则不会覆盖。也支持推送匹配版本的 `v*` 标签。ZIP 和安装器中的版本号由 CI 同步，Release 包含 SHA256SUMS.txt。

**重要**：构建自动化不等于产物已发布。只有 [Releases](https://github.com/weepwood/GPUI-Kit-Demo/releases) 页面出现新版本和完整附件才代表发布成功。Windows 安装程序未签名；macOS .app 未签名或公证，正式对外分发需要配备代码签名和公证流程。工作流创建 Release 需允许 `contents: write`，如仓库策略拒绝自带令牌，可配置 `RELEASE_TOKEN` Secret。

## 工程结构

```text
src/main.rs                 可执行入口（仅调用库的 run()）
src/lib.rs                  GPUI 原生工作台与交互（可被测试直接挂载）
src/gallery.rs              Library 组件画廊（六节展示，绑定真实状态）
src/kit.rs                  可复用设计构件：卡片、舞台、标题、胶囊、设置行等
src/palette.rs              深浅色语义 Token、主题色同步与对比度测试
src/storage.rs              本地状态持久化与数据校验
tests/common/mod.rs         无头 UI 测试 fixture（固定窗口、临时配置目录）
tests/{navigation,tasks,preferences,library,developer,layout}.rs
                            46 个无头 UI 用例：真实点击、快捷键、语义与布局断言
docs/DESIGN_SYSTEM.md       可复用设计约束和 AI 生成检查清单
docs/UI_AUDIT.md            设计缺陷与修复记录
docs/TESTING.md             无头 UI 测试指南与元素 id 约定
docs/DEVTOOLS.md            元素检查器、Developer 页与调试开关
vendor/rand-xorshift        临时补丁 crate（补齐 proptest 缺失的依赖，见 DEVTOOLS.md）
.github/workflows/build.yml Windows/macOS 构建、校验、发布
installer/                  Inno Setup、macOS plist
```

MIT License.
