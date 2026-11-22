# GPUI Kit Demo

<img width="2678" height="1692" alt="image" src="https://github.com/user-attachments/assets/48785e30-bc25-4724-834e-08bd7e7d3a5c" />


**A native Rust workbench for exploring desktop UI patterns.**

[![Build](https://github.com/weepwood/GPUI-Kit-Demo/actions/workflows/build.yml/badge.svg)](https://github.com/weepwood/GPUI-Kit-Demo/actions/workflows/build.yml)

基于 [GPUI Kit](https://github.com/longbridge/gpui-kit) 的轻量原生桌面示例。界面采用克制的中性色、清晰的内容层级和少量暖色强调；重点展示可以直接复用的原生交互，不模拟聊天产品或 AI 助手。

## 界面组成

- **Home**：本地任务数与高频入口，不使用虚构的增长指标。
- **UI Components**：按 Controls / Patterns / Foundations 分类展示原生 Button、Input、Switch、Checkbox、导航、状态反馈、排版与语义色板；组件示例均可交互，预览状态与任务编辑互不干扰。
- **Tasks**：输入、创建、删除本地任务；此处是任务编辑的唯一入口。
- **Preferences**：浅色 / 深色模式与导航折叠状态，重启后保留。
- **Status bar**：持续反馈本地保存状态，不占用内容区。

界面由 Rust + GPUI 原生渲染，无 Electron 或 WebView。任务、主题与侧栏偏好保存到系统配置目录的 JSON 文件中，不需要在线账号。

完整颜色、排版、文案和交互约束见 [设计系统说明](docs/DESIGN_SYSTEM.md)，检查记录见 [UI 审查记录](docs/UI_AUDIT.md)。

## 本地开发

要求 Rust stable（edition 2024）；Windows 推荐 VS 2022 C++ Build Tools、Windows SDK 与 MSVC 工具链；macOS 需要 Xcode Command Line Tools。

```bash
cargo run
cargo fmt --all
cargo test
cargo build --release
```

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
src/main.rs                 GPUI 原生工作台、导航和状态管理
src/component_gallery.rs    可分类切换的交互式原生 UI 组件画廊
src/palette.rs              深浅色语义 Token 和对比度测试
src/storage.rs              本地状态持久化与数据校验
docs/DESIGN_SYSTEM.md       可复用设计约束和 AI 生成检查清单
docs/UI_AUDIT.md            设计缺陷与修复记录
.github/workflows/build.yml Windows/macOS 构建、校验、发布
installer/                  Inno Setup、macOS plist
```

MIT License.
