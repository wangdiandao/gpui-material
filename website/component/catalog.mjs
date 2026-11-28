// Each guide shares its Rust snippet across the two locales. A story defaults to the page slug.
export const components = [
  {
    slug: "button",
    group: "basic",
    title: "Button",
    source: "crates/components/src/button.rs",
    api: "struct.Button.html",
    description: {
      en: "Run a command with a labeled action.",
      zh: "用有文本标签的按钮执行命令。",
    },
    usage: {
      en: "Use on_activate for pointer, keyboard, and accessibility activation. Filled is the default variant. Set disabled when the command is unavailable, or loading(true) to show an inline circular spinner while suppressing activation and preserving width.",
      zh: "用 on_activate 统一处理指针、键盘和无障碍激活。默认是 Filled。命令不可用时设置 disabled，忙碌中设置 loading(true) 在保留宽度的同时显示内联环形指示器并屏蔽激活。",
    },
    imports: "use gpui_material::Button;",
    code: `Button::new("save", "Save")
    .on_activate(|_, _| println!("Save requested"))`,
  },
  {
    slug: "toggle-button",
    group: "basic",
    title: "ToggleButton",
    source: "crates/components/src/button.rs",
    api: "struct.ToggleButton.html",
    description: {
      en: "Toggle a persistent selection with a text label.",
      zh: "通过有文本标签的按钮切换选中状态。",
    },
    usage: {
      en: "Pass the current selection to new. Use the requested boolean from on_change to update your application state and rerender.",
      zh: "将当前选中状态传给 new。在 on_change 中使用回调给出的布尔值更新应用状态并重新渲染。",
    },
    imports: "use gpui_material::ToggleButton;",
    code: `ToggleButton::new("pin", "Pin", is_pinned)
    .on_change(|selected, _, _| println!("Pinned: {selected}"))`,
  },
  {
    slug: "icon-button",
    group: "basic",
    title: "IconButton",
    source: "crates/components/src/button.rs",
    api: "struct.IconButton.html",
    description: {
      en: "Run a command represented by an icon.",
      zh: "通过图标执行命令。",
    },
    usage: {
      en: "Provide an accessible name even when no label is visible. Use the variant, width, and size builders to change appearance.",
      zh: "即使没有可见文字也要提供无障碍名称。用 variant、width 和 size 调整外观。",
    },
    imports: "use gpui_material::{Icon, IconButton, IconName};",
    code: `IconButton::new("archive", "Archive", Icon::new(IconName::Archive))
    .on_activate(|_, _| println!("Archive requested"))`,
  },
  {
    slug: "icon-toggle-button",
    group: "basic",
    title: "IconToggleButton",
    source: "crates/components/src/button.rs",
    api: "struct.IconToggleButton.html",
    story: "icon-button",
    description: {
      en: "Expose an on or off state using an icon.",
      zh: "用图标表达开关状态。",
    },
    usage: {
      en: "Keep the selected value in the application. Give the control an accessible name and optionally supply a different selected icon.",
      zh: "由应用持有选中值。为控件提供无障碍名称，还可单独配置选中时的图标。",
    },
    imports: "use gpui_material::{Icon, IconName, IconToggleButton};",
    code: `IconToggleButton::new(
    "favorite", "Favorite", Icon::new(IconName::Favorite), is_favorite,
)
.selected_icon(Icon::new(IconName::Favorite).filled(true))
.on_change(|selected, _, _| println!("Favorite: {selected}"))`,
  },
  {
    slug: "fab",
    group: "basic",
    title: "Fab",
    source: "crates/components/src/fab.rs",
    api: "struct.Fab.html",
    description: {
      en: "Place a prominent primary action above the content.",
      zh: "把主要操作悬浮在内容上方。",
    },
    usage: {
      en: "Supply an accessibility name and icon. Mount the FAB alongside the scroll viewport or put it in the Scaffold FAB slot to keep it visible while the content scrolls.",
      zh: "提供无障碍名称和图标。把 FAB 放在滚动区域旁或 Scaffold 的 FAB 槽位中，内容滚动时它仍然可见。",
    },
    imports: "use gpui_material::{Fab, Icon, IconName};",
    code: `Fab::new("create", "Create", Icon::new(IconName::Add))
    .on_activate(|_, _| println!("Create requested"))`,
  },
  {
    slug: "extended-fab",
    group: "basic",
    title: "ExtendedFab",
    source: "crates/components/src/fab.rs",
    api: "struct.ExtendedFab.html",
    description: {
      en: "Give a floating action a visible text label.",
      zh: "给悬浮操作加上可见的文字标签。",
    },
    usage: {
      en: "Provide the label and icon. Pass expanded(false) to collapse the visible label while preserving the accessibility name.",
      zh: "传入标签和图标。使用 expanded(false) 收起可见标签，同时保留无障碍名称。",
    },
    imports: "use gpui_material::{ExtendedFab, Icon, IconName};",
    code: `ExtendedFab::new("compose", "Compose", Icon::new(IconName::Edit))
    .expanded(true)
    .on_activate(|_, _| println!("Compose requested"))`,
  },
  {
    slug: "checkbox",
    group: "basic",
    title: "Checkbox",
    source: "crates/components/src/selection.rs",
    api: "struct.Checkbox.html",
    description: {
      en: "Select independent options and represent a mixed selection.",
      zh: "选择独立选项，也可以表示部分选中。",
    },
    usage: {
      en: "Pass the app-owned CheckboxState into new and apply the value received in on_change. For a parent checkbox, use Checkbox::aggregate to derive its value from the child states.",
      zh: "将应用持有的 CheckboxState 传给 new，并应用 on_change 给出的值。父复选框可以用 Checkbox::aggregate 从子项状态推导。",
    },
    imports: "use gpui_material::{Checkbox, CheckboxState};",
    code: `Checkbox::new("alerts", "Email alerts", CheckboxState::Unchecked)
    .on_change(|state, _, _| println!("New state: {state:?}"))`,
  },
  {
    slug: "radio",
    group: "basic",
    title: "Radio / RadioGroup",
    source: "crates/components/src/selection.rs",
    api: "struct.RadioGroup.html",
    description: {
      en: "Choose one option from a set.",
      zh: "从一组选项中选择一项。",
    },
    usage: {
      en: "Pass stable option IDs and the current selected ID to RadioGroup. Handle on_select in the app. The group provides keyboard navigation.",
      zh: "给 RadioGroup 传入稳定的选项 ID 和当前选中项 ID。在应用里处理 on_select。组内支持键盘导航。",
    },
    imports: "use gpui_material::{Radio, RadioGroup};",
    code: `RadioGroup::new("theme", "Theme")
    .options([Radio::new("light", "Light"), Radio::new("dark", "Dark")])
    .selected("light")
    .on_select(|id, _, _| println!("Selected: {id:?}"))`,
  },
  {
    slug: "switch",
    group: "basic",
    title: "Switch",
    source: "crates/components/src/selection.rs",
    api: "struct.Switch.html",
    description: {
      en: "Turn a preference on or off.",
      zh: "开启或关闭一项设置。",
    },
    usage: {
      en: "Pass the current boolean value into the constructor. Apply the new value from on_change, and optionally use checked_icon or unchecked_icon for the thumb.",
      zh: "在构造时传入当前布尔值，并在 on_change 中应用目标值。可使用 checked_icon 或 unchecked_icon 配置滑块图标。",
    },
    imports: "use gpui_material::Switch;",
    code: `Switch::new("sync", "Sync", is_sync_enabled)
    .on_change(|checked, _, _| println!("Sync: {checked}"))`,
  },
  {
    slug: "chip",
    group: "basic",
    title: "Chip",
    source: "crates/components/src/chip.rs",
    api: "struct.Chip.html",
    description: {
      en: "Represent a filter, suggestion, input, or small action.",
      zh: "表示筛选项、建议项、输入项或轻量操作。",
    },
    usage: {
      en: "Filter and Input chips can request a selection change or expose a separate removal action. Use on_activate for command-style chips.",
      zh: "Filter 和 Input 可以请求改变选中状态，也可以提供独立移除操作。执行命令的标签使用 on_activate。",
    },
    imports: "use gpui_material::{Chip, ChipVariant};",
    code: `Chip::new("unread", "Unread")
    .variant(ChipVariant::Filter)
    .selected(is_unread_only)
    .on_change(|selected, _, _| println!("Unread only: {selected}"))`,
  },
  {
    slug: "slider",
    group: "basic",
    title: "Slider",
    source: "crates/components/src/slider.rs",
    api: "struct.Slider.html",
    description: {
      en: "Adjust a value or range using the pointer or keyboard.",
      zh: "通过指针或键盘调整单值或范围。",
    },
    usage: {
      en: "Create a SliderState entity once and store it in the view. Read or subscribe to the entity for value changes. Use Slider::range with a range value for two thumbs.",
      zh: "创建一次 SliderState 实体并保存在视图中。读取或订阅实体获取值变化。双滑块使用 Slider::range 并给状态配置范围值。",
    },
    imports: "use gpui_material::{Slider, SliderState};",
    code: `// In the owning view's constructor
let level = cx.new(|_| SliderState::new().max(10.).step(1.).default_value(3.));

// In render
Slider::new("level", "Level", &self.level)`,
  },

  {
    slug: "text",
    group: "basic",
    title: "Text",
    source: "crates/components/src/text.rs",
    api: "struct.Text.html",
    description: {
      en: "Render text with a Material typography role.",
      zh: "按 Material 排版角色渲染文字。",
    },
    usage: {
      en: "Choose a TypeRole to inherit size, weight, and line height from the active theme. Use wrap or truncate to control narrow layouts.",
      zh: "选择 TypeRole，从当前主题继承字号、字重和行高。可用 wrap 或 truncate 处理窄布局。",
    },
    imports: "use gpui_material::{Text, theme::TypeRole};",
    code: `Text::new("Recent projects").role(TypeRole::HeadlineSmall).wrap()`,
  },
  {
    slug: "icon",
    group: "basic",
    title: "Icon",
    source: "crates/components/src/icon.rs",
    api: "struct.Icon.html",
    description: {
      en: "Render a bundled Material Symbol.",
      zh: "渲染库内置的 Material Symbol。",
    },
    usage: {
      en: "Use built-in IconName values for Material Symbols. Give a standalone icon a label when it conveys information. For icon buttons, set the accessible name on the button.",
      zh: "使用内置的 IconName 渲染 Material Symbols。独立图标承担信息表达时应添加标签。图标按钮的无障碍名称由按钮提供。",
    },
    imports: `use gpui::px;
use gpui_material::{Icon, IconName};`,
    code: `Icon::new(IconName::Favorite).filled(true).size(px(24.))`,
  },
  {
    slug: "surface",
    group: "layout",
    title: "Surface",
    source: "crates/foundation/src/surface.rs",
    api: "struct.Surface.html",
    description: {
      en: "Compose a themed background or a clickable, selectable, or checked surface.",
      zh: "构建主题背景，或可点击、可选及可勾选的表面。",
    },
    usage: {
      en: "Use new for a passive background or clickable, selectable, and toggleable for named controls. Keep selected and checked values in the caller. Set shape, border, content color, elevation, and feedback for each surface.",
      zh: "用 new 创建静态背景，或用 clickable、selectable、toggleable 创建带名称的控件。由调用方保存选中和勾选状态。可分别设置形状、描边、内容颜色、层级和反馈。",
    },
    imports: `use gpui::ParentElement as _;
use gpui_material::{Surface, Text, theme::ColorRole};`,
    code: `Surface::new(ColorRole::SurfaceContainerHigh)
    .child(Text::new("Project details"))`,
  },
  {
    slug: "card",
    group: "basic",
    title: "Card",
    source: "crates/components/src/card.rs",
    api: "struct.Card.html",
    description: {
      en: "Group related content on a Material surface.",
      zh: "把相关内容组织到同一个 Material 表面上。",
    },
    usage: {
      en: "Build the card from child elements. Choose Filled, Elevated, or Outlined. Add on_activate (and accessible_label) when the whole card opens a destination.",
      zh: "用子元素构成卡片，再选择 Filled、Elevated 或 Outlined。如果整张卡片用于打开目标，添加 on_activate 与 accessible_label。",
    },
    imports: `use gpui::ParentElement as _;
use gpui_material::{Card, CardVariant, Text};`,
    code: `Card::new("project")
    .variant(CardVariant::Outlined)
    .child(Text::new("Project Orion"))`,
  },
  {
    slug: "list",
    group: "advanced",
    title: "List / ListItem",
    source: "crates/components/src/list.rs",
    api: "struct.List.html",
    description: {
      en: "Present rows with labels, supporting text, and actions.",
      zh: "展示带标题、辅助文本和操作的列表行。",
    },
    usage: {
      en: "Build rows with unique, stable IDs. Use ListItem::on_activate for a row action, or group ListItem, SwitchTile, CheckboxTile, RadioTile, SliderTile, and ChoiceTile inside ListSection::segmented for M3E grouped corners and unified vertical roving focus.",
      zh: "给每行提供唯一且稳定的 ID。用 ListItem::on_activate 处理行操作，或在 ListSection::segmented 中混排 ListItem、SwitchTile、CheckboxTile、RadioTile、SliderTile 与 ChoiceTile 以获得 M3E 分组圆角和统一纵向漫游焦点。",
    },
    imports: "use gpui_material::{List, ListItem};",
    code: `List::new("projects").items([
    ListItem::new("orion", "Orion")
        .on_activate(|_, _| println!("Open Orion")),
])`,
  },
  {
    slug: "divider",
    group: "basic",
    title: "Divider",
    source: "crates/components/src/display.rs",
    api: "struct.Divider.html",
    description: {
      en: "Separate adjacent regions or list items.",
      zh: "分隔相邻区域或列表项。",
    },
    usage: {
      en: "Pick horizontal or vertical according to the parent layout. Give a vertical divider a parent with a bounded height.",
      zh: "根据父布局选择横向或纵向。垂直分隔线的父布局应有确定高度。",
    },
    imports: "use gpui_material::Divider;",
    code: `Divider::horizontal()`,
  },
  {
    slug: "badge",
    group: "basic",
    title: "Badge",
    source: "crates/components/src/display.rs",
    api: "struct.Badge.html",
    description: {
      en: "Add a dot, short label, or count to a host element.",
      zh: "给宿主元素添加圆点、短标签或计数。",
    },
    usage: {
      en: "Wrap the host element with dot, label, or count. count takes the maximum number to display before it switches to a capped label such as 99+.",
      zh: "用 dot、label 或 count 包裹宿主元素。count 接收显示上限，超过时会显示类似 99+ 的简写。",
    },
    imports: "use gpui_material::{Badge, Icon, IconName};",
    code: `Badge::count(Icon::new(IconName::Notifications), 3, 99)`,
  },
  {
    slug: "progress",
    group: "basic",
    title: "LinearProgress / CircularProgress",
    source: "crates/components/src/progress.rs",
    api: "struct.LinearProgress.html",
    description: {
      en: "Show a measured fraction or an indeterminate operation.",
      zh: "显示确定进度或不确定操作。",
    },
    usage: {
      en: "Pass a fraction from 0.0 to 1.0 with value for determinate progress. Omit value for an indeterminate operation. Both constructors take an accessibility label.",
      zh: "确定进度用 value 传入 0.0 至 1.0 的比例，不确定进度省略 value。两种构造器都接收无障碍标签。",
    },
    imports: "use gpui_material::{CircularProgress, LinearProgress};",
    code: `let upload = LinearProgress::new("upload", "Upload").value(0.65);
let refresh = CircularProgress::new("refresh", "Refreshing");`,
  },
  {
    slug: "loading-indicator",
    group: "basic",
    title: "LoadingIndicator",
    source: "crates/components/src/progress.rs",
    api: "struct.LoadingIndicator.html",
    description: {
      en: "Indicate activity while a result is pending.",
      zh: "等待结果时显示加载状态。",
    },
    usage: {
      en: "Provide an accessibility label that describes the current operation. Use contained(true) to draw the indicator inside a themed container.",
      zh: "提供描述当前操作的无障碍标签。用 contained(true) 将指示器画在有主题背景的容器中。",
    },
    imports: "use gpui_material::LoadingIndicator;",
    code: `LoadingIndicator::new("fetching", "Fetching projects").contained(true)`,
  },
  {
    slug: "snackbar",
    group: "layout",
    title: "Snackbar / SnackbarHost",
    source: "crates/components/src/overlay/snackbar.rs",
    api: "struct.SnackbarHost.html",
    description: {
      en: "Queue transient feedback and optional follow-up actions.",
      zh: "排队显示短时反馈和可选的后续操作。",
    },
    usage: {
      en: "Keep one SnackbarHost entity in the view. Mount it in the Scaffold snackbar slot and call push from a command to show feedback. A timeout of None keeps a message visible until dismissal.",
      zh: "在视图中保留一个 SnackbarHost 实体，将它放进 Scaffold 的 snackbar 槽位，再从命令里调用 push。把超时设为 None 可以持续显示，直到关闭。",
    },
    imports: "use gpui_material::{Scaffold, Snackbar, SnackbarHost};",
    code: `// In the view constructor
let host = cx.new(|_| SnackbarHost::default());

// When an operation completes
self.host.update(cx, |host, cx| {
    host.push(Snackbar::new("Project saved"), window, cx);
});

// In render
Scaffold::new("project", content).snackbar(self.host.clone())`,
  },

  {
    slug: "text-field",
    group: "form",
    title: "TextField",
    source: "crates/components/src/input/text_field.rs",
    api: "struct.TextField.html",
    description: {
      en: "Edit single-line, multiline, or masked text.",
      zh: "编辑单行、多行或带掩码的文本字段。",
    },
    usage: {
      en: "Create and retain a TextFieldState entity. Subscribe to TextFieldEvent on it to react to edits, use clearable(true) for a built-in trailing clear button, and use validate or TextFieldMaskPattern on the state for filtering and structured formatting. Use TextAreaState with TextField::multiline, and a dedicated masked state with TextField::secure and visibility_toggle or TextFieldObfuscationMode.",
      zh: "创建并保留 TextFieldState 实体，通过订阅 TextFieldEvent 响应编辑，使用 clearable(true) 自动添加尾部清除按钮，并在状态上使用 validate 或 TextFieldMaskPattern 进行过滤和结构化格式化。多行输入使用 TextAreaState 和 TextField::multiline，密码输入使用独立的掩码状态搭配 TextField::secure 与 visibility_toggle 或 TextFieldObfuscationMode。",
    },
    imports: "use gpui_material::{TextField, TextFieldState, TextFieldVariant};",
    code: `// In the owning view's constructor
let email = cx.new(|cx| TextFieldState::new(window, cx).placeholder("name@example.com"));

// In render
TextField::new("email", "Email", &self.email)
    .variant(TextFieldVariant::Outlined)`,
  },
  {
    slug: "exposed-dropdown",
    group: "form",
    title: "ExposedDropdown",
    source: "crates/components/src/input/exposed_dropdown.rs",
    api: "struct.ExposedDropdown.html",
    description: {
      en: "Choose a value from a text-field anchored list.",
      zh: "从文本字段锚定的下拉列表中选择值。",
    },
    usage: {
      en: "Create one ExposedDropdownState per field. Pass true for editable filtering or false for selection only. Give each option a stable ID and handle on_select in the application.",
      zh: "给每个字段创建一个 ExposedDropdownState。传入 true 允许输入筛选，false 仅允许选择。为每个选项设置稳定的 ID，并在应用里处理 on_select。",
    },
    imports: "use gpui_material::{DropdownOption, ExposedDropdown, ExposedDropdownState};",
    code: `let language = cx.new(|cx| ExposedDropdownState::new("English", false, window, cx));

// In render
ExposedDropdown::new("language", "Language", &self.language, [
    DropdownOption::new("en", "English"),
    DropdownOption::new("zh", "简体中文"),
])`,
  },
  {
    slug: "menu",
    group: "advanced",
    title: "Menu",
    source: "crates/components/src/overlay/menu.rs",
    api: "struct.Menu.html",
    description: {
      en: "Open an anchored list of commands or a context menu.",
      zh: "打开锚定命令列表或上下文菜单。",
    },
    usage: {
      en: "Supply a focusable trigger and a list of MenuItem values with stable IDs. Use context() to open on secondary click. Bind each command with on_activate.",
      zh: "提供可聚焦的触发控件和带稳定 ID 的 MenuItem 列表。使用 context() 响应右键打开，用 on_activate 绑定各项命令。",
    },
    imports: "use gpui_material::{Button, Menu, MenuItem};",
    code: `Menu::new("options", Button::new("open-options", "Options"), [
    MenuItem::new("rename", "Rename").on_activate(|_, _| println!("Rename")),
])`,
  },
  {
    slug: "tooltip",
    group: "basic",
    title: "Tooltip",
    source: "crates/components/src/overlay/tooltip.rs",
    api: "struct.Tooltip.html",
    description: {
      en: "Attach a short hint or richer help to a control.",
      zh: "给控件添加简短提示或更丰富的帮助内容。",
    },
    usage: {
      en: "Wrap the trigger with Tooltip::plain for a text hint. For keyboard-triggered help, share a retained FocusHandle between the trigger and Tooltip::track_focus.",
      zh: "用 Tooltip::plain 包裹触发控件，显示文本提示。键盘提示需要让触发控件和 Tooltip::track_focus 共用一个持久化的 FocusHandle。",
    },
    imports: "use gpui_material::{Button, Tooltip};",
    code: `Tooltip::plain(
    "archive-help",
    Button::new("archive", "Archive").track_focus(&self.archive_focus),
    "Move this project out of the active list",
)
.track_focus(&self.archive_focus)`,
  },
  {
    slug: "dialog",
    group: "layout",
    title: "Dialog",
    source: "crates/components/src/overlay/dialog.rs",
    api: "struct.Dialog.html",
    description: {
      en: "Ask for a decision or accept input in a modal.",
      zh: "通过模态对话框请求确认或接收输入。",
    },
    usage: {
      en: "Retain a DialogState entity and mount the dialog alongside its trigger. Open the entity from the trigger callback. Return true from confirm to dismiss the dialog.",
      zh: "保留 DialogState 实体，将对话框与触发控件一起渲染。在触发回调中打开实体。确认回调返回 true 时关闭对话框。",
    },
    imports: "use gpui_material::{Button, Dialog, DialogState};",
    code: `let dialog = cx.new(DialogState::new);

// In render
let state = self.dialog.clone();
let trigger = Button::new("open", "Delete project").on_activate(move |window, cx| {
    state.update(cx, |state, cx| state.open(window, cx));
});
let dialog = Dialog::new("delete", &self.dialog, "Delete project?")
    .description("You can restore it from the archive.")
    .confirm("Delete", |_, _| true);`,
  },
  {
    slug: "sheet",
    group: "layout",
    title: "Sheet",
    source: "crates/components/src/overlay/sheet.rs",
    api: "struct.Sheet.html",
    story: "sheets",
    description: {
      en: "Reveal supporting content at the bottom or one side.",
      zh: "从底部或侧边展开补充内容。",
    },
    usage: {
      en: "Share a retained DialogState with the trigger. Select Bottom, Start, or End with edge, and choose Modal when the sheet should block background interaction.",
      zh: "与触发控件共用持久化的 DialogState。用 edge 选 Bottom、Start 或 End，需要阻止背景交互时用 Modal。",
    },
    imports: "use gpui_material::{Sheet, SheetEdge, SheetMode, Text};",
    code: `Sheet::new("filters", &self.filters, "Filters", Text::new("Project filters"))
    .edge(SheetEdge::End)
    .mode(SheetMode::Modal)`,
  },

  {
    slug: "app-bar",
    group: "layout",
    title: "AppBar",
    source: "crates/components/src/navigation/app_bar.rs",
    api: "struct.AppBar.html",
    description: {
      en: "Place a title, navigation, and commands above the content.",
      zh: "在内容上方布置标题、导航和命令。",
    },
    usage: {
      en: "Choose a variant and supply optional navigation and action elements. For a flexible app bar, pass the same ScrollHandle used by the content viewport.",
      zh: "选择变体，再按需传入导航和操作元素。Flexible 应用栏需传入与内容滚动区共用的 ScrollHandle。",
    },
    imports: "use gpui_material::{AppBar, AppBarVariant};",
    code: `AppBar::new("projects-header", "Projects")
    .variant(AppBarVariant::MediumFlexible)
    .track_scroll(&self.scroll)`,
  },
  {
    slug: "tabs",
    group: "advanced",
    title: "Tabs",
    source: "crates/components/src/navigation/items.rs",
    api: "struct.Tabs.html",
    description: {
      en: "Switch between peer content panels.",
      zh: "在同级内容面板之间切换。",
    },
    usage: {
      en: "Build NavigationItems with stable IDs and the current selection. Use on_select to update the active panel. Enable scrollable when the tabs may overflow.",
      zh: "给 NavigationItems 传入稳定的 ID 和当前选中项。在 on_select 中切换面板，标签可能溢出时启用 scrollable。",
    },
    imports: "use gpui_material::{NavigationItem, NavigationItems, Tabs};",
    code: `Tabs::new(NavigationItems::new("tabs", "Project views", [
    NavigationItem::new("recent", "Recent"),
    NavigationItem::new("shared", "Shared"),
]).selected("recent"))`,
  },
  {
    slug: "navigation-bar",
    group: "layout",
    title: "NavigationBar",
    source: "crates/components/src/navigation/items.rs",
    api: "struct.NavigationBar.html",
    description: {
      en: "Switch between primary destinations in a short bar.",
      zh: "通过短导航栏切换主要目的地。",
    },
    usage: {
      en: "Pass the current destination through NavigationItems and handle on_select in the application. Place the bar in Scaffold::bottom_bar when composing a window.",
      zh: "通过 NavigationItems 传入当前目的地，并在应用中处理 on_select。组合窗口时可把导航栏放进 Scaffold::bottom_bar。",
    },
    imports: "use gpui_material::{Icon, IconName, NavigationBar, NavigationItem, NavigationItems};",
    code: `NavigationBar::new(NavigationItems::new("destinations", "Destinations", [
    NavigationItem::new("inbox", "Inbox").icon(Icon::new(IconName::Inbox)),
    NavigationItem::new("archive", "Archive").icon(Icon::new(IconName::Archive)),
]).selected("inbox"))`,
  },
  {
    slug: "navigation-rail",
    group: "layout",
    title: "NavigationRail",
    source: "crates/components/src/navigation/rail.rs",
    api: "struct.NavigationRail.html",
    description: {
      en: "Place primary destinations in a side rail.",
      zh: "在侧边轨道上布置主要目的地。",
    },
    usage: {
      en: "Build the rail from NavigationItems. Bind mode to your window layout, and pass a DialogState to modal_state to show it as a modal rail.",
      zh: "用 NavigationItems 构造导航轨道。根据窗口布局设置 mode，向 modal_state 传入 DialogState 可将轨道用作模态导航。",
    },
    imports: "use gpui_material::{NavigationItem, NavigationItems, NavigationRail, RailMode};",
    code: `NavigationRail::new("rail", NavigationItems::new("sections", "Sections", [
    NavigationItem::new("inbox", "Inbox"),
    NavigationItem::new("archive", "Archive"),
]).selected("inbox")).mode(RailMode::Expanded)`,
  },
  {
    slug: "scaffold",
    group: "layout",
    title: "Scaffold",
    source: "crates/components/src/navigation/scaffold.rs",
    api: "struct.Scaffold.html",
    story: "adaptive-layout",
    description: {
      en: "Compose a window from app chrome, content, and feedback.",
      zh: "组合窗口界面、内容和操作反馈。",
    },
    usage: {
      en: "Place the header, rail, bottom bar, FAB, and snackbar in their named slots. Give the scaffold a bounded size and scroll the content within its viewport.",
      zh: "将应用栏、轨道、底部栏、FAB 和消息条放入各自槽位。给 Scaffold 一个确定的尺寸，在内容区域内实现滚动。",
    },
    imports: "use gpui_material::{AppBar, Scaffold, Text};",
    code: `Scaffold::new("workspace", Text::new("Recent projects"))
    .app_bar(AppBar::new("header", "Workspace"))`,
  },
  {
    slug: "vertical-drag-handle",
    group: "layout",
    title: "VerticalDragHandle",
    source: "crates/components/src/drag_handle.rs",
    api: "struct.VerticalDragHandle.html",
    description: {
      en: "Resize two adjacent panes using a vertical handle.",
      zh: "用竖向拖柄调整相邻窗格的宽度。",
    },
    usage: {
      en: "Place the handle between the panes. Supply the current start pane width, its permitted range, and an accessible label. Apply accepted resize requests in your view. Use sizes, colors, and shapes to customize the resting, pressed, and dragged appearances.",
      zh: "将拖柄放在窗格之间，传入起始侧窗格的当前宽度、允许范围和无障碍名称。由视图应用接受的宽度请求。用 sizes、colors 和 shapes 定制静止、按下和拖动外观。",
    },
    imports: "use gpui::px;\nuse gpui_material::VerticalDragHandle;",
    code: `let view = cx.entity();
VerticalDragHandle::new("pane-divider").on_resize(
    "Start pane width",
    self.pane_width,
    px(80.)..=px(360.),
    move |width, _, cx| {
        view.update(cx, |view, cx| {
            view.pane_width = width;
            cx.notify();
        });
    },
)`,
  },
  {
    slug: "adaptive-layout",
    group: "layout",
    title: "AdaptiveNavigation",
    source: "crates/components/src/navigation/adaptive.rs",
    api: "struct.AdaptiveNavigation.html",
    description: {
      en: "Choose a navigation bar or rail based on available width.",
      zh: "根据可用宽度选择导航栏或导航轨道。",
    },
    usage: {
      en: "Create the scaffold and supply an application-controlled NavigationItems model. Pass the available width in GPUI pixels on each render.",
      zh: "先创建 Scaffold，再提供应用控制的 NavigationItems。每次渲染都传入以 GPUI 像素表示的可用宽度。",
    },
    imports: "use gpui_material::{AdaptiveNavigation, NavigationItem, NavigationItems, Scaffold, Text};",
    code: `let items = NavigationItems::new("sections", "Sections", [
    NavigationItem::new("inbox", "Inbox"),
    NavigationItem::new("archive", "Archive"),
]).selected("inbox");

AdaptiveNavigation::new(Scaffold::new("mail", Text::new("Inbox")), items, width)`,
  },
  {
    slug: "list-detail",
    group: "layout",
    title: "ListDetail",
    source: "crates/components/src/navigation/adaptive.rs",
    api: "struct.ListDetail.html",
    story: "adaptive-layout",
    description: {
      en: "Show a list and its selected detail in a responsive layout.",
      zh: "响应式展示列表及选中项详情。",
    },
    usage: {
      en: "Pass the list, detail, and available width. Keep the selected item in your application and use active_pane to choose the visible pane at narrow widths.",
      zh: "传入列表、详情和可用宽度。应用持有选中项，并通过 active_pane 指定较窄时显示哪个面板。",
    },
    imports: "use gpui_material::{ListDetail, ListDetailPane, Text};",
    code: `ListDetail::new("projects", width, Text::new("Projects"), Text::new("Orion"))
    .active_pane(ListDetailPane::Detail)`,
  },
  {
    slug: "supporting-pane",
    group: "layout",
    title: "SupportingPane",
    source: "crates/components/src/navigation/adaptive.rs",
    api: "struct.SupportingPane.html",
    story: "adaptive-layout",
    description: {
      en: "Place supplementary content next to the primary view.",
      zh: "在主视图旁显示辅助内容。",
    },
    usage: {
      en: "Supply the primary element, supporting element, and available width. Use show_support to control whether the supporting view is shown.",
      zh: "传入主元素、辅助元素和可用宽度。通过 show_support 控制辅助视图是否显示。",
    },
    imports: "use gpui_material::{SupportingPane, Text};",
    code: `SupportingPane::new("editor", width, Text::new("Document"), Text::new("Outline"))
    .show_support(true)`,
  },
  {
    slug: "feed",
    group: "layout",
    title: "Feed",
    source: "crates/components/src/navigation/adaptive.rs",
    api: "struct.Feed.html",
    story: "adaptive-layout",
    description: {
      en: "Lay out a collection of cards in a responsive feed.",
      zh: "用响应式信息流排列多张卡片。",
    },
    usage: {
      en: "Retain a GPUI ScrollHandle and pass the current available width to Feed. Add cards as child elements.",
      zh: "持久化保留一个 GPUI ScrollHandle，并将当前可用宽度传给 Feed。以子元素形式添加卡片。",
    },
    imports: `use gpui::ParentElement as _;
use gpui_material::{Card, Feed, Text};`,
    code: `Feed::new("projects-feed", width, &self.scroll)
    .child(Card::new("orion").child(Text::new("Orion")))
    .child(Card::new("lyra").child(Text::new("Lyra")))`,
  },

  {
    slug: "button-group",
    group: "basic",
    title: "ButtonGroup",
    source: "crates/extras/src/groups.rs",
    feature: "expressive-groups",
    description: {
      en: "Group commands or connected selections.",
      zh: "组合一组命令或相连的选择按钮。",
    },
    usage: {
      en: "Use ButtonGroupItem for each member. For connected single-choice rows, use ButtonGroup::connected with single_select or single_optional_select. For multi-selection, use selection and on_change.",
      zh: "为每个成员构造 ButtonGroupItem。相连的单选按钮组可直接使用 ButtonGroup::connected 搭配 single_select 或 single_optional_select；多选场景使用 selection 与 on_change。",
    },
    imports: "use gpui_material::{ButtonGroup, ButtonGroupItem};",
    code: `ButtonGroup::new("alignment", "Alignment", [
    ButtonGroupItem::new("left", "Left").on_activate(|_, _| println!("Left")),
    ButtonGroupItem::new("right", "Right").on_activate(|_, _| println!("Right")),
])`,
  },
  {
    slug: "split-button",
    group: "basic",
    title: "SplitButton",
    source: "crates/extras/src/split_button.rs",
    feature: "expressive-groups",
    description: {
      en: "Expose a primary command next to a menu of related commands.",
      zh: "把主要命令与相关命令的菜单放在一起。",
    },
    usage: {
      en: "Provide separate labels for the primary action and the menu. Bind the primary command with on_activate, and each secondary command on its MenuItem.",
      zh: "分别提供主操作和菜单的标签。用 on_activate 绑定主要命令，并在各 MenuItem 上绑定次要命令。",
    },
    imports: "use gpui_material::{MenuItem, SplitButton};",
    code: `SplitButton::new("export", "Export PDF", "More formats", [
    MenuItem::new("csv", "Export CSV").on_activate(|_, _| println!("CSV")),
])
.on_activate(|_, _| println!("PDF"))`,
  },
  {
    slug: "fab-menu",
    group: "advanced",
    title: "FabMenu",
    source: "crates/extras/src/toolbar.rs",
    feature: "toolbars",
    description: {
      en: "Expand a floating action into a set of related commands.",
      zh: "从悬浮按钮展开一组相关命令。",
    },
    usage: {
      en: "Pass the trigger label and icon, then add stable FabMenuItem IDs. Each item can have its own on_activate callback.",
      zh: "传入触发按钮的标签和图标，再添加带稳定 ID 的 FabMenuItem。每个菜单项可以有自己的 on_activate 回调。",
    },
    imports: "use gpui_material::{FabMenu, FabMenuItem, IconName};",
    code: `FabMenu::new("create-menu", "Create", IconName::Add, [
    FabMenuItem::new("note", "New note", IconName::Edit)
        .on_activate(|_, _| println!("New note")),
])`,
  },
  {
    slug: "toolbar",
    group: "layout",
    title: "Toolbar",
    source: "crates/extras/src/toolbar/bar.rs",
    feature: "toolbars",
    story: "toolbars",
    description: {
      en: "Arrange related actions in a floating or docked toolbar.",
      zh: "在悬浮或停靠的工具栏中排列相关操作。",
    },
    usage: {
      en: "Use ToolbarAction for labeled icon commands. Choose placement and orientation on the toolbar. Secondary actions can move to overflow when it collapses.",
      zh: "用 ToolbarAction 构造带标签的图标命令，并设置工具栏的放置位置和方向。工具栏收起时，次要操作可以进入溢出菜单。",
    },
    imports: "use gpui_material::{IconName, Toolbar, ToolbarAction};",
    code: `Toolbar::new("document-tools", "Document tools", [
    ToolbarAction::new("edit", "Edit", IconName::Edit)
        .on_activate(|_, _| println!("Edit")),
])`,
  },
  {
    slug: "search",
    group: "advanced",
    title: "SearchBar / SearchView",
    source: "crates/extras/src/search.rs",
    feature: "search",
    description: {
      en: "Search from a collapsed trigger or an expanded surface.",
      zh: "通过收起的入口或展开的界面进行搜索。",
    },
    usage: {
      en: "Retain a SearchState entity. Use SearchBar for the collapsed trigger or SearchView for an expanded surface. Subscribe to SearchEvent on the state and supply results with stable IDs.",
      zh: "持久化保留 SearchState 实体。用 SearchBar 作为收起时的入口，或用 SearchView 渲染展开的界面。订阅状态上的 SearchEvent，并提供带稳定 ID 的结果。",
    },
    imports: "use gpui_material::{SearchBar, SearchState};",
    code: `// In the view constructor
let search = cx.new(|cx| SearchState::new(window, cx));

// In render
SearchBar::new("search", "Search projects", &self.search)`,
  },
  {
    slug: "date-picker",
    group: "form",
    title: "DatePicker",
    source: "crates/extras/src/date_time/date.rs",
    feature: "date-time",
    relatedStory: { id: "date-range-picker", title: { en: "Date range", zh: "日期范围" } },
    description: {
      en: "Choose a date or an inclusive date range.",
      zh: "选择一个日期或包含起止两日的日期范围。",
    },
    usage: {
      en: "Retain a DatePickerState entity for the picker interaction. Pass DateSelection::Single or DateSelection::Range from application state and commit the value in on_confirm.",
      zh: "为选择器的交互保留 DatePickerState 实体。应用传入 DateSelection::Single 或 DateSelection::Range，并在 on_confirm 中保存结果。",
    },
    imports: "use gpui_material::date_time::{DatePicker, DatePickerState, DateSelection};",
    code: `// In the view constructor
let date = cx.new(|cx| DatePickerState::new(window, cx));

// In render
DatePicker::new("due-date", "Due date", &self.date, DateSelection::Single(None))
    .on_confirm(|value, _, _| println!("Selected: {value:?}"))`,
  },
  {
    slug: "time-picker",
    group: "form",
    title: "TimePicker",
    source: "crates/extras/src/date_time/time.rs",
    feature: "date-time",
    description: {
      en: "Choose a time using a dial or text input.",
      zh: "通过表盘或文本输入选择时间。",
    },
    usage: {
      en: "Retain a TimePickerState entity and pass the current time from the application. Apply the value received by on_confirm.",
      zh: "保留一个 TimePickerState 实体，由应用传入当前时间，并在 on_confirm 中应用确认后的值。",
    },
    imports: "use gpui_material::date_time::{TimePicker, TimePickerState};",
    code: `// In the view constructor
let picker = cx.new(|cx| TimePickerState::new(window, cx));

// In render
TimePicker::new("reminder", "Reminder time", &self.picker, self.time)
    .on_confirm(|time, _, _| println!("Reminder: {time}"))`,
  },
  {
    slug: "carousel",
    group: "advanced",
    title: "Carousel",
    source: "crates/extras/src/carousel.rs",
    feature: "carousel",
    description: {
      en: "Browse a collection using one of six Material layouts.",
      zh: "用六种 Material 布局之一浏览集合。",
    },
    usage: {
      en: "Give each CarouselItem a stable ID, label, and content. Choose the variant to control how much adjacent content is visible.",
      zh: "给每个 CarouselItem 提供稳定 ID、标签和内容。通过 variant 调整相邻内容的可见范围。",
    },
    imports: "use gpui_material::{Carousel, CarouselItem, CarouselVariant, Text};",
    code: `Carousel::new("featured", "Featured projects", [
    CarouselItem::new("orion", "Orion", Text::new("Project Orion")),
    CarouselItem::new("lyra", "Lyra", Text::new("Project Lyra")),
])
.variant(CarouselVariant::MultiBrowse)`,
  },
  {
    slug: "avatar",
    group: "basic",
    title: "Avatar / AvatarGroup",
    source: "crates/components/src/avatar.rs",
    api: "struct.Avatar.html",
    description: {
      en: "Represent a person or entity with initials, an icon, an image, or custom content, or stack members in an AvatarGroup.",
      zh: "用首字母、图标、图片或自定义内容展示用户或实体头像，或通过 AvatarGroup 展示重叠头像堆叠。",
    },
    usage: {
      en: "Choose Avatar::initials, Avatar::icon, Avatar::image, or Avatar::custom. Configure size with AvatarSize and shape with AvatarShape. Use AvatarGroup::new with max_visible, overlap, and ring_width to stack avatars with a +N overflow indicator.",
      zh: "按需选用 Avatar::initials、Avatar::icon、Avatar::image 或 Avatar::custom。通过 AvatarSize 与 AvatarShape 调整尺寸和形状；使用 AvatarGroup::new 搭配 max_visible、overlap 和 ring_width 构建带 +N 溢出计数的重叠堆叠。",
    },
    imports: "use gpui_material::{Avatar, AvatarGroup, AvatarSize};",
    code: `AvatarGroup::new("team", [
    Avatar::initials("Ada Lovelace"),
    Avatar::initials("Grace Hopper"),
    Avatar::initials("Alan Turing"),
])
.size(AvatarSize::Medium)
.max_visible(2)`,
  },
  {
    slug: "empty-state",
    group: "layout",
    title: "EmptyView",
    source: "crates/components/src/empty_view.rs",
    api: "struct.EmptyView.html",
    description: {
      en: "Present a centered Material 3 Expressive empty-state layout with an optional shape backdrop, copy, and actions.",
      zh: "呈现居中的 Material 3 Expressive 空状态布局，包含可选形状衬底、说明文案与操作按钮。",
    },
    usage: {
      en: "Pass a stable ID and title to EmptyView::new. Attach icon (rendered inside a decorative MaterialShape Surface backdrop) or visual, add supporting text, and supply primary_action / secondary_action buttons.",
      zh: "向 EmptyView::new 传入稳定 ID 与标题。可配置带装饰性 MaterialShape 衬底的 icon 或自定义 visual，补充 supporting 文案，并提供 primary_action / secondary_action 操作按钮。",
    },
    imports: "use gpui_material::{Button, EmptyView, Icon, IconName};",
    code: `EmptyView::new("no-projects", "No projects yet")
    .icon(Icon::new(IconName::Inbox))
    .supporting("Create your first project to start organizing files.")
    .primary_action(Button::new("create-project", "Create project"))`,
  },
  {
    slug: "drop-target",
    group: "form",
    title: "DropTarget",
    source: "crates/components/src/drop_target.rs",
    api: "struct.DropTarget.html",
    description: {
      en: "Accept external OS files or typed in-app drag payloads with Material drop feedback.",
      zh: "接收外部系统文件或应用内强类型拖拽载荷，并提供 Material 拖放反馈。",
    },
    usage: {
      en: "Use DropTarget::files for gpui::ExternalPaths or DropTarget::new::<T> for typed in-app payloads. Supply can_drop to validate hovered payloads and on_drop to handle accepted drops.",
      zh: "外部文件拖放使用 DropTarget::files（gpui::ExternalPaths），应用内载荷使用 DropTarget::new::<T>。通过 can_drop 校验悬停载荷，并在 on_drop 中处理接收结果。",
    },
    imports: "use gpui_material::DropTarget;",
    code: `DropTarget::files("upload-zone")
    .title("Drop files here")
    .supporting("PNG, SVG, or JSON")
    .on_drop(|paths, _, _| println!("Dropped {} files", paths.paths().len()))`,
  },
  {
    slug: "switch-tile",
    group: "form",
    title: "SwitchTile",
    source: "crates/components/src/settings_tiles.rs",
    api: "struct.SwitchTile.html",
    story: "list",
    description: {
      en: "Combine a settings list row and a synchronized Switch into a single focus stop.",
      zh: "将设置列表行与同步开关封装为单一焦点的快捷设置项。",
    },
    usage: {
      en: "Pass the controlled boolean state to new and update your application state inside on_change. The entire row activates with pointer, keyboard, or accessibility actions without double-toggling.",
      zh: "将受控布尔状态传给 new，并在 on_change 中更新应用状态。整行支持指针、键盘和无障碍激活且不会重复触发。",
    },
    imports: "use gpui_material::SwitchTile;",
    code: `SwitchTile::new("auto-save", "Auto-save", auto_save)
    .supporting("Save documents automatically while editing")
    .on_change(|enabled, _, _| println!("Auto-save: {enabled}"))`,
  },
  {
    slug: "choice-tile",
    group: "form",
    title: "ChoiceTile",
    source: "crates/components/src/settings_tiles.rs",
    api: "struct.ChoiceTile.html",
    story: "list",
    description: {
      en: "Display the active setting value in a list row and choose from an anchored Material menu.",
      zh: "在设置列表行展示当前选中值，并通过锚定 Material 菜单切换选项。",
    },
    usage: {
      en: "Supply stable ChoiceOption entries and the currently selected option ID. Apply the chosen ID in on_select; focus returns to the row when the menu closes.",
      zh: "提供带稳定 ID 的 ChoiceOption 列表与当前选中 ID，在 on_select 中应用新值；菜单关闭后焦点自动回到设置行。",
    },
    imports: "use gpui_material::{ChoiceOption, ChoiceTile};",
    code: `ChoiceTile::new(
    "density",
    "Density",
    [
        ChoiceOption::new("compact", "Compact"),
        ChoiceOption::new("standard", "Standard"),
        ChoiceOption::new("comfortable", "Comfortable"),
    ],
    selected_density,
)
.on_select(|id, _, _| println!("Density: {id}"))`,
  },
  {
    slug: "bottom-app-bar",
    group: "layout",
    title: "BottomAppBar",
    source: "crates/components/src/navigation/bottom_app_bar.rs",
    api: "struct.BottomAppBar.html",
    description: {
      en: "Place secondary icon actions and an optional trailing FAB in a bottom bar.",
      zh: "在底部栏中排列次要图标操作与可选的尾侧悬浮操作按钮。",
    },
    usage: {
      en: "Mount BottomAppBar directly or inside Scaffold::bottom_bar. Add leading IconButton actions with action/actions and an optional trailing Fab with fab. Attach AppBarScrollState to hide on scroll.",
      zh: "可直接放置或传入 Scaffold::bottom_bar。用 action/actions 添加起始侧图标按钮，用 fab 放置尾侧悬浮按钮，并可绑定 AppBarScrollState 实现滚动隐藏。",
    },
    imports: "use gpui_material::{BottomAppBar, Fab, Icon, IconButton, IconName};",
    code: `BottomAppBar::new("editor-bottom-bar")
    .action(IconButton::new("search", "Search", Icon::new(IconName::Search)))
    .action(IconButton::new("archive", "Archive", Icon::new(IconName::Archive)))
    .fab(Fab::new("add", "Add", Icon::new(IconName::Add)))`,
  },
  {
    slug: "navigation-drawer",
    group: "layout",
    title: "NavigationDrawer",
    source: "crates/components/src/navigation/drawer.rs",
    api: "struct.NavigationDrawer.html",
    description: {
      en: "Present top-level destinations in a permanent standard panel or a modal drawer.",
      zh: "在常驻标准面板或模态抽屉中呈现顶层导航目的地。",
    },
    usage: {
      en: "Build destinations with NavigationItems, sections, headers, and dividers. Use NavigationDrawerMode::Standard for a docked 360 dp drawer or Modal with a retained DialogState for scrim and focus restoration.",
      zh: "使用 NavigationItems、分组、标题和分隔线组织导航项。Standard 模式呈现 360 dp 停靠抽屉，Modal 模式搭配持久化的 DialogState 提供遮罩与回焦。",
    },
    imports: "use gpui_material::{Icon, IconName, NavigationDrawer, NavigationItem, NavigationItems};",
    code: `NavigationDrawer::new(
    "main-drawer",
    NavigationItems::new("destinations", "Destinations", [
        NavigationItem::new("inbox", "Inbox").icon(Icon::new(IconName::Inbox)),
        NavigationItem::new("archive", "Archive").icon(Icon::new(IconName::Archive)),
    ])
    .selected("inbox"),
)
.headline("Workspace")`,
  },
  {
    slug: "stepper",
    group: "layout",
    title: "Stepper",
    source: "crates/components/src/navigation/stepper.rs",
    api: "struct.Stepper.html",
    description: {
      en: "Guide users through a multi-step horizontal or vertical workflow.",
      zh: "引导用户完成横向或纵向的多步骤流程。",
    },
    usage: {
      en: "Supply ordered StepItem entries and set active_step for automatic completed, active, and upcoming status derivation. Enable non_linear to allow jumping directly to any enabled step.",
      zh: "传入有序的 StepItem 列表并用 active_step 自动推导已完成、当前和待办状态；开启 non_linear 可直接跳转至任意启用步骤。",
    },
    imports: "use gpui_material::{StepItem, Stepper};",
    code: `Stepper::new(
    "setup-steps",
    [
        StepItem::new("account", "Account"),
        StepItem::new("profile", "Profile"),
        StepItem::new("confirm", "Confirm"),
    ],
)
.active_step(1)
.on_select(|index, _, _| println!("Step {index}"))`,
  },
  {
    slug: "banner",
    group: "layout",
    title: "Banner",
    source: "crates/components/src/banner.rs",
    api: "struct.Banner.html",
    description: {
      en: "Display an in-flow Material 3 banner or callout with semantic tones, actions, and optional dismissal.",
      zh: "展示支持语义色板、操作按钮与可选关闭按钮的流式 Material 3 横幅提示。",
    },
    usage: {
      en: "Keep visibility in application state and pass a stable ID and message to Banner::new. Set tone for semantic palettes, add up to two actions, and attach on_dismiss for a trailing close button.",
      zh: "由应用持有显示状态，向 Banner::new 传入稳定 ID 与正文。通过 tone 选用语义色板，添加最多两个操作按钮，并可用 on_dismiss 附加尾部关闭按钮。",
    },
    imports: "use gpui_material::{Banner, Icon, IconName, SemanticTone};",
    code: `Banner::new("sync-banner", "Workspace sync paused while offline.")
    .title("Offline mode")
    .tone(SemanticTone::Warning)
    .icon(Icon::new(IconName::Info))
    .action("Retry", |_, _| println!("Retry sync"))`,
  },
  {
    slug: "chip-group",
    group: "basic",
    title: "ChipGroup",
    source: "crates/components/src/chip/group.rs",
    api: "struct.ChipGroup.html",
    description: {
      en: "Organize controlled filter or input chips with roving focus and wrapping or horizontal scrolling.",
      zh: "以漫游焦点与换行或横向滚动布局组织受控筛选或输入标签组。",
    },
    usage: {
      en: "Pass ChipGroupItem entries to ChipGroup::new. Choose Single, SingleRequired, or Multiple via selection + on_change, or use single_select for required single-choice groups.",
      zh: "向 ChipGroup::new 传入 ChipGroupItem 列表。通过 selection 和 on_change 选择单选、必选单选或多选模式，或用 single_select 快速创建必选单选组。",
    },
    imports: "use gpui_material::{ChipGroup, ChipGroupItem, ChipGroupSelection};",
    code: `ChipGroup::new(
    "status-filters",
    "Status filters",
    [
        ChipGroupItem::new("open", "Open"),
        ChipGroupItem::new("review", "In review"),
        ChipGroupItem::new("done", "Done"),
    ],
)
.selection(ChipGroupSelection::Multiple, ["open".into()])
.on_change(|selected, _, _| println!("Selected: {selected:?}"))`,
  },
  {
    slug: "skeleton",
    group: "layout",
    title: "Skeleton",
    source: "crates/components/src/skeleton.rs",
    api: "struct.Skeleton.html",
    description: {
      en: "Render animated Material 3 loading placeholders with zero offscreen frames and reduced-motion support.",
      zh: "渲染具备离屏零帧调度与低动效静态回退的 Material 3 骨架屏加载占位。",
    },
    usage: {
      en: "Compose Skeleton::rect, text_line, circle, and shape (for MaterialShape polygons) while content loads. Placeholders pause offscreen and fall back to a static tint under reduced motion.",
      zh: "在数据加载期间组合使用 Skeleton::rect、text_line、circle 与 shape（支持 MaterialShape 多边形）。占位在离屏时自动暂停，在低动效下保持静态填充。",
    },
    imports: `use gpui::px;
use gpui_material::Skeleton;`,
    code: `Skeleton::text_line("headline-skeleton").width(px(220.))`,
  },
  {
    slug: "breadcrumbs",
    group: "layout",
    title: "Breadcrumbs",
    source: "crates/components/src/navigation/breadcrumbs.rs",
    api: "struct.Breadcrumbs.html",
    description: {
      en: "Navigate hierarchical desktop paths with automatic middle-segment overflow into a Material menu.",
      zh: "提供桌面端层级路径导航，并在空间不足时自动将中间路径折叠至 Material 菜单。",
    },
    usage: {
      en: "Pass ordered BreadcrumbItem entries to Breadcrumbs::new and handle ancestor navigation in on_select. Middle segments collapse automatically when constrained by max_items or available width.",
      zh: "向 Breadcrumbs::new 传入有序的 BreadcrumbItem 列表，在 on_select 中处理上级跳转。受 max_items 或可用宽度限制时，中间路径会自动折叠进溢出菜单。",
    },
    imports: "use gpui_material::{BreadcrumbItem, Breadcrumbs};",
    code: `Breadcrumbs::new(
    "doc-path",
    [
        BreadcrumbItem::new("home", "Home"),
        BreadcrumbItem::new("projects", "Projects"),
        BreadcrumbItem::new("q4", "Q4 Plan"),
    ],
)
.on_select(|id, _, _| println!("Navigate to {id}"))`,
  },
  {
    slug: "window-chrome",
    group: "layout",
    title: "TitleBar",
    source: "crates/extras/src/window_chrome.rs",
    feature: "window-chrome",
    story: null,
    description: {
      en: "Render a cross-platform client-drawn window title bar with drag regions and platform caption controls.",
      zh: "渲染支持拖动区域与平台窗口控制按钮的跨平台自绘标题栏。",
    },
    usage: {
      en: "Open the window with title_bar_window_options() and place TitleBar at the top of the root view. Provide an explicit title or omit it to mirror the live window title, and attach optional leading, content, or trailing slots.",
      zh: "使用 title_bar_window_options() 打开窗口，并将 TitleBar 放在根视图顶部。可传入自定义标题或省略以跟随窗口标题，并可附加 leading、content 或 trailing 插槽。",
    },
    imports: "use gpui_material::window_chrome::TitleBar;",
    code: `TitleBar::new("main-title-bar")
    .title("Workspace")`,
  },
  {
    slug: "data-views",
    group: "advanced",
    title: "DataTable / Tree / VirtualList",
    source: "crates/extras/src/data_views.rs",
    feature: "data-views",
    description: {
      en: "Present sortable data tables, virtualized hierarchical trees, and variable-size virtual lists with Material 3 Expressive styling.",
      zh: "以 Material 3 Expressive 风格展示可排序数据表、虚拟化层级树和可变尺寸虚拟列表。",
    },
    usage: {
      en: "Enable the data-views feature and import DataTable, Tree, and VirtualList from gpui_material::data_views. Keep sorting and row selection in application state, retain TreeState for hierarchical trees, and pair VirtualList with VirtualListScrollHandle.",
      zh: "启用 data-views 特性并从 gpui_material::data_views 导入 DataTable、Tree 与 VirtualList。由应用持有排序与行选中状态，为层级树保留 TreeState，并为 VirtualList 搭配 VirtualListScrollHandle。",
    },
    imports:
      "use gpui_material::data_views::{ColumnAlignment, DataTable, DataTableCell, DataTableColumn, DataTableRow, SortDirection};",
    code: `DataTable::new("services")
    .accessible_label("Cluster services")
    .columns([
        DataTableColumn::new("name", "Service").sort_direction(Some(SortDirection::Ascending)),
        DataTableColumn::new("latency", "Latency").alignment(ColumnAlignment::End),
    ])
    .rows([
        DataTableRow::new("api", [DataTableCell::text("API Gateway"), DataTableCell::text("18 ms")])
            .selected(true),
        DataTableRow::new("billing", [DataTableCell::text("Billing"), DataTableCell::text("12 ms")]),
    ])
    .on_sort(|column, direction, _, _| println!("Sort {column} -> {direction:?}"))
    .on_row_select(|row, _, _| println!("Selected row: {row}"))`,
  },
  {
    slug: "collapsible",
    group: "layout",
    title: "Collapsible",
    source: "crates/extras/src/disclosure/collapsible.rs",
    api: "struct.Collapsible.html",
    feature: "disclosure",
    description: {
      en: "Reveal or hide supplementary details inside a Material 3 disclosure container.",
      zh: "在 Material 3 可展开/收起容器中揭示或隐藏补充内容。",
    },
    usage: {
      en: "Pass the controlled open flag to Collapsible::new and update it from on_open_change. Customize the header with supporting text, leading or trailing slots, bordered and divider toggles, and content padding.",
      zh: "将受控展开状态传入 Collapsible::new，并在 on_open_change 中更新。可通过 supporting、leading、trailing、bordered、divider 和 content_padding 自定义外观。",
    },
    imports: "use gpui_material::{Collapsible, Text};",
    code: `Collapsible::new("release-notes", "Release notes", true)
    .supporting("Desktop highlights")
    .content(Text::new("Material 3 disclosure container."))
    .on_open_change(|open, _, _| println!("Open: {open}"))`,
  },
  {
    slug: "accordion",
    group: "layout",
    title: "Accordion",
    source: "crates/extras/src/disclosure/accordion.rs",
    api: "struct.Accordion.html",
    feature: "disclosure",
    description: {
      en: "Expand and collapse grouped sections in Standard or Segmented Material 3 layouts.",
      zh: "以 Standard 或 Segmented 样式组织可展开与收起的分组内容。",
    },
    usage: {
      en: "Pass ordered AccordionItem entries and the expanded item IDs to Accordion::new. Choose Single, SingleRequired, or Multiple via mode, and Standard or Segmented via variant. Enabled triggers support vertical roving focus.",
      zh: "向 Accordion::new 传入有序的 AccordionItem 列表和当前展开 ID。通过 mode 选择单选、必选单选或多选，通过 variant 选择 Standard 或 Segmented 样式，并支持纵向漫游焦点。",
    },
    imports: "use gpui_material::{Accordion, AccordionItem, AccordionMode, AccordionVariant, Text};",
    code: `Accordion::new(
    "faq",
    [
        AccordionItem::new("overview", "Workspace overview")
            .content(Text::new("Organized across theme, foundation, components, and extras.")),
        AccordionItem::new("keyboard", "Keyboard navigation")
            .content(Text::new("Use Up, Down, Home, End, Enter, and Space.")),
    ],
)
.mode(AccordionMode::Single)
.variant(AccordionVariant::Segmented)
.expanded(["overview"])
.on_change(|ids, _, _| println!("Expanded: {ids:?}"))`,
  },
  {
    slug: "pagination",
    group: "layout",
    title: "Pagination",
    source: "crates/extras/src/disclosure/pagination.rs",
    api: "struct.Pagination.html",
    feature: "disclosure",
    description: {
      en: "Navigate multi-page desktop collections with active page pills and ellipsis truncation.",
      zh: "使用当前页高亮胶囊和省略号截断浏览多页桌面内容。",
    },
    usage: {
      en: "Pass the 1-based current page and total page count to Pagination::new, and apply requested changes from on_change. Previous and next buttons disable automatically at boundaries and mirror in RTL.",
      zh: "向 Pagination::new 传入从 1 开始的当前页与总页数，并在 on_change 中应用切页请求。上一页/下一页按钮在边界自动禁用并在 RTL 中镜像。",
    },
    imports: "use gpui_material::Pagination;",
    code: `Pagination::new("results-pages", 5, 12)
    .accessible_label("Search results pages")
    .on_change(|page, _, _| println!("Page {page}"))`,
  },
  {
    slug: "command-palette",
    group: "advanced",
    title: "CommandPalette",
    source: "crates/extras/src/command_palette.rs",
    api: "struct.CommandPalette.html",
    feature: "command-palette",
    description: {
      en: "Filter and execute grouped desktop commands in a Ctrl/Cmd+K modal palette.",
      zh: "在桌面端 Ctrl/Cmd+K 风格模态面板中过滤并执行分组命令。",
    },
    usage: {
      en: "Retain a CommandPaletteState entity and call open or toggle from a shortcut or button. Supply typed CommandItem or CommandGroup lists, navigate with Up, Down, Home, and End, execute with Enter, and dismiss with Escape.",
      zh: "保留 CommandPaletteState 实体，并在快捷键或按钮中调用 open 或 toggle。传入 CommandItem 或 CommandGroup 列表，支持上下、Home、End 导航，Enter 执行，Escape 关闭并恢复焦点。",
    },
    imports: "use gpui_material::{CommandGroup, CommandItem, CommandPalette, CommandPaletteState, IconName};",
    code: `let palette = cx.new(|cx| CommandPaletteState::new(window, cx));

// In render
CommandPalette::grouped(
    "commands",
    &palette,
    [CommandGroup::new(
        "Navigation",
        [CommandItem::new("inbox", "Go to Inbox")
            .icon(IconName::Inbox)
            .shortcut("G I")],
    )],
)
.on_select(|item, _, _| println!("Run {}", item.label()))`,
  },
  {
    slug: "number-input",
    group: "form",
    title: "NumberInput",
    source: "crates/extras/src/specialized_inputs/number_input.rs",
    api: "struct.NumberInput.html",
    feature: "specialized-inputs",
    description: {
      en: "Edit bounded numeric values with decrement and increment steppers and keyboard Up/Down stepping.",
      zh: "通过加减步进按钮与键盘上下键编辑带范围限制的数值。",
    },
    usage: {
      en: "Retain a NumberInputState entity configured with bounds, step, and optional decimal precision. Choose Filled or Outlined variants and Split, Trailing, or None stepper layouts.",
      zh: "持久化持有配置了上下限、步长和可选小数精度的 NumberInputState 实体。按需选择 Filled 或 Outlined 外观以及 Split、Trailing 或 None 步进按钮布局。",
    },
    imports: "use gpui_material::{NumberInput, NumberInputState, NumberInputStepper};",
    code: `let state = cx.new(|cx| {
    NumberInputState::new(window, cx)
        .bounds(0.0, 20.0)
        .step(1.0)
        .default_value(4.0, window, cx)
});
NumberInput::outlined("quantity", "Quantity", &state)
    .stepper(NumberInputStepper::Split)`,
  },
  {
    slug: "otp-input",
    group: "form",
    title: "OtpInput",
    source: "crates/extras/src/specialized_inputs/otp_input.rs",
    api: "struct.OtpInput.html",
    feature: "specialized-inputs",
    description: {
      en: "Capture fixed-length verification codes with segmented slots, masking, paste, and slot cursor navigation.",
      zh: "以分段槽位采集定长验证码，支持掩码、剪贴板粘贴与槽位光标导航。",
    },
    usage: {
      en: "Retain an OtpInputState entity with the desired slot count. Supports Outlined or Filled slots, optional separator dashes, masked bullets, Ctrl/Cmd+V paste, and LTR-safe numeric slot order in RTL windows.",
      zh: "持久化持有指定长度的 OtpInputState 实体。支持 Outlined/Filled 槽位、可选分隔短横线、圆点掩码、Ctrl/Cmd+V 粘贴，以及在 RTL 窗口中保持自左向右的数字槽位顺序。",
    },
    imports: "use gpui_material::{OtpInput, OtpInputState};",
    code: `let state = cx.new(|cx| OtpInputState::new(6, window, cx));
OtpInput::new("verify-code", "Verification code", &state)
    .separator_after(3)`,
  },
  {
    slug: "color-picker",
    group: "form",
    title: "ColorPicker",
    source: "crates/extras/src/specialized_inputs/color_picker.rs",
    api: "struct.ColorPicker.html",
    feature: "specialized-inputs",
    description: {
      en: "Choose colors with Material 3 swatch presets, HSLA sliders, and live hex synchronized with Argb.",
      zh: "通过 Material 3 预设色板、HSLA 滑块与实时十六进制输入选色，并与 Argb 无缝衔接。",
    },
    usage: {
      en: "Retain a ColorPickerState entity and render either an inline panel or a popover trigger. Read state.argb(cx) to feed picked colors straight into MaterialTheme::from_seed when dynamic-color is enabled.",
      zh: "持久化持有 ColorPickerState 实体，可渲染为常驻内联面板或可关闭弹出触发器。通过 state.argb(cx) 可将选中色直接传给 dynamic-color 的 MaterialTheme::from_seed。",
    },
    imports: `use gpui_material::{ColorPicker, ColorPickerState};
use gpui_material::theme::Argb;`,
    code: `let state = cx.new(|cx| {
    ColorPickerState::new(window, cx)
        .default_argb(Argb::from_rgb(0x67, 0x50, 0xA4), window, cx)
});
ColorPicker::inline("seed-color", "Seed color", &state)`,
  },
  {
    slug: "selectable-text",
    group: "basic",
    title: "SelectableText",
    source: "crates/extras/src/rich_text/selectable_text.rs",
    api: "struct.SelectableText.html",
    feature: "rich-text",
    description: {
      en: "Render Material 3 typography that supports pointer drag selection and keyboard copy.",
      zh: "渲染支持指针拖拽选区与键盘复制的 Material 3 排版文本。",
    },
    usage: {
      en: "Use any Material type-role shorthand with color roles, wrapping, and PrimaryContainer selection highlights. Attach a shared SelectableTextGroup with document_order for multi-paragraph selection.",
      zh: "可使用全部 Material 排版快捷构造器、语义色板、自动换行与 PrimaryContainer 选区高亮；搭配共享的 SelectableTextGroup 与 document_order 实现跨段落选区。",
    },
    imports: "use gpui_material::SelectableText;",
    code: `SelectableText::body_large(
    "summary",
    "Drag to select this text and press Ctrl/Cmd+C to copy.",
)`,
  },
  {
    slug: "rich-text",
    group: "advanced",
    title: "RichText",
    source: "crates/extras/src/rich_text/viewer.rs",
    api: "struct.RichText.html",
    feature: "rich-text",
    description: {
      en: "Render Markdown and HTML with Material 3 typography, code surfaces, links, blockquotes, and ThemeScope support.",
      zh: "以 Material 3 排版体系、代码块表面、链接、引用与 ThemeScope 投影渲染 Markdown 和 HTML。",
    },
    usage: {
      en: "Use RichText::markdown or RichText::html for stateless documents, or retain a RichTextState entity for incremental streaming with push_str, selection inspection, and max_lines clamping.",
      zh: "静态内容可直接使用 RichText::markdown 或 RichText::html；流式追加、选区读取或行数截断状态检测可持久化持有 RichTextState 实体。",
    },
    imports: "use gpui_material::RichText;",
    code: `RichText::markdown(
    "release-notes",
    "### Release notes\\n\\nIncludes \`SurfaceContainerHighest\` code spans and [Material 3 links](https://m3.material.io).",
)`,
  },
].map((guide) => ({ story: guide.slug, ...guide }));
