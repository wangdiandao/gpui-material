use gpui::{App, AppContext, Entity, Render, Window, WindowHandle, WindowOptions};
use gpui_base::Root;

/// Open the same Base Root in native and test windows. Initialize Base once per App.
pub fn open_probe<V: Render>(
    cx: &mut App,
    options: WindowOptions,
    build: impl FnOnce(&mut Window, &mut App) -> Entity<V>,
) -> gpui::Result<WindowHandle<Root>> {
    cx.open_window(options, |window, cx| {
        let view = build(window, cx);
        cx.new(|cx| Root::new(view, window, cx))
    })
}
