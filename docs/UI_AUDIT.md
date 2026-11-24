# UI review · 2026-11-24（第一轮：组件画廊）

| 位置 | 实测代码发现 | 修改 |
| --- | --- | --- |
| Library | 仅 Button / Input / Switch 与三个色块，组件覆盖面与"组件库"定位不符 | 新增 `src/gallery.rs` 六节画廊：Actions、Entry、Selection、Indicators、Feedback、Color roles，全部绑定真实状态 |
| Theme | GPUI Kit 控件使用默认蓝紫 primary，与页面陶土色系脱节 | `palette::apply_theme_overrides` 在启动与每次切换外观后把 clay primary 写入主题配色，并补充对比度单测 |
| Buttons | 只演示 primary / secondary 两种 | 覆盖 primary / secondary / ghost / danger / link、四种尺寸、图标按钮、禁用态与真实 busy 流（900ms 计时 + 完成计数） |
| Home | 单卡片 + 文案入口，缺少容量信息 | 重排为任务统计卡（计数 + 容量环）加双入口行，全部数据来自本地状态 |
| Tasks | 无密度概念，删除仅文字按钮 | 行距由 Library 的密度单选真实驱动；删除改为图标按钮并带 tooltip；新增行号图块 |
| Preferences | 开关裸排、无真实路径信息 | 改为 `kit::setting_row`（图标 + 说明 + 右侧控件），显示真实 settings.json 路径 |
| Nav | 选中项只有底色差异 | 选中项加陶土色左指示条与强调图标，头部加页面图标与 LOCAL 胶囊，折叠按钮改用 PanelLeft 图标 |
| Reuse | 页面样式内联重复 | 抽出 `src/kit.rs`：card / stage / section_head / field_label / caption / pill / swatch / divider / setting_row |

## 验证范围

- `cargo fmt`、`cargo check`、`cargo clippy`（仅遗留 2 条历史告警：storage Default 可派生、destination_row 参数数，均在改动前已存在）、`cargo test` 7 项全部通过。
- `cargo run` 启动冒烟：窗口正常打开、无 panic，9 秒后手动结束。
- Windows 实机以外（macOS、多 DPI、reduced-motion、键盘可达性）的核对应由 CI 与实机完成，未通过前不得宣称发布。

---

# UI review · 2026-11-24（第二轮：折叠修正、动效、图标与原生毛玻璃）

| 位置 | 实测代码发现 | 修改 |
| --- | --- | --- |
| 侧栏折叠态 | 指示条 + 图标组合整体居中导致图标偏移，左侧出现空白 | 折叠时隐藏指示条、菜单项/折叠按钮/头像一律 `justify_center`，logo 居中 |
| 动效 | 界面完全无过渡，切页与折叠为硬切 | 侧栏折叠宽度过渡（200ms ease-in-out）、页面淡入上浮（180ms ease-out-quint）、Home 计数滚动（400ms）；动画 id 携带目标态以保证状态翻转时重播；新增 `animations` 偏好开关并持久化 |
| 图标 | 画廊缺少对内嵌图标资源的展示 | Library 04 区块新增 18 个 Lucide 图标的网格（图标 + 名称） |
| 毛玻璃 | 所有表面不透明，与"原生桌面"定位不符 | 窗口启用 `WindowBackgroundAppearance::Blurred`（Windows Acrylic / macOS Vibrancy），canvas / sidebar / surface 三层半透明；新增 `translucent` 偏好，运行时经 `Window::set_background_appearance` 即时切换 |
| 兼容 | 旧 settings.json 缺少新字段 | `#[serde(default)]` 兜底，`animations` / `translucent` 默认开启；序列化往返测试更新 |

## 验证范围

- `cargo fmt` / `check` / `clippy`（仍仅 2 条历史告警）/ `test` 7 项全部通过；`cargo run` 实机冒烟正常。
- Acrylic 依赖 Windows 10 1809+（build < 17763 自动跳过，退化为不透明）；macOS Vibrancy 以 CI 构建为准。
- 折叠动画期间整树重排，若低端机出现卡顿可将 `animations` 关闭；多 DPI 与 reduced-motion 实机核对项保持待办。

---

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
