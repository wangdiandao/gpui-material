# gpui-material

基于 Rust 和 GPUI 的原生桌面 Material 3 Expressive 组件库。gpui-base 提供行为、输入、焦点和基础设施，gpui-material 定义主题、绘制、组件状态呈现和桌面自适应组合。

首版工程基线与验收结果见 [P2 工程基线](docs/phases/P02.md)。首版目标包括浅色与深色主题、品牌色和动态色、键盘与原生辅助技术、中文输入、RTL、窗口缩放和高 DPI。Windows 是主要验收平台，macOS 与 Linux 从 P1 起参与能力验证。各平台的验证范围见[平台与质量要求](docs/verification.md)。

在 Windows 11 x64/MSVC 环境运行 `cargo gallery` 可打开最小 Gallery 窗口。工具链、测试和 CI 制品见[开发与验证](docs/development.md)。组件示例随对应生产组件交付。

- [文档索引](docs/index.md)
- [组件覆盖矩阵](docs/components/coverage.md)
- [阶段状态](status.toml)

项目代码和原创文档采用 [Apache-2.0](LICENSE)。第三方参考及资源见[第三方声明](THIRD_PARTY_NOTICES.md)。
