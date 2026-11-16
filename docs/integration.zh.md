# 下游集成

[English](integration.md) | 中文

使用支持 2024 edition 的 Rust 1.99.1 或更新版本，并将应用的 Cargo 锁文件纳入版本控制。

## 添加门面

从 crates.io 添加门面。精确匹配 GPUI 包身份，让门面和应用共享同一套 `App`、`Window` 和 `Entity` 类型。

```toml
[dependencies]
gpui = { package = "gpui-pre", version = "=0.3.12" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.12", features = ["font-kit", "x11", "wayland", "runtime_shaders"] }
gpui-material = "0.1.0"
```

使用已发布的 0.1.0 门面时，直接使用 Base 的应用应固定 `gpui-base = "=0.9.1"`。从源码集成时，使用 `{ path = "../gpui-material/crates/material" }`，固定检出提交并匹配其 `gpui-base` 版本。当前源码工作区使用 `gpui-base = "=0.10.0"`。升级后运行 `cargo tree -d` 检查重复版本，并验证应用所依赖的平台和输入行为。

```rust
use gpui::{AppContext, Context, IntoElement, QuitMode, Render, Window, WindowOptions};
use gpui_material::{Button, MaterialRoot, MaterialTheme};

struct Demo;

impl Render for Demo {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Button::new("save", "Save").on_activate(|_, _| println!("Save requested"))
    }
}

fn main() {
    gpui_platform::application()
        .with_quit_mode(QuitMode::LastWindowClosed)
        .run(|cx| {
            MaterialRoot::init(MaterialTheme::default(), cx);
            gpui_material::init_components(cx);
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| cx.new(|_| Demo))
                .expect("open Material window");
        });
}
```

`MaterialRoot::init` 安装应用主题并初始化 Base。`init_components` 安装组件键绑定。通过 `MaterialRoot::open_window` 打开的窗口带有管理浮层和焦点遍历的 Base 根节点。受控值和命令结果由应用持有。`SliderState` 和 `TextFieldState` 等实体状态应跨渲染保留。

## API 与迁移

Search 和 dropdown 的虚拟化无需调用方迁移。结果和选项的 ID 必须唯一，并在重排时保持稳定。`ListItem::track_focus(&FocusHandle, bool)` 允许调用方的虚拟集合提供保留的行焦点及 roving tab stop；该集合还须负责完整模型的导航，并滚动显示屏外目标。现有 `List` API 保持不变。

`foundation::set_window_material_theme` 覆盖 Material 呈现，包括显式传入 Base 编辑器的颜色和 Material 浮层表面。独立 Base 控件、滚动条、resize handle 及光标闪烁策略仍使用应用级 Base 全局主题；这些集成需要显式局部样式。窗口渲染不会切换 Base 全局主题。

应用通过[公开门面](../crates/material/src/lib.rs)集成，包括已记录的 `foundation` 和 `theme` 路径。按 [README 中的特性名](../README.zh.md)启用可选组件系列。`date-time` 的 API 位于 `gpui_material::date_time` 下。`dynamic-color` 和 `serde` 转发到纯主题 crate。公开签名中的 GPUI 和 Base 类型遵循固定的上游版本。Gallery 模块和工作区 crate 的内部组织可能调整。

使用 0.0.4 门面的应用升级至 0.1.0 时可沿用导入、feature 名称和状态初始化方式。若对 `IconName` 使用穷尽匹配，为四个新增变体添加分支，或用一个兜底分支处理行为相同的变体。变体名称和其他变更见[变更日志](../CHANGELOG.md)。

## 平台支持

| 目标 | 范围 |
| --- | --- |
| Windows 11 x64、MSVC、DirectX | 首发支持。原生键盘、指针、鼠标滚轮、微软拼音、UI Automation 和讲述人采用 Windows 验收场景。分发包为独立 ZIP。 |
| macOS 15 ARM64 | CI 构建与自动测试。原生支持仍在探索。 |
| Ubuntu 24.04 x64 | CI 构建与自动测试。原生支持仍在探索。 |

[平台验收场景](verification.zh.md) · [固定来源](design-sources.zh.md) · [版本与发布](release.zh.md)
