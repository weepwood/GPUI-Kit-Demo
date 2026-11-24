# Workbench Design System · v0.3

## 产品定位

这是原生组件与工作台交互的参考应用，不是 SaaS 仪表盘，也不是聊天式 AI 客户端。每个可点击元素都应对应真实状态和用途。

## 视觉原则

1. **信息先于装饰**：首页首先显示真实本地任务数与实际可用的入口；禁止占位统计、假进度、伪造完成状态。
2. **安静的界面**：非交互表面以低对比的中性灰绿区分层级，暖陶土色只用于区域编号与关联强调，不做全屏渐变或霓虹发光。
3. **两层主要表面**：Canvas 是阅读背景，Surface 作为承载实际内容的容器。侧栏低于主内容的视觉权重。
4. **克制圆角**：导航为中等圆角；卡片与控制块使用轻量圆角；不叠加厚阴影、泛光或多层毛玻璃。唯一例外是系统原生模糊背景：窗口本身启用 Acrylic / Vibrancy，界面只有 canvas、sidebar、surface 三个半透明层级，关闭即完全恢复不透明。
5. **有意义的密度**：避免为了凑版式重复 Quick Start 卡片，导航列表优先于并列营销卡。
6. **语义化动效**：只在状态切换、导航与数据变更时产生反馈——侧栏折叠的宽度过渡（200ms）、页面切换的淡入上浮（180ms）与任务计数滚动（400ms）。全部动效受 Preferences 的 `animations` 偏好控制，关闭后立即静止；GPUI 0.2 未暴露系统 reduced-motion，用户手动开关是当前最佳替代。
7. **真实文案**：使用 Tasks、Local、Saved 等具体名词。禁止 “Build something great”“Power your workflow” 等无法传达行为的信息。
8. **平台一致性**：Windows、macOS 保持同一信息架构；使用原生控件与 GPUI Kit 组件，不将 Web CSS 或 JS 直接抄进 Rust。半透明层级在两平台均走原生模糊通道，在不支持的系统（老版本 Windows）上自动退化为不透明，不需要降级样式。

## 颜色角色

以下角色由 `src/palette.rs` 管理，禁止在页面里直接写颜色常量：

| 角色 | Light | Dark | 语义 |
| --- | --- | --- | --- |
| Canvas | `#F6F7F3` | `#171B19` | 内容阅读区 |
| Surface | `#FEFEFC` | `#202521` | 交互容器 |
| Sidebar | `#EFF0EB` | `#1B201D` | 低权重导航 |
| Text | `#242B25` | `#F2F1EC` | 标题与正文 |
| Muted | `#57615A` | `#B8BDB4` | 辅助文案 |
| Border | `#DDE1D9` | `#394039` | 结构分隔 |
| Accent soft | `#F6E9DF` | `#45362C` | 限定范围强调 |
| Accent ink | `#885035` | `#F0B992` | 强调文字、图标 |
| Theme primary | `#A85432` | `#E2A97F` | GPUI Kit 原生控件主色 |
| Alpha | 0.96 / 0.84 / 0.90 | 0.94 / 0.80 / 0.86 | canvas / sidebar / surface 的毛玻璃不透明度 |

`src/palette.rs` 单测确保核心文字和强调文字与底色满足至少 4.5:1 对比度，主题 primary 与其前景色同样受测。`palette::apply_theme_overrides` 把陶土色 primary 写入 GPUI Kit 主题的当前配色（`Theme::change` 会恢复默认配色，因此每次切换外观后必须重新应用；当前在启动与 `set_dark` 时调用）。图标、disabled、hover 和 focus 仍需实机检查。

Library 页面是组件画廊，按 01–06 编号展示：Actions（按钮变体 / 尺寸 / 图标 / busy）、Entry（受控输入与实时字数）、Selection（Switch / Checkbox / Radio，密度选项真实作用于 Tasks 行距）、Indicators（Badge / Tag / Kbd）、Feedback（Slider 驱动 Progress、容量环、Spinner / Skeleton）、Color roles（当前模式的语义色板与十六进制值）。画廊内均为真实工作台状态：任务计数、容量、主题与侧栏偏好直接来自 Workspace。

