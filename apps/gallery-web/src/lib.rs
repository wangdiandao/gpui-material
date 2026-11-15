//! Browser entry point for the Material Gallery and its embedded stories.
#![cfg(target_arch = "wasm32")]

use std::{borrow::Cow, cell::RefCell, rc::Rc};

use gpui::{
    AnyView, App, AppContext, Application, ApplicationHandle, Context, InteractiveElement,
    IntoElement, ParentElement, Render, ScrollHandle, StatefulInteractiveElement, Styled, Window,
    WindowOptions, div,
};
use gpui_base::{Root, Scrollbar, ScrollbarMode};
use gpui_material::{
    Surface,
    foundation::set_material_theme,
    theme::{Appearance, ColorRole, MaterialTheme},
};
use gpui_material_story::{Story, init_gallery};
use gpui_web::{CanvasFontFallback, WebBackendPreference, WebPlatform};
use js_sys::Function;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

thread_local! {
    static APPLICATION: RefCell<Option<ApplicationHandle>> = const { RefCell::new(None) };
}

#[wasm_bindgen]
pub fn run(story_id: String, dark: bool, ready: Function) -> Result<(), JsValue> {
    let story = Story::find(&story_id)
        .filter(|story| *story != Story::Introduction)
        .ok_or_else(|| JsValue::from_str("Unknown component story"))?;

    gpui_web::init_logging();
    let platform = Rc::new(WebPlatform::new_with_backend_and_font_fallback(
        false,
        WebBackendPreference::Auto,
        CanvasFontFallback::EmojiAndCjk,
    ));
    let application = Application::with_platform(platform).run_embedded(move |cx| {
        if let Err(error) = open_gallery(story, dark, &ready, cx) {
            let _ = ready.call1(&JsValue::NULL, &JsValue::from_str(&format!("{error:#}")));
        }
    });
    APPLICATION.with(|slot| *slot.borrow_mut() = Some(application));
    Ok(())
}

#[wasm_bindgen]
pub fn set_theme(dark: bool) {
    APPLICATION.with(|slot| {
        if let Some(application) = slot.borrow().as_ref() {
            application.update(|cx| apply_theme(dark, cx));
        }
    });
}

fn apply_theme(dark: bool, cx: &mut App) {
    let appearance = if dark {
        Appearance::Dark
    } else {
        Appearance::Light
    };
    set_material_theme(MaterialTheme::baseline(appearance), cx);
}

fn open_gallery(story: Story, dark: bool, ready: &Function, cx: &mut App) -> gpui::Result<()> {
    cx.text_system().add_fonts(vec![Cow::Borrowed(
        include_bytes!("../fonts/IBMPlexSans-Regular.ttf").as_slice(),
    )])?;
    init_gallery(cx);
    apply_theme(dark, cx);

    let ready = ready.clone();
    cx.open_window(WindowOptions::default(), |window, cx| {
        let view = story
            .new_view(window, cx)
            .expect("component story has a view");
        let view = cx.new(|_| EmbeddedStory {
            view,
            scroll: ScrollHandle::new(),
        });
        window.on_next_frame(move |_, _| {
            let _ = ready.call0(&JsValue::NULL);
        });
        cx.new(|cx| Root::new(view, window, cx))
    })?;
    cx.activate(true);
    Ok(())
}

struct EmbeddedStory {
    view: AnyView,
    scroll: ScrollHandle,
}

impl Render for EmbeddedStory {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Surface::new(ColorRole::Surface)
            .size_full()
            .relative()
            .child(
                div()
                    .id("embedded-story")
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .p_4()
                    .child(self.view.clone()),
            )
            .child(Scrollbar::vertical(&self.scroll).mode(ScrollbarMode::Always))
    }
}
