use std::{any::Any, collections::HashMap};

use gpui::{AnyElement, App, Context, Window};

use super::ComponentPageView;

/// One declarative configuration control rendered in the Configuration card.
#[derive(Clone, Copy, Debug)]
pub enum Control {
    Choice {
        id: &'static str,
        label: &'static str,
        options: &'static [&'static str],
        default: usize,
    },
    Switch {
        id: &'static str,
        label: &'static str,
        default: bool,
    },
    Slider {
        id: &'static str,
        label: &'static str,
        min: f32,
        max: f32,
        step: f32,
        default: f32,
    },
    Text {
        id: &'static str,
        label: &'static str,
        default: &'static str,
    },
}

impl Control {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Choice { id, .. }
            | Self::Switch { id, .. }
            | Self::Slider { id, .. }
            | Self::Text { id, .. } => id,
        }
    }
}

/// A titled group of configuration controls.
#[derive(Clone, Copy, Debug)]
pub struct Section {
    pub title: &'static str,
    pub controls: &'static [Control],
}

/// A value override applied by a [`Scenario`] preset.
#[derive(Clone, Copy, Debug)]
pub enum Set {
    Option(&'static str),
    Bool(bool),
    Number(f32),
    Text(&'static str),
}

/// One named scenario preset shown at the top of the Configuration card.
#[derive(Clone, Copy, Debug)]
pub struct Scenario {
    pub name: &'static str,
    pub values: &'static [(&'static str, Set)],
}

/// Runtime value of one configuration control.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Choice(usize),
    Bool(bool),
    Number(f32),
    Text(String),
}

/// Current playground configuration, passed to `preview` and `code`.
#[derive(Clone)]
pub struct Config {
    playground: &'static Playground,
    values: Vec<(&'static str, Value)>,
}

impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.playground, other.playground) && self.values == other.values
    }
}

impl Config {
    pub fn defaults(playground: &'static Playground) -> Self {
        let mut values = Vec::new();
        for section in playground.sections {
            for control in section.controls {
                let default = match *control {
                    Control::Choice { default, .. } => Value::Choice(default),
                    Control::Switch { default, .. } => Value::Bool(default),
                    Control::Slider { default, .. } => Value::Number(default),
                    Control::Text { default, .. } => Value::Text(default.to_owned()),
                };
                values.push((control.id(), default));
            }
        }
        Self { playground, values }
    }

    pub fn scenario(playground: &'static Playground, scenario: &Scenario) -> Self {
        let mut config = Self::defaults(playground);
        for &(id, value) in scenario.values {
            config.apply_set(id, value);
        }
        config
    }

    fn apply_set(&mut self, id: &str, set: Set) {
        let Some(control) = self.playground.control(id) else {
            panic!("unknown playground control `{id}`");
        };
        let next = match (control, set) {
            (Control::Choice { options, .. }, Set::Option(option)) => {
                let ix = options
                    .iter()
                    .position(|candidate| *candidate == option)
                    .unwrap_or_else(|| panic!("unknown option `{option}` for `{id}`"));
                Value::Choice(ix)
            }
            (Control::Switch { .. }, Set::Bool(v)) => Value::Bool(v),
            (Control::Slider { min, max, .. }, Set::Number(v)) => Value::Number(v.clamp(min, max)),
            (Control::Text { .. }, Set::Text(v)) => Value::Text(v.to_owned()),
            _ => panic!("mismatched scenario value type for `{id}`"),
        };
        self.set_value(id, next);
    }

    pub(super) fn get(&self, id: &str) -> &Value {
        self.values
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
            .unwrap_or_else(|| panic!("unknown playground control `{id}`"))
    }

    pub(super) fn set_value(&mut self, id: &str, value: Value) {
        let slot = self
            .values
            .iter_mut()
            .find(|(key, _)| *key == id)
            .unwrap_or_else(|| panic!("unknown playground control `{id}`"));
        slot.1 = value;
    }

    pub fn option(&self, id: &str) -> &'static str {
        let Control::Choice { options, .. } = self
            .playground
            .control(id)
            .unwrap_or_else(|| panic!("unknown choice `{id}`"))
        else {
            panic!("control `{id}` is not Choice");
        };
        let Value::Choice(ix) = *self.get(id) else {
            unreachable!();
        };
        options[ix]
    }

    pub fn bool(&self, id: &str) -> bool {
        let Value::Bool(value) = *self.get(id) else {
            panic!("control `{id}` is not Switch");
        };
        value
    }

    pub fn number(&self, id: &str) -> f32 {
        let Value::Number(value) = *self.get(id) else {
            panic!("control `{id}` is not Slider");
        };
        value
    }

    pub fn text(&self, id: &str) -> &str {
        let Value::Text(ref value) = *self.get(id) else {
            panic!("control `{id}` is not Text");
        };
        value
    }
}

/// Declarative definition of one component playground.
pub struct Playground {
    pub scenarios: &'static [Scenario],
    pub sections: &'static [Section],
    pub preview:
        fn(&Config, &mut PreviewCtx, &mut Window, &mut Context<ComponentPageView>) -> AnyElement,
    pub code: fn(&Config) -> String,
}

impl Playground {
    pub fn control(&self, id: &str) -> Option<Control> {
        self.sections
            .iter()
            .flat_map(|section| section.controls.iter().copied())
            .find(|control| control.id() == id)
    }
}

pub type StatusCallback = std::rc::Rc<dyn Fn(String, &mut App)>;

/// Helper passed to a playground's `preview` function for retained preview-only state
/// and two-way syncing back to the Configuration card.
pub struct PreviewCtx<'a> {
    pub(super) locals: &'a mut HashMap<&'static str, Box<dyn Any>>,
}

impl<'a> PreviewCtx<'a> {
    /// Returns or initializes preview-local state retained across redraws and navigation.
    pub fn local<T: 'static>(&mut self, key: &'static str, init: impl FnOnce() -> T) -> &mut T {
        self.locals
            .entry(key)
            .or_insert_with(|| Box::new(init()))
            .downcast_mut::<T>()
            .expect("playground local state type mismatch")
    }

    /// Returns a callback that writes a `bool` back to a `Control::Switch`.
    pub fn on_bool(
        &self,
        id: &'static str,
        cx: &Context<ComponentPageView>,
    ) -> impl Fn(bool, &mut Window, &mut App) + 'static {
        let owner = cx.weak_entity();
        move |next, window, cx| {
            let _ = owner.update(cx, |this, cx| {
                this.set_value(id, Value::Bool(next), window, cx);
            });
        }
    }

    /// Returns a callback that writes a choice index back to a `Control::Choice`.
    pub fn on_choice(
        &self,
        id: &'static str,
        cx: &Context<ComponentPageView>,
    ) -> impl Fn(usize, &mut Window, &mut App) + 'static {
        let owner = cx.weak_entity();
        move |next, window, cx| {
            let _ = owner.update(cx, |this, cx| {
                this.set_value(id, Value::Choice(next), window, cx);
            });
        }
    }

    /// Returns a callback that records an interactive status message shown below the preview.
    pub fn on_status(&self, cx: &Context<ComponentPageView>) -> StatusCallback {
        let owner = cx.weak_entity();
        std::rc::Rc::new(move |message, cx| {
            let _ = owner.update(cx, |this, cx| {
                this.last_event = Some(message);
                cx.notify();
            });
        })
    }
}
