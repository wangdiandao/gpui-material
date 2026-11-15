# 下游集成

[English](integration.md) | 中文

使用支持 2024 edition 的 Rust 1.99.1 或更新版本。已验证的最低工具链为 Rust 1.99.1。从检出目录使用门面，在应用中固定检出提交，并将应用的 Cargo 锁文件纳入版本控制。

## 添加门面

下面假设 Material 检出目录与应用目录同级。精确匹配 GPUI 包身份，让门面和应用共享同一套 `App`、`Window` 和 `Entity` 类型。

```toml
[dependencies]
gpui = { package = "gpui-pre", version = "=0.3.12" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.12", features = ["font-kit", "x11", "wayland", "runtime_shaders"] }
gpui-material = { path = "../gpui-material/crates/material" }
```

应用直接使用 Base 时也应固定 `gpui-base = "=0.9.1"`。更新检出提交时，运行 `cargo tree -d` 检查重复版本，并验证应用所依赖的平台和输入行为。

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

应用通过[公开门面](../crates/material/src/lib.rs)集成，包括已记录的 `foundation` 和 `theme` 路径。按 [README 中的特性名](../README.zh.md)启用可选组件系列。`date-time` 的 API 位于 `gpui_material::date_time` 下。`dynamic-color` 和 `serde` 转发到纯主题 crate。公开签名中的 GPUI 和 Base 类型遵循固定的上游版本。Gallery 模块和工作区 crate 的内部组织可能调整。

使用 0.0.4 门面的应用可将检出目录移至 P9 验收提交，沿用同样的导入、特性名和状态初始化方式。下一版本仍处于 1.0 之前。升级次版本时核对 API 契约变化及迁移步骤。

## 平台支持

| 目标 | 范围 |
| --- | --- |
| Windows 11 x64、MSVC、DirectX | 首发支持。原生键盘、指针、鼠标滚轮、微软拼音、UI Automation 和讲述人采用 Windows 验收场景。分发包为独立 ZIP。 |
| macOS 15 ARM64 | CI 构建与自动测试。原生支持处于探索阶段。 |
| Ubuntu 24.04 x64 | CI 构建与自动测试。原生支持处于探索阶段。 |

触摸优先手势和尚未验证的触控板或混合 DPI 硬件组合不在初版原生覆盖范围内。固定版本中的 Windows Base 滑块支持键盘和语义递增、递减操作。UI Automation RangeValue 编程写入由 [#49](https://github.com/wangdiandao/gpui-material/issues/49) 跟踪。GPUI 的原生指针捕获丢失由 [#48](https://github.com/wangdiandao/gpui-material/issues/48) 跟踪。

[平台验收场景](verification.zh.md) · [固定来源](design-sources.zh.md) · [P9 验收](phases/P09.zh.md)
