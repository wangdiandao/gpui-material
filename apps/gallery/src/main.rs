use gpui::{
    App, AppContext, QuitMode, TitlebarOptions, WindowBounds, WindowHandle, WindowOptions, px, size,
};
use gpui_base::Root;
use gpui_material::foundation::{
    MaterialRoot, material_theme, set_material_theme, set_window_material_theme,
};
use gpui_material::theme::{Appearance, MaterialTheme};

use gpui_material_story::{Gallery, Story, init_gallery};

const TITLE: &str = "GPUI Material Gallery";

fn main() {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let two_windows = arguments.iter().any(|arg| arg == "--two-windows");
    let rtl = arguments.iter().any(|arg| arg == "--rtl");
    let reduced_motion = arguments.iter().any(|arg| arg == "--reduce-motion");
    let dark = arguments.iter().any(|arg| arg == "--dark");
    let initial_story = arguments
        .iter()
        .find(|arg| !arg.starts_with('-'))
        .and_then(|name| Story::find(name))
        .unwrap_or_default();
    gpui_platform::application()
        .with_quit_mode(QuitMode::LastWindowClosed)
        .run(move |cx| {
            init_gallery(cx);
            if rtl || dark {
                let mut theme = MaterialTheme::baseline(if dark {
                    Appearance::Dark
                } else {
                    Appearance::Light
                });
                if rtl {
                    theme.layout.direction = gpui_material::theme::Direction::Rtl;
                }
                set_material_theme(theme, cx);
            }
            if reduced_motion {
                cx.set_reduce_motion(true);
            }
            let result = open_gallery(cx, false, initial_story).and_then(|_| {
                if two_windows {
                    open_gallery(cx, true, initial_story)?;
                }
                Ok(())
            });
            if let Err(error) = result {
                eprintln!("Could not open {TITLE}: {error:#}");
                std::process::exit(1);
            }
        });
}

fn open_gallery(
    cx: &mut App,
    local_dark: bool,
    initial_story: Story,
) -> gpui::Result<WindowHandle<Root>> {
    let options = WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some(if local_dark {
                format!("{TITLE} · Dark preview").into()
            } else {
                TITLE.into()
            }),
            ..Default::default()
        }),
        window_bounds: Some(WindowBounds::centered(size(px(1040.), px(744.)), cx)),
        window_min_size: Some(size(px(320.), px(240.))),
        ..Default::default()
    };
    MaterialRoot::open_window(options, cx, |window, cx| {
        if local_dark {
            let mut theme = material_theme(cx).clone();
            theme.colors = gpui_material::theme::ColorScheme::baseline(Appearance::Dark);
            set_window_material_theme(Some(theme), window, cx);
        }
        cx.new(|cx| Gallery::new(initial_story, window, cx).with_window_theme_toggle(local_dark))
    })
}

#[cfg(all(test, feature = "test-support"))]
mod desktop_tests;
