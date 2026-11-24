# 参与开发

## 环境

- Rust stable（edition 2024），版本由 `rust-toolchain.toml` 固定。
- Windows：VS 2022 C++ Build Tools + Windows SDK（MSVC 工具链）；macOS：Xcode Command Line Tools。

## 常用命令

```bash
cargo run          # 调试运行（自带元素检查器，Ctrl+Shift+I）
cargo t            # 单元测试 + 无头 UI 测试
cargo check-all    # clippy --all-targets -D warnings
cargo fmt-check    # 只校验格式
cargo fmt --all    # 提交前实际格式化
cargo build --release
```

提交前请确保 `cargo fmt-check`、`cargo check-all`、`cargo t` 三条全绿——它们也是 CI 的发布门禁。

## 改动界面时

1. **新增可点击元素**：给稳定 `.id(...)`；自绘 `div` 还要 `.test_support()` + `.aria_label(...)`（组件内部已注册，无需重复）。约定见 [TESTING.md](TESTING.md#可测试元素的三条约定)。
2. **新增交互**：在 `tests/` 对应的文件里补一个用例，断言「状态变了」以及「用户能在界面上看到这个变化」。
3. **新增快捷键**：在 `src/lib.rs` 的 `static_action_bindings()` 登记，在 `Workspace::render()` 根节点加 `.on_action(...)`（动作处理器必须在 paint 阶段注册），同步更新 Library 页的快捷键列表、README 的表格与 `docs/DEVTOOLS.md`。
4. **动效**：只在 `self.animations` 为真时挂 `with_animation`；需要几何断言的场景会被 `Fixture::open_calm` 关掉。
5. **颜色/间距**：只用 `palette::Palette` 的语义 token 与 4px 节奏，不写死色值；新增 token 要过 `palette.rs` 里的对比度测试。

## 提交与发布

- 一个提交聚焦一件事；界面改动请在 `docs/UI_AUDIT.md` 追加一段记录（问题 → 处理 → 验证）。
- 发布流程：`Cargo.toml` 升版本 → 同步 `installer/windows.iss` 与 `installer/Info.plist` → 在 `RELEASE_NOTES.md` 顶部加一节 → 推送主分支。CI 会读 `Cargo.toml` 的版本创建 `vX.Y.Z` 预发布版并附带 SHA256。
- 版本号三处必须一致，否则 Release 段落里的 tag 校验会失败。

## 依赖变更注意

`gpui` 的 `test-support` 依赖链上有一个上游未发布的依赖（`rand_xorshift 0.5`），仓库用 `[patch.crates-io]` 指向 `vendor/rand-xorshift` 兜底。在它被移除之前：

- 不要执行 `cargo update`（会解析失败），按需用 `cargo update -p <crate>`。
- 不要删除 `vendor/rand-xorshift`，除非上游已经发布该版本。
- 细节见 [DEVTOOLS.md](DEVTOOLS.md#5-关于-vendorrand-xorshift)。
