# gpui-material

[中文](#中文) · [English](#english)

## 中文

基于 Rust 和 GPUI 的原生桌面 Material 3 Expressive 组件库。gpui-base 提供行为、输入、焦点和基础设施，gpui-material 定义主题、绘制、组件状态呈现和桌面自适应组合。

首版工程基线与验收结果见 [P2 工程基线](docs/phases/P02.md)。首版目标包括浅色与深色主题、品牌色和动态色、键盘与原生辅助技术、中文输入、RTL、窗口缩放和高 DPI。Windows 是主要验收平台，macOS 与 Linux 从 P1 起参与能力验证。各平台的验证范围见[平台与质量要求](docs/verification.md)。

在 Windows 11 x64/MSVC 环境运行 `cargo gallery` 可打开最小 Gallery 窗口。工具链、测试和 CI 制品见[开发与验证](docs/development.md)。组件示例随对应生产组件交付。

- [文档索引](docs/index.md)
- [组件覆盖矩阵](docs/component-coverage.md)
- [阶段状态](status.toml)

项目代码和原创文档采用 [Apache-2.0](LICENSE)。第三方参考及资源见[第三方声明](THIRD_PARTY_NOTICES.md)。

## English

A native desktop Material 3 Expressive component library for Rust and GPUI. gpui-base provides behavior, input, focus, and infrastructure. gpui-material defines themes, drawing, component state presentation, and adaptive desktop composition.

The initial release targets light and dark themes, brand and dynamic color, keyboard and native assistive technology, Chinese input, RTL, window resizing, and high DPI. Windows is the primary acceptance platform. macOS and Linux have participated in capability probes since P1. [P2 engineering baseline](docs/phases/P02.en.md) · [Platforms and quality requirements](docs/verification.en.md)

Run `cargo gallery` on Windows 11 x64/MSVC to open the minimal Gallery window. Component examples ship alongside their production components. [Toolchain, tests, and CI artifacts](docs/development.en.md)

- [Documentation index](docs/index.en.md)
- [Component coverage matrix](docs/component-coverage.en.md)
- [Phase status](status.toml)

Project code and original documentation are licensed under [Apache-2.0](LICENSE). [Third-party notices](THIRD_PARTY_NOTICES.md#english)
