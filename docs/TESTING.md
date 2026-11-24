# 无头 UI 测试指南

这个项目的界面测试是真的点、真的敲、真的量尺寸：它们在 `TestAppContext` 里挂载真实工作台，用 GPUI 的指针与键盘分派触发真实 `on_click` / `on_action`，再读取元素的**原生可访问性属性**与**布局结果**做断言。没有截图基线，也不需要 GPU。

## 运行

```bash
cargo t                 # 全部：单元测试 + UI 测试
cargo t-ui              # 只跑 tests/ 下的 UI 测试
cargo test --features gpui-kit/test-support   # 等价的长写法
```

`gpui-kit/test-support` 必须开启：它把库里的 `.test_support()` 从「原样返回」切换成真正的元素注册表。忘记开启时用例会给出明确提示：

```text
element `nav-overview` is not registered: run the suite with `cargo test --features gpui-kit/test-support`
```

在正常构建（含 release）里 `.test_support()` 是零成本透传，所以它可以长期留在产品代码里。

## 一个用例长什么样

```rust
mod common;

use common::Fixture;
use gpui_kit::TestAppContext;
use gpui_kit::test::TestWindowExt;
use gpui_kit_demo::storage::Settings;

#[gpui_kit::test]
fn the_editor_adds_a_task(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    fixture.update(cx, |window, cx| window.click("nav-activity", cx));
    fixture.update(cx, |window, cx| {
        window.click("task-input", cx);
        window.input("Ship the design system", cx);
        window.click("add-task", cx);
    });
    assert_eq!(fixture.state(cx, |workspace| workspace.tasks.len()), 1);
}
```

`tests/common/mod.rs` 提供的 fixture：

| 方法 | 用途 |
| --- | --- |
| `Fixture::open(cx, settings)` | 1220×800 固定窗口挂载工作台，配置写到临时目录 |
| `Fixture::open_calm(cx)` | 同上但关闭动效，用于**几何断言**（第一帧就是终态） |
| `fixture.update(cx, \|window, cx\| ...)` | 拿到 `Window`，执行点击、输入、按键、滚动 |
| `fixture.render(cx)` | 只重绘一帧，刷新记录到的元素事实 |
| `fixture.state(cx, \|w\| ...)` | 读真实 `Workspace` 字段 |
| `fixture.settings(cx)` / `fixture.stored()` | 读内存设置 / 读磁盘设置 |
| `fixture.rect(cx, id)` | 元素 `(x, y, w, h)`，单位像素 |
| `fixture.label / value / checked / selected / focused` | 原生语义属性 |
| `fixture.has / try_label` | 元素是否存在（不 panic） |
| `fixture.scroll_to(cx, id)` | 滚动到元素**完整**进入视口 |
| `fixture.click_and_focus(cx, id)` | 反复点击直到拿到键盘焦点 |
| `fixture.focus_shell(cx)` | 聚焦工作台根节点（快捷键走焦点路径） |

## 可测试元素的三条约定

1. **组件自带注册**：`Button` / `Checkbox` / `Switch` / `Input` / `Slider` / `Progress` 等在内部已调用 `.test_support()`，只要给稳定 `.id(...)` 就能被 `find` 到。
2. **自绘元素要显式注册**：`div()` / `h_flex()` 自己搭的可点击行必须 `.id("x").test_support().aria_label("…")`。顺序很重要——样式与 `role()` 调用放在 `test_support()` 之前，`aria_*` 与 `.child()` 可以放在之后。
3. **id 命名**：`nav-*` 导航、`task-*` 任务、`pref-*`/`*-switch` 偏好、`library-*`/`gallery-*` 画廊、`dev-*` Developer 页、`page-title`/`page-eyebrow`/`status-bar` 全局。列表类元素用存储索引（`task-row-{index}`），不随渲染顺序变化。

## 五个已知的坑

| 现象 | 原因 | 做法 |
| --- | --- | --- |
| 快捷键用例不生效 | 动作沿焦点路径分发，测试里没有焦点 | 先 `fixture.focus_shell(cx)` 再 `window.press("ctrl-2")` |
| `is not visible` 导致点击失败 | 元素被滚动区域裁切，或只露出一半 | `fixture.scroll_to(cx, id)`；`scroll_to` 要求**完整**可见 |
| 点击了但没聚焦 | 滚动尚未稳定，点到上一帧的位置 | `fixture.click_and_focus(cx, id)` |
| 几何断言数值抖动 | 入场/侧栏动画的第一帧是起点 | 用 `open_calm`（关闭动效）后一帧即终态 |
| `RadioGroup` 子项找不到 | 组件用序号重写子元素 id | 点击 `window.within("density-radio").click(1usize, cx)` |

另外：`window.scroll` 的 `ScrollDelta::Lines` **负值**才是向下滚动；`ElementId` 只接受 `&'static str`，所以 fixture 里的 id 参数都是 `&'static str`。

## 覆盖范围

| 文件 | 保护什么 |
| --- | --- |
| `navigation.rs` | 五个目的地、选中态、`Ctrl+1..5`、侧栏折叠与图标居中、Home 卡片跳转 |
| `tasks.rs` | 新增/空输入/勾选/删除、`Ctrl+N` 聚焦、持久化落盘、行 id 与渲染顺序、紧凑密度 |
| `preferences.rs` | 主题双向同步、动效与毛玻璃开关、折叠开关、配置路径与版本行 |
| `library.rs` | 受控输入的字符计数、计数复选框、滑块驱动进度、画廊开关、按钮点击 |
| `developer.rs` | 构建信息、状态 JSON 预览、二次确认清理、复制反馈、演示触发器、帧计数 |
| `layout.rs` | 侧栏与内容区拼接成窗口宽度、状态栏贴底、最小点击尺寸、内容不越界 |

新增交互时，先问三个问题：它有稳定 id 吗？它能被真实点击/按键触发吗？失败时断言消息说得清原因吗？三条都满足，就补一个用例。
