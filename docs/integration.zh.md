# 下游集成

[English](integration.md) | 中文

使用支持 2024 edition 的 Rust 1.95.0 或更新版本，并将应用的 Cargo 锁文件纳入版本控制。

## 添加门面

从 crates.io 添加门面。精确匹配 GPUI 包身份，让门面和应用共享同一套 `App`、`Window` 和 `Entity` 类型。

```toml
[dependencies]
gpui = { package = "gpui-pre", version = "=0.3.14" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.14", features = ["font-kit", "x11", "wayland", "runtime_shaders"] }
gpui-material = "0.4.1"
```

应用直接使用 Base 时也应固定 `gpui-base = "=0.11.0"`。从源码集成时，使用 `{ path = "../gpui-material/crates/material" }` 并固定检出提交。升级后运行 `cargo tree -d` 检查是否存在重复包版本。

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
            gpui_material::init(MaterialTheme::default(), cx);
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| cx.new(|_| Demo))
                .expect("open Material window");
        });
}
```

`gpui_material::init` 单步完成应用主题安装、Base 初始化，并在启用组件特性时注册组件键绑定（内部调用 `MaterialRoot::init` 以及在 `display` 启用时调用 `init_components`）。通过 `MaterialRoot::open_window` 打开的窗口带有管理浮层和焦点遍历的 Base 根节点。受控值和命令结果由应用持有。`SliderState` 和 `TextFieldState` 等实体状态应跨渲染保留。

<a id="material-symbols"></a>

## Material Symbols

需要完整图标目录时，在 `gpui-material` 之外添加 `gpui-material-design-symbols = "0.4.1"`。从源码集成时使用 `gpui-material-design-symbols = { path = "../gpui-material/crates/design-symbols" }`。图标目录包含固定版本的全部 4,304 个名称，包括别名。通过 `Symbol::ALL` 遍历，或通过 `Symbol::from_name("favorite")` 查找上游名称。[上游码点表和字体资源](https://github.com/google/material-design-icons/tree/10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7/variablefont)

桌面端为 `gpui-material-design-symbols` 启用 `embed-assets`，调用 `SymbolFont::embedded` 加载 Outlined、Rounded 或 Sharp 字体。字体保留 `FILL` 和 `wght`，grade 固定为 0，光学尺寸固定为 24。`SymbolAxes::default()`（在 `const` 上下文中可使用 `SymbolAxes::new()`）使用未填充样式和 400 字重。`.filled(true)` 选择 FILL=1，`Weight::new(550)` 可指定 100–700 范围内的任意整数值，`is_filled()` 和 `get_weight()` 可读取已配置的轴值。

```rust
use gpui_material::Icon;
use gpui_material_design_symbols::{Symbol, SymbolAxes, SymbolFont, SymbolStyle, Weight};

let font = SymbolFont::embedded(SymbolStyle::Rounded);
let axes = SymbolAxes::default().filled(true).weight(Weight::BOLD);
let favorite = font.svg_data(Symbol::Favorite, axes).expect("favorite glyph");
let icon = Icon::svg_data(favorite);
```

如果同一个图标及轴值会在多次渲染中复用，可将生成的 SVG 字节保存在视图或应用状态中。`Icon::svg_data` 继承图标槽的尺寸和颜色，支持 `.label(...)`、`.size(...)`、`.color(...)`、旋转和 RTL 镜像。填充和字重在生成 SVG 时应用。`Icon::new(IconName::Favorite)` 使用零配置的内置系统图标。

浏览器 WASM 端将所需的 `.ttf` 和许可证作为独立静态资源部署。应用决定何时加载某个样式，可将返回的 `SymbolFont` 或生成的 SVG 字节保存在视图状态中。等待期间可显示内置图标，替换后通知 GPUI。`fetch` 会返回网络、HTTP 和无效字体错误。使用同源 URL，或在资源服务器上配置 CORS。

```rust
use gpui_material_design_symbols::{Symbol, SymbolAxes, SymbolFont};

