# gpui-material 贡献指南

[English](CONTRIBUTION.md) | 中文

## Issue 与 PR

- 通过[文档索引](docs/index.zh.md)和[阶段状态](status.toml)确认所属阶段和允许范围。新建 Issue 或 PR 前先搜索已有记录。
- 使用仓库的 Issue 模板。报告缺陷时附上最小复现、环境、预期行为和实际行为。提议功能时说明使用场景和验收标准。
- 从最新的 `main` 创建分支，遵循[分支命名约定](docs/development.zh.md#贡献流程与分支命名)。每个 PR 聚焦一个问题并关联对应 Issue。说明公开 API 的变更和迁移步骤。
- 按[开发指南](docs/development.zh.md#测试规范)运行相关检查，在 PR 中记录可复现的命令和结果。界面改动附上受影响的 Gallery 场景和修改前后的验证材料。

## 向上游提交 Issue 与贡献

超出 gpui-material 范围的问题，应向负责该行为的上游项目提交 Issue 或修复。本项目负责 Material 主题、绘制与呈现。可复用的控件行为和无障碍语义归 GPUI Kit 的 `gpui-base`。窗口、渲染和平台输入归 Zed 的 GPUI。 [项目架构](docs/architecture.zh.md)

1. 在受影响的上游版本上制作最小复现，核对上游当前分支，并搜索现有 Issue 或 PR。
2. 遵循对应仓库的贡献、测试、许可和评审规范。将补丁的范围和规模控制在修复问题及验证受影响行为所需的最小程度。 [GPUI Kit 贡献指南](https://github.com/longbridge/gpui-kit/blob/main/CONTRIBUTING.md) · [Zed 贡献指南](https://github.com/zed-industries/zed/blob/main/CONTRIBUTING.md)
3. 问题影响 gpui-material 时，在本地跟踪 Issue 中关联上游 Issue 或 PR，记录修复版本、依赖更新、回归检查和临时适配的移除条件。 [上游贡献](docs/development.zh.md#上游贡献)
