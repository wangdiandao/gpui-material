use super::platform::configure_window_options;
use super::*;
use gpui::{TitlebarOptions, WindowDecorations, WindowOptions, point, px, size};

#[test]
fn chrome_options_preserve_the_callers_window_settings() {
    let min_size = size(px(360.), px(240.));
    let options = WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some("My window".into()),
            ..Default::default()
        }),
        window_min_size: Some(min_size),
        is_resizable: false,
        is_minimizable: false,
        ..Default::default()
    };

    let options = title_bar_window_options(title_bar_window_options(options));
    let titlebar = options.titlebar.unwrap();
    assert!(titlebar.appears_transparent);
    assert_eq!(titlebar.title.as_deref(), Some("My window"));
    assert_eq!(options.window_min_size, Some(min_size));
    assert!(!options.is_resizable);
    assert!(!options.is_minimizable);
}

#[test]
fn chrome_options_create_a_titlebar_configuration_when_absent() {
    let options = title_bar_window_options(WindowOptions {
        titlebar: None,
        ..Default::default()
    });
    assert!(options.titlebar.unwrap().appears_transparent);
}

#[test]
fn macos_options_center_traffic_lights_from_height_and_respect_caller_override() {
    let options =
        configure_window_options(WindowOptions::default(), px(40.), ChromePlatform::MacOs);
    let titlebar = options.titlebar.unwrap();
    assert!(titlebar.appears_transparent);
    assert_eq!(
        titlebar.traffic_light_position,
        Some(point(px(12.), px(13.)))
    );
    assert_eq!(options.window_decorations, None);

    let custom_pos = point(px(16.), px(18.));
    let preserved = configure_window_options(
        WindowOptions {
            titlebar: Some(TitlebarOptions {
                traffic_light_position: Some(custom_pos),
                ..Default::default()
            }),
            ..Default::default()
        },
        px(60.),
        ChromePlatform::MacOs,
    );
    assert_eq!(
        preserved.titlebar.unwrap().traffic_light_position,
        Some(custom_pos)
    );
}

#[test]
fn linux_options_request_client_decorations_unless_explicitly_set() {
    let options =
        configure_window_options(WindowOptions::default(), px(48.), ChromePlatform::Linux);
    assert_eq!(options.window_decorations, Some(WindowDecorations::Client));
    assert!(options.titlebar.unwrap().appears_transparent);

    let preserved = configure_window_options(
        WindowOptions {
            window_decorations: Some(WindowDecorations::Server),
            ..Default::default()
        },
        px(48.),
        ChromePlatform::Linux,
    );
    assert_eq!(
        preserved.window_decorations,
        Some(WindowDecorations::Server)
    );
}

#[test]
fn windows_options_do_not_force_traffic_lights_or_linux_decorations() {
    let options =
        configure_window_options(WindowOptions::default(), px(48.), ChromePlatform::Windows);
    let titlebar = options.titlebar.unwrap();
    assert!(titlebar.appears_transparent);
    assert_eq!(titlebar.traffic_light_position, None);
    assert_eq!(options.window_decorations, None);
}

#[test]
fn invalid_height_falls_back_to_default_for_macos_traffic_lights() {
    let options = configure_window_options(
        WindowOptions::default(),
        px(f32::NAN),
        ChromePlatform::MacOs,
    );
    assert_eq!(
        options.titlebar.unwrap().traffic_light_position,
        Some(point(px(12.), px(17.)))
    );
}