let font = SymbolFont::fetch("/fonts/material-symbols-rounded.ttf").await?;
let favorite = font.svg_data(Symbol::Favorite, SymbolAxes::default())?;
```

[配套资源](../crates/design-symbols/assets/)中有三份可直接部署的字体。`embed-assets` 仅在原生目标内嵌字体，即使 Cargo 在 WASM 构建中启用了该 feature。使用自定义子集或自行下载时，将 TrueType 字节传给 `SymbolFont::from_bytes`。请求子集缺少的图标时返回 `SymbolError::MissingGlyph`。

重新生成资源时，把上游三份 `.ttf` 文件、对应的 `.codepoints` 文件和上游 `LICENSE` 放在同一个目录。安装 fontTools 4.63.0，在检出目录运行 `python xtask/material-symbols.py <source-directory>`。[固定输入与校验和](../crates/design-symbols/assets/manifest.toml)

## API 与迁移

Search 和 dropdown 的虚拟化无需调用方迁移。结果和选项的 ID 必须唯一，并在重排时保持稳定。`ListItem::track_focus(&FocusHandle)` 配合 `.tab_stop(bool)` 允许调用方的虚拟集合提供保留的行焦点及 roving tab stop；该集合还须负责完整模型的导航，并滚动显示屏外目标。现有 `List` API 保持不变。

`set_material_theme` 使用目标动效方案的 Slow Effects 弹簧过渡应用色彩。切换途中收到新主题时，从当前显示的色谱开始过渡。初始化和减少动态效果模式立即应用色彩。目标排版和布局在请求时生效。应用主题投影把应用色谱和缩放后的文字指标同步给 Base。

`set_window_material_theme`（与 `window_material_theme` 一起同时在门面根路径和 `foundation` 下导出）设置单个窗口的 Material 主题。通过 `ThemeScope::new(child).theme(theme).direction(Direction::Rtl).strings(strings)` 覆盖子树，所有构建器均可省略：仅设置方向或文案时，主题继承自外层。Material 后代、多行 `TextField` 滚动条、延迟退出浮层以及 `ThemeScope::bind_action` 回调优先读取最近的作用域，其次是窗口和应用默认值，且不会篡改 `gpui_base::Theme::global`。独立 Base 控件可调用 `foundation::current_base_theme(window, cx)`（或 `foundation::project_base_theme(theme, cx)`）获取与当前局部作用域一致的 Base Token 与滚动条样式。

内置的默认控件标签与无障碍文案（如 `Banner` 的关闭提示、`TextField` 的清除和密码显隐切换、`ExposedDropdown` 的选项开关、`Dialog` 的取消按钮、`Sheet` 的关闭与调整手柄、`Scrim` 的遮罩点击提示、溢出菜单触发器以及 `SearchView` / `CaptionButtons` 默认文案）在未显式指定组件级标签时统一从 `MaterialStrings` 读取。调用 `set_material_strings(strings, cx)`（通过 `material_strings(cx)` 或 `current_material_strings(window, cx)` 读取）即可在零第三方国际化依赖下全局本地化默认文案，也可通过 `ThemeScope::new(child).strings(strings)` 为局部子树（含经由 `ThemeScope::inherit` 传递的延迟浮层）单独覆盖。显式的组件级设置器（如 `.dismiss_label(...)`、`.clear_label(...)`、`.close_label(...)`、`.accessibility_label(...)` 等）始终优先于 `MaterialStrings`。

如需自动跟随操作系统外观与减少动态效果偏好而无需手动映射 `WindowAppearance`，可在启动时用 `system_appearance(cx)`（或 `appearance_from_window(window.appearance())`，将 `Light`/`VibrantLight` 映射为 `Appearance::Light`，将 `Dark`/`VibrantDark` 映射为 `Appearance::Dark`）读取初始外观，并通过 `observe_system_appearance(window, cx, AppearanceMode::System, build_theme)`（或 `MaterialRoot::sync_system_appearance`）保持窗口同步。该辅助函数会立即应用当前解析的主题、订阅 `window.observe_window_appearance`、在解析后的 `Appearance` 未变化时跳过冗余更新，并通过 `set_material_theme` 平滑更新应用主题（同时保留其他窗口的独立覆盖）。若只需同步单个窗口而不修改全局应用主题，请使用 `observe_window_system_appearance`（或 `MaterialRoot::sync_window_system_appearance`），它会通过 `set_window_material_theme` 更新目标窗口。`gpui_material::init` 已通过 `gpui-base` 读取一次系统减少动态效果偏好；需要按需重新读取系统设置时调用 `sync_system_reduce_motion(cx)`，需要显式覆盖或恢复跟随系统时调用 `set_reduce_motion_preference(ReduceMotionPreference::..., cx)`。

应用通过[公开门面](../crates/material/src/lib.rs)集成，包括已记录的 `foundation` 和 `theme` 路径。`gpui-material` 默认启用 `full`，编译全部核心组件分组（`display`、`actions`、`selection`、`inputs`、`surfaces`、`settings-tiles`、`feedback`、`navigation`）。如需极致裁剪，可设置 `default-features = false` 并按需列出特性（例如 `features = ["display"]` 仅启用基础展示原语、`features = ["actions", "inputs"]` 仅编译所选分组，或 `features = ["window-chrome"]` / `features = ["carousel"]` 完全不编译 `gpui-material-components` 及其内嵌 SVG）。按 [README 中的特性名](../README.zh.md)启用可选组件系列，每个可选组件系列仅激活其依赖的核心分组。`date-time` 的 API 及重导出的 `jiff::civil::{Date, Time, Weekday}` 位于 `gpui_material::date_time` 下。`dynamic-color` 和 `serde` 转发到纯主题 crate。公开签名中的 GPUI 和 Base 类型遵循固定的上游版本。Gallery 模块和工作区 crate 的内部组织可能调整。

升级至 0.4.0 时，建议在启动阶段改用单步 `gpui_material::init(theme, cx)`，将 `MaterialTheme` 的 `with_*` 构建器改为裸名（`colors`、`typography`、`motion`、`layout`、`density`、`direction`），并将各组件独立的边距/形状/描边/海拔结构体统一替换为 `LogicalEdges`、`ControlShapes`、`ControlBorder` 与 `ControlElevations`。将 `EmptyState` 更名为 `EmptyView`，将浮层关闭回调统一改为 `.on_closed(...)`；使用 `default-features = false` 时按需显式启用 `display` 或 `settings-tiles` 特性；`date_time` 调用方需从 `chrono` 迁移至 `gpui_material::date_time` 重导出的 `jiff::civil::{Date, Time, Weekday}`。完整破坏性变更见[变更日志](../CHANGELOG.md)。

升级至 0.3.3 时，将直接使用的 GPUI 同步至 `0.3.14`，并将 Base 同步至 `0.11.0`。若直接调用 `foundation::Shape` 裁剪接口，需按 `Option<gpui::Path>` 适配 `clip_rect`、`clip_disc` 和 `clip_convex_polygon`，并将 `clip_path`/`paint_clipped` 替换为 `clip_shape`/`paint_clipped_shape`。

升级至 0.3.0 时，请将 Cargo 特性 `windows-chrome` 与模块路径 `gpui_material::windows_chrome` 替换为跨平台的 `window-chrome` 特性与 `gpui_material::window_chrome` 模块；若对 `IconName` 使用穷尽匹配，需补充 `IconName::Visibility` 与 `IconName::VisibilityOff` 分支。

升级至 0.2.0 时，若对 `ColorRole` 使用穷尽匹配，需补充九个新增的语义状态变体（`Info`、`OnInfo`、`InfoContainer`、`Success`、`OnSuccess`、`SuccessContainer`、`Warning`、`OnWarning` 和 `WarningContainer`）。最低支持的 Rust 版本调整为 `1.95.0`，`gpui-material` 默认仍启用 `full`，现有组件导入可直接兼容。

升级至 0.1.1 时，将直接使用的 Base 同步到 0.10.0，并将配套 GPUI 固定在 0.3.12。对 `AppBarVariant::TwoRows`、`DatePickerVariant::FullScreen`、`SearchViewVariant::FullScreenContained` 和 `TimePickerVariant::Scroll` 补充穷尽匹配分支。键盘和无障碍操作改变滑块数值时，先发出 `Change`，再发出 `Release`。原先只把 `Release` 当作指针事件的订阅方应调整逻辑。

使用 0.0.4 门面的应用升级至 0.1.0 时可沿用导入、feature 名称和状态初始化方式。若对 `IconName` 使用穷尽匹配，为四个新增变体添加分支，或用一个兜底分支处理行为相同的变体。变体名称和其他变更见[变更日志](../CHANGELOG.md)。

## 平台支持

启用 `window-chrome` 即可在 Windows、macOS 和 Linux 上为指定窗口使用[跨平台自绘标题栏](component.zh.md#window-chrome)。创建窗口时调用 `title_bar_window_options`，并在现有应用根视图下挂载 `TitleBar`。

| 目标 | 状态 |
| --- | --- |
| Windows 11 x64（MSVC、DirectX） | 主要支持的桌面平台，覆盖原生键盘/指针/滚轮、微软拼音 IME、UI Automation 与讲述人。 |
| macOS 15 ARM64 | 纳入 CI 构建与自动化测试，原生桌面适配持续推进。 |
| Ubuntu 24.04 x64 | 纳入 CI 构建与自动化测试，原生桌面适配持续推进。 |

<a id="ui-testing"></a>

## UI 测试

仅在 `[dev-dependencies]` 中启用 `test-support`，让生产构建不引入任何观测开销：

```toml
[dev-dependencies]
gpui = { package = "gpui-pre", version = "=0.3.14", features = ["test-support"] }
gpui-material = { version = "0.4.1", features = ["test-support"] }
```

启用 `test-support` 后，`gpui_material::test` 会导出完全基于 GPUI 与 `gpui-base` 公开 API 构建的无头 UI 测试工具（`TestWindowExt`、`ScopedWindow`、`ClickOptions`、`TestQueryExt`、`TestEventExt`、`TestInput`、`TestAppContextExt::wait_for`、`ElementSnapshot` 与 `TestSupportExt`）。自定义元素通过 `div().id("status").test_support()` 注册观测（可聚焦元素须置于 `.track_focus(&handle)` 之前），测试窗口使用 `MaterialRoot::open_window` 打开，并同时断言原生无障碍快照与应用业务状态：

```rust
use gpui::{AppContext, TestAppContext, WindowOptions};
use gpui_material::{
    MaterialRoot, MaterialTheme,
    test::{TestSupportExt as _, TestWindowExt as _},
};

