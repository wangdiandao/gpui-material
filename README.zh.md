# GPUI Material

[English](README.md) | 中文

<p align="center">
  <img src="./website/public/logo.svg" width="112" height="112" alt="GPUI Material 标志" />
</p>

为基于 GPUI 的 Rust 原生桌面应用提供 Material 3 Expressive 组件。组件库负责 Material 配色、排版、形状和动效，GPUI Base 提供输入、焦点和窗口服务。

## 体验 Gallery

从源码运行 Gallery。你可以从 Overview 首页浏览组件概览，在顶栏切换 Components、Styles（Color、Typography、Shape、Motion）和 Examples 分区，在分组侧栏中搜索过滤，并切换内置主题预设及浅色/深色外观。在 Adaptive layout 页面调整窗口大小，可以观察导航和内容窗格如何随可用空间变化。

```sh
git clone https://github.com/wangdiandao/gpui-material.git
cd gpui-material
cargo gallery
```

Windows 11 x64/MSVC 是主要支持环境。macOS ARM64 和 Linux x64 纳入 CI 构建和自动化测试。各平台的构建条件见[开发指南](docs/development.zh.md)。

## 在应用中使用

使用 Rust 1.95.0 或更高版本及 2024 Edition。添加依赖 `gpui-material = "0.4.1"`。[集成指南](docs/integration.zh.md)列出了匹配的 GPUI 包，并提供完整的窗口示例。

先初始化 `MaterialRoot` 并调用 `gpui_material::init_components`，再用 `MaterialRoot::open_window` 打开窗口。通过 `gpui_material` 门面组合界面，在应用中管理受控值和命令结果。

```rust
use gpui_material::Button;

let save = Button::new("save", "Save")
    .on_activate(|_, _| println!("Save requested"));
```

## 组件与特性

- **操作和选择。** 按钮、悬浮操作按钮、复选框、单选组、开关、芯片和滑块。
- **内容和反馈。** 卡片、列表、徽标、进度指示器、加载指示器和 Snackbar。
- **输入和浮层。** 填充、描边、多行和安全文本字段，以及下拉选择、菜单、提示、对话框和 Sheet。
- **导航和布局。** 应用栏、标签页、导航栏、导航轨道、Scaffold、列表详情、辅助窗格和 Feed。

在 `gpui-material` 上按需启用可选特性，以使用更多组件和主题功能。

| 特性 | 提供能力 |
| --- | --- |
| `expressive-groups` | 按钮组和分割按钮 |
| `toolbars` | FAB 菜单、悬浮工具栏和停靠工具栏 |
| `search` | 搜索栏和搜索视图 |
| `date-time` | 日期、日期范围和时间选择器 |
| `carousel` | 六种轮播布局 |
| `data-views` | 数据表、虚拟化层级树和虚拟列表 |
| `disclosure` | 可展开/收起容器、手风琴与桌面分页栏 |
| `command-palette` | `Ctrl+K` / `Cmd+K` 风格模态命令面板 |
| `specialized-inputs` | 数值步进输入、验证码输入和颜色选择器 |
| `rich-text` | 可选 Material 文本和 Markdown/HTML 富文本视图 |
| `window-chrome` | 跨平台自绘标题栏和窗口控制按钮 |
| `dynamic-color` | 从种子色生成配色方案 |
| `serde` | 主题数据序列化 |

当前支持的控件、变体及可选组件特性见[组件文档](docs/component.zh.md)。

## 项目文档

- [文档网站](https://wangdiandao.github.io/gpui-material/zh-CN/)
- [下游集成](docs/integration.zh.md)
- [架构与依赖](docs/architecture.zh.md)
- [文档索引](docs/index.zh.md)
- [贡献指南](CONTRIBUTING.zh.md)
- [变更日志](CHANGELOG.md)

## 许可证

项目原创代码和文档采用 [Apache-2.0](LICENSE)。随包资源和依赖的许可见[第三方声明](THIRD_PARTY_NOTICES.zh.md)。
