# UI review · 2026-11-21

| 位置 | 实测代码发现 | 修改 |
| --- | --- | --- |
| Home | `Build something great.` 和三张并列的模板卡片 | 以真实本地任务数量为中心，使用紧凑目标入口 |
| Global bottom | 所有页面常驻类似聊天窗口的 Composer | 移到 Tasks 页，只让任务场景显示编辑框 |
| Copy | `YOUR WORKSPACE / START HERE`、`Getting started`、重复“没有 AI”免责声明 | 用明确的功能标签与真实存储状态替换 |
| Default data | 首次运行自动注入两条完成度不明的示例任务 | 首次启动为空；不改写用户旧数据 |
| Inputs | 组件预览与任务编辑共用输入状态 | 拆分为独立的 GPUI InputState |
| Preferences | Settings 折叠开关只调用 notify，不保存配置 | 改为统一的 set_collapsed 持久化方法 |
| Color | 黑白灰色阶缺少独立可复用的强调层级 | 统一明暗 Palette，加入 accent_soft / accent_ink |
| Release | Cargo.toml 是 v0.2.0，安装器与 macOS plist 是 v0.1.0 | CI 从 Cargo.toml 读版本并注入各平台包 |
| Platform | 现有 macOS 构建只覆盖 ARM64 | 增加 macos-15-intel 的 x86_64 构建 |

## 验证范围

- 已进行源代码层面的功能路径和版本检查，并为色彩角色增加 WCAG 对比度相关的自动化测试。
- Windows / macOS 的完整实机构建结果必须以 GitHub Actions 工作流为准；未通过 CI 前不得标记“已发布”。
- UI 的多 DPI、键盘可达性、触控板与 reduced-motion 等事项仍需真实桌面环境验收。
- 重点复查：旧 settings.json 的向后兼容性、保存失败提示、新增和删除任务、两个主题的一致性。
