# 版本与发布

[English](release.md) | 中文

<a id="version-policy"></a>

## 版本规则

最新版本为 0.3.1。Material 各库与 Gallery 统一使用根清单 `[workspace.package]` 的版本。[公开门面及集成契约](integration.zh.md#api-与迁移)界定兼容范围，包括文档列出的 feature、公开签名中的上游类型及受支持的平台行为。

- 在 `1.0.0` 之前，向后兼容的修复递增 patch，向后兼容的新功能或破坏性变更递增 minor（同时将 patch 归零）。同一版本包含多种变更时采用最高适用级别。
- 从 `1.0.0` 起遵循 [SemVer](https://semver.org/lang/zh-CN/)：公开 API 不兼容变更递增 major，兼容新功能和公开弃用递增 minor，兼容修复递增 patch。
- 所有破坏性变更都需记录迁移步骤，包括 `1.0.0` 之前。

<a id="changelog"></a>

## 变更日志

Material 各 crate 和 Gallery 中面向用户的变更合并时，写入 [CHANGELOG.md](../CHANGELOG.md) 的 `Unreleased`，包括新功能、可感知修复、无障碍与性能改进、影响集成的平台或依赖变化以及迁移方式。网站和文档变更均不写入变更日志，即使与代码改动一同提交。按受影响区域分组、简述影响并在适用时注明贡献者。格式参考 [GPUI Kit v0.9.1 Release](https://github.com/longbridge/gpui-kit/releases/tag/v0.9.1)。

准备发版时，对照上一发布 tag 至今的全部提交核对 `Unreleased`，将本次变更移到带发布日期的新版本标题下，并保留空的 `Unreleased` 标题。该版本节即为 GitHub Release 正文（适用时附贡献者列表与 `Full Changelog` 比较链接）。

<a id="prepare-and-publish"></a>

## 准备与发布

GitHub Release 提供版本说明及源码归档。检出源码后用 `cargo gallery` 运行 Gallery。

1. **发布 PR**：更新共享工作区版本、内部依赖精确版本、`Cargo.lock`、`CHANGELOG.md` 以及文档中的最新版本与迁移说明。
2. **CI 与桌面冒烟检查**：确认 `main` 上 `CI required` 通过，并在 Windows 11 x64 上对发版提交做快速检查：
   - `cargo gallery`：确认窗口标题与图标、缩窄到紧凑布局的切换、状态栏主题与版本号，以及受影响的示例（涉及输入与浮层时附带检查微软拼音与讲述人）。
   - `cargo gallery --window-chrome`：确认标题栏拖拽、双击最大化/还原、悬停最大化触发 Windows 贴靠布局、**Block close** 开关，以及关闭最后一个窗口后进程正常退出。
3. **预演**：在 `main` 上手动触发 [Release 工作流](../.github/workflows/release.yml)（`version = X.Y.Z`，`publish: false`），核验工作区清单、变更日志提取，并在工作区 MSRV 下从独立下游 crate 检查全部六个库（`gpui-material-theme`、`gpui-material-foundation`、`gpui-material-components`、`gpui-material-extras`、`gpui-material`、`gpui-material-design-symbols`）的打包结果。
4. **正式发布**：获仓库所有者批准后，在 `main` 上运行 [Release 工作流](../.github/workflows/release.yml)（`version = X.Y.Z`，`publish: true`）。通过 `release` 环境审批后，工作流经由 crates.io Trusted Publishing（OIDC）按依赖顺序发布全部六个库（等待各版本进入 sparse index），并创建带附注的 `vX.Y.Z` tag 与 GitHub Release。

<a id="release-workflow-setup"></a>

### 发布工作流配置

[Release 工作流](../.github/workflows/release.yml)通过 GitHub Actions OIDC（`rust-lang/crates-io-auth-action`）配合 `release` GitHub Environment 完成发布，无需保存长期 `CARGO_REGISTRY_TOKEN` 密钥。

- 在 GitHub 仓库设置中配置 `release` 环境，开启必需审批人并限制仅允许 `main` 分支部署。
- 在 crates.io 上为全部六个库添加 Trusted Publishing 绑定，仓库填 `wangdiandao/gpui-material`，工作流文件名填 `release.yml`，环境名填 `release`。
