# Phase 01 — 整合社区 PR #7 与 #8

**Status**: `completed`
**目标**: 合并 #8 与 #7，修正合并前发现的问题，并让两者的文字排版与外观选项协同工作。
**前置**: 2026-10-08 完成三个 PR 的审核；用户授权由 Claude 决定整合方案。

## 验收判据

- #8、#7 的原始提交保留在 `main` 中，GitHub 自动标记两者为 merged
- 发布 EXE 不再直接导入 `NotifyNetworkConnectivityHintChange`
- 切换外观后窗口宽度随字体更新，11 种语言在 4 种外观组合下文字不超出各自的列
- `cargo fmt -- --check`、`cargo test --locked`、`cargo build --release --locked` 通过
- 用户在任务栏实际运行发布构建并确认无问题

## Tasks

- [x] 审核 #6、#7、#8 并确认问题（#6 旧配置显示方向反转；#7 外观切换后不重新定位；#8 直接导入 Win10 2004 接口；三者两两冲突，`git merge-tree` 确认）
- [x] 合并 #8（merge `fc771e6`）
- [x] 网络恢复通知接口改为运行时查找（`src/recovery_events.rs`; commit `0265be5`; 发布 EXE 的 `iphlpapi.dll` 仅导入 `CancelMibChangeNotify2`）
- [x] 合并 #7 并解决 `src/window.rs` 的 13 处冲突（merge `0451b90`）
- [x] 统一排版：字体与固定数字列沿用 #8；配色、进度条、字号来自 #7；列宽按标签字体和重置字体实测；大字号行高 19 px（`src/window.rs`; `src/quota_text.rs`）
- [x] 外观切换后先 `position_at_taskbar()` 再重绘（`src/window.rs`）
- [x] 改写 #7 的本地化布局测试以适配双字体双列布局（`cargo test --locked`: 64 passed）
- [x] 离屏渲染 5 种语言/外观/服务组合并逐张检查（临时探针，未提交）
- [x] 经 PR #9 合并到 `main`（merge `694edb9`）

## Notes

- 用户于 2026-10-09 在任务栏运行发布构建，测试中将组件移到另一显示器，确认“没问题”。测试前后的 `settings.json` 已备份并恢复。
- Clippy：11 条风格警告（参数过多、`is_multiple_of` 等），未与 v1.9.1 基线逐条比对。
- 本地重复实现原在 `codex/usage-display-reliability` 的未提交改动中，已由 #8 覆盖；stash 与分支已删除。
