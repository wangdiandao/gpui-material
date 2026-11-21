use gpui::QuitMode;
use gpui_material::foundation::set_material_theme;
use gpui_material::theme::{Appearance, MaterialTheme};

use gpui_material_story::{GALLERY_TITLE, Story, init_gallery, open_gallery};

#[cfg(feature = "frame-profile")]
mod frame_profile;

fn main() {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let two_windows = arguments.iter().any(|arg| arg == "--two-windows");
    let rtl = arguments.iter().any(|arg| arg == "--rtl");
    let reduced_motion = arguments.iter().any(|arg| arg == "--reduce-motion");
    let dark = arguments.iter().any(|arg| arg == "--dark");
    let frame_profile = arguments.iter().any(|arg| arg == "--frame-profile");
    let window_chrome = arguments.iter().any(|arg| arg == "--window-chrome");
    if frame_profile && (two_windows || !cfg!(feature = "frame-profile")) {
        eprintln!("--frame-profile requires the frame-profile feature and a single window");
        std::process::exit(2);
    }
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
            let result = open_gallery(cx, false, initial_story, window_chrome).and_then(|_| {
                if two_windows {
                    open_gallery(cx, true, initial_story, window_chrome)?;
                }
                Ok(())
            });
            if let Err(error) = result {
                eprintln!("Could not open {GALLERY_TITLE}: {error:#}");
                std::process::exit(1);
            }
            #[cfg(feature = "frame-profile")]
            if frame_profile {
                frame_profile::capture(cx);
            }
        });
}