#[gpui::test]
fn saves_form_through_the_ui(cx: &mut TestAppContext) {
    let (handle, view) = cx.update(|cx| {
        gpui_material::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(true);
        let mut entity = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let v = cx.new(|cx| MyForm::new(window, cx));
            entity = Some(v.clone());
            v
        })
        .unwrap();
        (handle, entity.unwrap())
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("name", cx);
        window.input("Ada", cx);
        window.press("backspace", cx);
        window.click("save", cx);
        assert_eq!(window.find("status").label(), Some("Saved: Ad"));
    })
    .unwrap();
}
```

对于延迟或异步 UI 状态，请在 `update_window` 外部调用 `cx.wait_for(handle.into(), timeout, |window, cx| ...).await`。若测试还需要真实 GPU 光栅化校验，可通过 `Window::render_to_image` 在 Windows（DirectX，含 WARP 回退）、macOS（Metal）与 Linux（WGPU / lavapipe）上无可见窗口地捕获离屏 RGBA 帧，示例见 [apps/gallery/tests/rendering.rs](../apps/gallery/tests/rendering.rs)。可执行端到端示例见 [crates/material/tests/integration/ui.rs](../crates/material/tests/integration/ui.rs)，完整分层与约定见[测试规范](development.zh.md#testing-conventions)。

[实战手册](cookbook.zh.md) · [设计依据](design-sources.zh.md) · [版本与发布](release.zh.md)