## 组件与交互规则

- 侧栏：默认展开；折叠后文字隐藏而图标保留；两种路径中的折叠状态必须持久化。折叠模式下指示条隐藏、图标精确居中，避免空白偏移。
- 输入：组件样例与任务编辑不能共用文字状态；文本最大长度与任务数量由 `storage.rs` 限制。
- 任务：以 `{title, done}` 持久化，旧版本纯字符串文件自动迁移；完成后仅弱化与删除线，不做成功庆祝动画；预览（Home 顶部三条）与任务列表（Tasks 页）读同一份状态。
- 状态栏：只报告真实的保存状态；错误必须可见，不能用装饰绿点替代状态反馈。
- 空状态：明确下一步操作；空任务列表不是异常，也不能预置虚构用户数据。
- 主题：GPUI Kit 主题与应用自定义 Token 应同时更新。
- 动效（第 6 条的落地实现）：`with_animation` 的元素状态按元素 id 存续，因此折叠动画的 id 携带目标态（`sidebar-width-{collapsed}`）、计数动画的 id 携带数值，使状态翻转/变更时动画必然重播；页面按 page index 命名，页面隐藏时状态被丢弃，重新进入时重播入场。

## 可测试性契约

- 每个可点击元素：稳定 `.id(...)`；自绘 `div` 还要 `.test_support()` + `.aria_label(...)`。
- id 前缀：`nav-` 导航、`task-` 任务、`*-switch` 偏好、`library-`/`gallery-` 画廊、`dev-` Developer、`page-title`/`page-eyebrow`/`status-bar` 全局结构。
- 语义优先：用 `role` / `aria_selected` / `aria_label` 表达状态，测试只读原生语义，不读像素。
- 列表元素 id 用存储索引而非渲染序号，避免视觉顺序变化导致断言漂移。
- 动效必须可关闭；关闭后一帧即为终态，几何断言依赖这个前提。

## 面向 UI 生成 Agent 的约束

> 为已有 Rust GPUI Kit 应用优化一个工作台页面。先读取组件和 `Palette`，遵循现有 Theme / Context / Render API。不得凭空新增 API。不要用通用 SaaS dashboard 的巨型标题、三列统计卡、荧光渐变、装饰标签和聊天框。以具体用户任务为导向，优先修正内容层级、文字、行距与状态反馈。每个按钮需有实际操作，每个计数需来自状态。交付时列出影响到的 UI 状态、深浅模式、响应式窗口、跨平台构建风险和测试结果。

### 生成后审核（每次都要做）

1. 逐项标出所有文案，替换不可验证的宣传句和假数据。
2. 检查色彩角色、文本对比度、两种主题以及窗口缩小时的溢出。
3. 手动体验首次启动空状态、创建/删除、切换页面、主题与持久化。
4. 检查真实控件 API、Rust 编译、Windows 与 macOS 构建。
5. 发布时核对包内版本号、SHA256、签名与完整性；不要以 CI 配置存在来宣称发布成功。

## 参考开源项目与方法

- [jakubkrehel/skills](https://github.com/jakubkrehel/skills)：以可访问性、布局、文字与细节检查替代“一句话美化”的工作流。
- [emilkowalski/skills](https://github.com/emilkowalski/skills)：控制动画节奏，只在必要位置添加反馈；尊重 reduced-motion。
- [longbridge/gpui-kit](https://github.com/longbridge/gpui-kit)：复用已经存在的 GPUI Kit 原生主题、组件和平台能力。
- [zed-industries/zed](https://github.com/zed-industries/zed)：参考成熟原生桌面产品的高信息密度、工作台架构与 GPUI 用法。

这些资源作为方法论参考，不直接拷贝第三方受版权保护的视觉素材、品牌元素或实现代码。优先采用项目已有的 GPUI Kit API。
