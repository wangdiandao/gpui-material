# gpui-material

[English](README.md) | 中文

基于 Rust 和 GPUI 的原生桌面 Material 3 Expressive 组件库。gpui-base 提供行为、输入、焦点和基础设施，gpui-material 定义主题、绘制、组件状态呈现和桌面自适应组合。

首版目标包括浅色与深色主题、品牌色和动态色、键盘与原生辅助技术、中文输入、RTL、窗口缩放和高 DPI。Windows 是主要验收平台，macOS 与 Linux 从 P1 起参与能力验证。见 [P2 工程基线](docs/phases/P02.zh.md)和[平台与质量要求](docs/verification.zh.md)。

在 Windows 11 x64/MSVC 环境运行 `cargo gallery` 可打开 Gallery。组件示例通过 `gpui-material` 门面运行。工具链、测试和 CI 制品见[开发与验证](docs/development.zh.md)。

使用检出目录下的 `crates/material` 作为路径依赖。打开窗口前，用 `MaterialTheme` 初始化 `MaterialRoot`。`gpui_material::Button::new("save", "Save").on_activate(handler)` 创建带标签的操作按钮。选择控件通过调用方回调请求变更，滑块使用由 Base 支持的 `SliderState`。门面转发主题的可选功能 `dynamic-color` 和 `serde`。

- [P5 基础组件](docs/phases/P05.zh.md)
- [文档索引](docs/index.zh.md)
- [贡献指南](CONTRIBUTION.zh.md)
- [组件覆盖矩阵](docs/component-coverage.zh.md)
- [阶段状态](status.toml)

项目代码和原创文档采用 [Apache-2.0](LICENSE)。第三方参考及资源见[第三方声明](THIRD_PARTY_NOTICES.zh.md)。
