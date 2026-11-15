# 版本与发布

[English](release.md) | 中文

<a id="version-policy"></a>

## 版本规则

最新版本为 0.1.0。Material 各库与 Gallery 统一使用根清单 `[workspace.package]` 的版本。[公开门面及受支持的集成契约](integration.zh.md#api-与迁移)界定兼容范围，包括文档列出的 feature、公开签名中的上游类型，以及受支持的平台行为。

- 在 `1.0.0` 之前，向后兼容的修复递增 patch，向后兼容的新功能或破坏性变更递增 minor。递增 minor 时将 patch 归零。同一版本包含不同类型的变更时，采用最高的适用级别。
- 由仓库所有者决定何时以 `1.0.0` 承诺 API 稳定。从 `1.0.0` 起遵循 [SemVer](https://semver.org/lang/zh-CN/)。公开 API 的不兼容变更递增 major，兼容的新功能和公开 API 弃用递增 minor，兼容修复递增 patch。
- 所有破坏性变更都要记录变更与迁移步骤，此要求也适用于 `1.0.0` 之前。

<a id="changelog"></a>

## 变更日志

合并对用户或下游应用有影响的变更时，写入 [CHANGELOG.md](../CHANGELOG.md) 的 `Unreleased`。记录新功能、可感知的修复、无障碍和性能改进、影响集成的平台或依赖变化，以及迁移方式。按受影响的模块组织条目，说明实际影响，并附对应 PR 或 Issue，适用时注明贡献者。格式参考 [GPUI Kit v0.9.1 Release](https://github.com/longbridge/gpui-kit/releases/tag/v0.9.1)。

仓库所有者敲定版本号与发版范围后，核对上次发布 tag 到本次发版提交之间的全部变更。`0.1.0` 以最近的 Gallery 预发行版 `v0.0.4` 为基线。对照该范围补齐条目，将本次发布的变更移到带实际发布日期的版本标题下，并保留 `Unreleased` 收集后续变更。使用该版本条目作为 GitHub Release 正文，适用时列出贡献者，并附起止 tag 的 `Full Changelog` 比较链接。

<a id="prepare-and-publish"></a>

## 准备与发布

GitHub Release 提供版本说明及 GitHub 生成的源码归档。检出源码后用 `cargo gallery` 运行 Gallery。

1. 仓库所有者敲定版本、范围和发布时间。准备发布 PR，更新共享版本、内部依赖的精确版本、`Cargo.lock` 和变更日志。同步更新文档中的最新版本和依赖示例。有破坏性变更时补充迁移说明。
2. 使用最低支持的 Rust 工具链从下游 crate 验证门面，覆盖受影响的 feature 组合及公开的上游类型。发版提交合入 `main` 后须通过 `CI required`。按[实机验收标准](verification.zh.md)验证，在发布 Issue 或 PR 记录环境、场景及未覆盖范围。
3. 对于获准发布至 crates.io 的版本，一起打包五个库，并在下游 crate 中验证打包后的门面默认配置与全 feature 配置。实机验收后，从已验证的检出按依赖顺序发布，依次为 `gpui-material-theme`、`gpui-material-foundation`、`gpui-material-components`、`gpui-material-extras` 和 `gpui-material`。每个库的版本出现在 registry 后，再发布依赖它的库。
4. 仓库所有者明确批准发布后，为已验证的提交打 `vX.Y.Z` tag，使用该版本的日志发布对应的 GitHub Release。核对生成的源码归档及比较链接。
