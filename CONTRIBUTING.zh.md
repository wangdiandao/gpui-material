# GPUI Material 贡献指南

[English](CONTRIBUTING.md) | 中文

## Issue 与 PR

- 先搜索现有 Issue 和 PR。使用[仓库模板](.github/ISSUE_TEMPLATE)，已有相同问题时关联原 Issue。缺陷报告附最小复现、环境、预期行为和实际行为。功能提议说明使用场景和验证方法。
- 从最新的 `main` 创建分支，使用 `<类型>/<简短主题>` 命名。每个 PR 聚焦一个问题并关联对应 Issue。 [分支命名约定](docs/development.zh.md#贡献流程与分支命名)

## 设计规范

- 控件样式、Token 和交互必须符合[当前锁定的设计规范](docs/design-sources.zh.md#material-specification)。界面 PR 引用对应的规范条目或 Token。需要评审桌面适配时，写明锁定规范的要求和建议行为。
- 设计规范版本升级只能通过[设计规范升级 Issue](.github/ISSUE_TEMPLATE/design_specification_update.md)提议。Issue 写明目标版本、涉及的组件与 Token，以及验证范围。由仓库维护者决定是否采纳和何时实施。获准后再提交实施 PR，并关联原 Issue。

## PR 评审与合并

- 遵守仓库的[编码规范](docs/development.zh.md#编码规范)和[测试规范](docs/development.zh.md#测试规范)。贡献者对提交的代码负责，包括对 AI 辅助编写的代码进行审查和测试。
- 按 crate 列出新增、修改或移除的公开 API，附签名和用途。破坏性变更须展示新旧用法，并在[变更日志](CHANGELOG.md)记录迁移方式。
- 记录可复现的验证命令、环境和结果。界面改动注明 Gallery 场景和已验证的变体与状态。截图或录像有助于评审时一并附上。
- 在[变更日志](CHANGELOG.md)的 `Unreleased` 下记录 crate 和 Gallery 中需要告知用户的变更。记录范围见[版本与发布](docs/release.zh.md#changelog)。
- 维护者审查改动范围、设计合规、API 兼容和验证结果。按评审意见更新 PR。在维护者批准、评审讨论解决且最新 PR 提交通过 `CI required` 后合并。

## 向上游提交 Issue 与贡献

超出 GPUI Material 范围的问题，应向负责该行为的上游项目提交 Issue 或修复。本项目负责 Material 主题、绘制与呈现。可复用的控件行为和无障碍语义归 GPUI Kit 的 `gpui-base`。窗口、渲染和平台输入归 Zed 的 GPUI。 [项目架构](docs/architecture.zh.md)

1. 在受影响的上游版本上制作最小复现，核对上游当前分支，并搜索现有 Issue 或 PR。
2. 遵循对应仓库的贡献、测试、许可和评审规范。将补丁的范围和规模控制在修复问题及验证受影响行为所需的最小程度。 [GPUI Kit 贡献指南](https://github.com/longbridge/gpui-kit/blob/main/CONTRIBUTING.md) · [Zed 贡献指南](https://github.com/zed-industries/zed/blob/main/CONTRIBUTING.md)
3. 问题影响 GPUI Material 时，在本地跟踪 Issue 中关联上游 Issue 或 PR，记录修复版本、依赖更新、回归检查和临时适配的移除条件。 [上游贡献](docs/development.zh.md#上游贡献)
