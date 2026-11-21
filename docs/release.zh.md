# 版本与发布

[English](release.md) | 中文

<a id="version-policy"></a>

## 版本规则

最新版本为 0.1.2。Material 各库与 Gallery 统一使用根清单 `[workspace.package]` 的版本。[公开门面及受支持的集成契约](integration.zh.md#api-与迁移)界定兼容范围，包括文档列出的 feature、公开签名中的上游类型，以及受支持的平台行为。

- 在 `1.0.0` 之前，向后兼容的修复递增 patch，向后兼容的新功能或破坏性变更递增 minor。递增 minor 时将 patch 归零。同一版本包含不同类型的变更时，采用最高的适用级别。
- 由仓库所有者决定何时以 `1.0.0` 承诺 API 稳定。从 `1.0.0` 起遵循 [SemVer](https://semver.org/lang/zh-CN/)。公开 API 的不兼容变更递增 major，兼容的新功能和公开 API 弃用递增 minor，兼容修复递增 patch。
- 所有破坏性变更都要记录变更与迁移步骤，此要求也适用于 `1.0.0` 之前。

<a id="changelog"></a>

## 变更日志

Material 各 crate 和 Gallery 中影响用户或下游应用的变更合并时，写入 [CHANGELOG.md](../CHANGELOG.md) 的 `Unreleased`。记录新功能、可感知的修复、无障碍和性能改进、影响集成的平台或依赖变化，以及迁移方式。网站和文档变更均不写入变更日志，包括与 crate 或 Gallery 改动一起提交的情况。按受影响的模块组织条目，说明实际影响，适用时注明贡献者。实现与验证记录放在对应 Issue 或 PR。格式参考 [GPUI Kit v0.9.1 Release](https://github.com/longbridge/gpui-kit/releases/tag/v0.9.1)。

仓库所有者敲定版本号与发版范围后，核对上次发布 tag 到本次发版提交之间的全部变更。`0.1.0` 以最近的 Gallery 预发行版 `v0.0.4` 为基线。对照该范围补齐条目，将本次发布的变更移到带实际发布日期的版本标题下，并保留 `Unreleased` 收集后续变更。使用该版本条目作为 GitHub Release 正文，适用时列出贡献者，并附起止 tag 的 `Full Changelog` 比较链接。

<a id="prepare-and-publish"></a>

## 准备与发布

GitHub Release 提供版本说明及 GitHub 生成的源码归档。检出源码后用 `cargo gallery` 运行 Gallery。

1. 仓库所有者敲定版本、范围和发布时间。准备发布 PR，更新共享版本、内部依赖的精确版本、`Cargo.lock` 和变更日志。同步更新文档中的最新版本和依赖示例。有破坏性变更时补充迁移说明。
2. 使用最低支持的 Rust 工具链从下游 crate 验证门面，覆盖受影响的 feature 组合及公开的上游类型。发版提交合入 `main` 后须通过 `CI required`。按[实机验收标准](verification.zh.md)验证，在发布 Issue 或 PR 记录环境、场景及未覆盖范围。
3. 在 `main` 上手动运行 [Release 工作流](../.github/workflows/release.yml)（填入 `version` 为 `X.Y.Z`，保持 `publish: false` 预演模式），在正式发布前核验工作区清单、变更日志提取与打包结果。验证作业会打包全部六个库（`gpui-material-theme`、`gpui-material-foundation`、`gpui-material-components`、`gpui-material-extras`、`gpui-material` 和 `gpui-material-design-symbols`），并在工作区 MSRV 下从独立下游 crate 检查解包后的归档：包括 `gpui-material` 的默认配置和 `--all-features`，以及 `gpui-material-design-symbols` 启用与禁用 `embed-assets` 的配置。
4. 完成实机验收并获仓库所有者明确批准后，在 `main` 上运行 [Release 工作流](../.github/workflows/release.yml)（填入 `version` 为 `X.Y.Z` 并勾选 `publish: true`）。通过 `release` 环境审批后，工作流会通过 crates.io Trusted Publishing（OIDC）鉴权，按依赖顺序依次发布门面依赖（`gpui-material-theme`、`gpui-material-foundation`、`gpui-material-components`、`gpui-material-extras`）、`gpui-material` 与独立的 `gpui-material-design-symbols`，在发布下游前等待各版本出现在 crates.io sparse index 并校验 registry 校验和。随后在同一运行中基于对应变更日志节创建带附注的 `vX.Y.Z` tag 与 GitHub Release，并核对生成的源码归档和比较链接。

<a id="release-workflow-setup"></a>

### 发布工作流配置

[Release 工作流](../.github/workflows/release.yml)通过 GitHub Actions OIDC（`rust-lang/crates-io-auth-action`）配合 `release` GitHub Environment 完成发布，无需保存长期 `CARGO_REGISTRY_TOKEN` 密钥。

- 在 GitHub 仓库设置中配置 `release` 环境，开启必需审批人并限制仅允许 `main` 分支部署。
- 在 crates.io 上为全部六个库添加 Trusted Publishing 绑定，仓库填 `wangdiandao/gpui-material`，工作流文件名填 `release.yml`，环境名填 `release`。
- crates.io 要求 crate 已存在后才能配置 Trusted Publishing。首次发布全新 crate（例如首个 `gpui-material-design-symbols` 版本）时，先从同一已验证提交手动发布该 crate 的首个版本并随后配置 Trusted Publishing；工作流会自动识别已上架且未撤回（non-yanked）的版本、核验其校验和并跳过重复发布，保证重试与断点续跑的幂等性。
