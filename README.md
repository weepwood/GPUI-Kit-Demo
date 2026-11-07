# GPUI Kit Demo

**Rust 原生桌面应用脚手架 · Codex-inspired Workspace UI**

[![Build](https://github.com/weepwood/GPUI-Kit-Demo/actions/workflows/build.yml/badge.svg)](https://github.com/weepwood/GPUI-Kit-Demo/actions/workflows/build.yml)

基于 **GPUI Kit 0.9.0** 的通用原生桌面脚手架，借鉴 Codex Desktop 的布局思路：灰白侧栏、简洁主工作区、轻量导航、底部任务输入区。不是 OpenAI 官方产品，也不复制其商标或素材。

## 已有功能

- 原生 GPUI 界面，无 Electron / WebView。
- 可折叠侧栏、顶栏、主工作区、固定底部输入区。
- Overview / Components / Activity / Settings 四个示例页面。
- 内置 GPUI Kit Button、Icon、Input、Switch，具有真实交互。
- 应用级颜色 Token：src/palette.rs；深色与浅色界面可切换。
- 内存任务记录、按钮事件计数、页面导航。
- Windows x64 ZIP + Inno Setup 安装器、macOS .app ZIP 自动构建与 Release。

**范围说明**：任务输入仅增加本地内存里的样例条目，不会启动 AI 任务、访问后端、写入磁盘或保存长期设置。

## 环境

Rust stable（建议 Rust 1.92+ / edition 2024）。

Windows 需要 Windows 10+、VS2022 C++ Build Tools、Windows SDK、CMake 及 MSVC Rust toolchain。macOS 需要 macOS 15+ 和 Xcode Command Line Tools。Linux 可参考 [GPUI Kit 官方安装说明](https://gpui-kit.com/docs/installation/) 配置 Vulkan/Wayland/X11 依赖。

本地运行：

~~~bash
cargo run
~~~

执行测试、生产构建：

~~~bash
cargo fmt --all
cargo test
cargo build --release
~~~

## 项目结构

~~~text
.
├── .github/workflows/build.yml  # CI、安装包和 Release
├── installer/
│   ├── windows.iss              # Windows Inno Setup
│   └── Info.plist               # macOS 应用元数据
├── src/
│   ├── main.rs                  # UI 页面与交互状态
│   └── palette.rs               # 浅色/深色 Token
├── Cargo.toml
├── rust-toolchain.toml
└── RELEASE_NOTES.md
~~~

## 开发与扩展

1. 在 Page 枚举中加入业务页面，将页面内容放到独立模块。
2. 通过 nav_item 注册导航入口，并使用 cx.notify() 触发更新。
3. 在 Palette 里集中维护产品视觉变量。
4. 如需数据持久化，可替换内存 tasks 为 SQLite、文件或服务端 API。
5. 投入正式产品前需要增加 UI 回归测试、不同 DPI 与无障碍验收。

## 发布

向 main 分支推送或推送 v* 标签会触发 GitHub Actions。首次 main 构建全部成功后，流程尝试自动创建 v0.1.0 预发布版本；后续版本建议打新的版本标签。

Windows 安装程序 **未进行代码签名**；macOS .app **未签名/未公证**。平台安全提示是预期情况，生产发布需配置证书。

- 上游依赖：https://github.com/longbridge/gpui-kit
- 文档：https://gpui-kit.com/
- Codex 和 OpenAI 是相应所有者的商标，本项目与 OpenAI 无官方关系。

MIT License.
