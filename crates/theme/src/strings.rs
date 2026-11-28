use std::sync::{Arc, LazyLock};

/// Pure-value table of built-in default labels and accessibility copy.
///
/// Uses shared [`Arc<str>`] storage (matching [`crate::FontFamilies`]) so cloning
/// and scoping strings across subtrees does not allocate on the heap.
///
/// All fields default to the library's English labels so existing applications
/// behave unchanged until they install custom strings via
/// `set_material_strings` or `ThemeScope::strings`. Explicit per-component label
/// setters always take precedence over `MaterialStrings`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterialStrings {
    /// Generic dismiss label (defaults to `"Dismiss"`, e.g. `Banner`).
    pub dismiss: Arc<str>,
    /// Accessible label for dismissing a notification message (defaults to `"Dismiss message"`, e.g. `Snackbar`).
    pub dismiss_message: Arc<str>,
    /// Accessible label for clicking a modal backdrop scrim (defaults to `"Dismiss overlay"`, e.g. `Scrim`).
    pub dismiss_overlay: Arc<str>,
    /// Generic close label (defaults to `"Close"`, e.g. `Sheet`, `CaptionButtons`).
    pub close: Arc<str>,
    /// Accessible label for closing floating action menus (defaults to `"Close actions"`, e.g. `FabMenu`).
    pub close_actions: Arc<str>,
    /// Accessible label for closing search views (defaults to `"Close search"`, e.g. `SearchView`).
    pub close_search: Arc<str>,
    /// Generic cancellation label (defaults to `"Cancel"`, e.g. `Dialog`).
    pub cancel: Arc<str>,
    /// Generic confirmation label (defaults to `"OK"`, e.g. `DatePickerDialog`, `TimePickerDialog`).
    pub confirm: Arc<str>,
    /// Accessible label for clearing a text input (defaults to `"Clear"`, e.g. `TextField`).
    pub clear: Arc<str>,
    /// Accessible label for revealing masked password text (defaults to `"Show password"`, e.g. `TextField`).
    pub show_password: Arc<str>,
    /// Accessible label for hiding revealed password text (defaults to `"Hide password"`, e.g. `TextField`).
    pub hide_password: Arc<str>,
    /// Accessible label for opening dropdown options (defaults to `"Open options"`, e.g. `ExposedDropdown`).
    pub open_options: Arc<str>,
    /// Accessible label for closing dropdown options (defaults to `"Close options"`, e.g. `ExposedDropdown`).
    pub close_options: Arc<str>,
    /// Empty-state message when a filterable dropdown has no matches (defaults to `"No matching options"`, e.g. `ExposedDropdown`).
    pub no_matching_options: Arc<str>,
    /// Accessible label for overflow menu triggers (defaults to `"More actions"`, e.g. `AppBarRow`, `AppBarColumn`, `ButtonGroup`, `Toolbar`).
    pub more_actions: Arc<str>,
    /// Accessible label for app bar action strips (defaults to `"App bar actions"`, e.g. `AppBarRow`, `AppBarColumn`).
    pub app_bar_actions: Arc<str>,
    /// Accessible label for bottom app bar landmarks (defaults to `"Bottom app bar"`, e.g. `BottomAppBar`).
    pub bottom_app_bar: Arc<str>,
    /// Accessible label for breadcrumb navigation landmarks (defaults to `"Breadcrumb"`, e.g. `Breadcrumbs`).
    pub breadcrumb: Arc<str>,
    /// Accessible label for collapsed breadcrumb overflow triggers (defaults to `"More breadcrumbs"`, e.g. `Breadcrumbs`).
    pub more_breadcrumbs: Arc<str>,
    /// Default navigation landmark and modal sheet title (defaults to `"Navigation"`, e.g. `NavigationRail`).
    pub navigation: Arc<str>,
    /// Accessible label for stepper containers (defaults to `"Steps"`, e.g. `Stepper`).
    pub steps: Arc<str>,
    /// Label for refresh actions and loading indicators (defaults to `"Refresh"`, e.g. `PullToRefresh`).
    pub refresh: Arc<str>,
    /// Accessible label for bottom sheet resize drag handles (defaults to `"Resize bottom sheet"`, e.g. `Sheet`).
    pub resize_bottom_sheet: Arc<str>,
    /// Label for expanding a collapsible bottom sheet (defaults to `"Expand"`, e.g. `Sheet`).
    pub expand: Arc<str>,
    /// Label for collapsing an expanded bottom sheet (defaults to `"Collapse"`, e.g. `Sheet`).
    pub collapse: Arc<str>,
    /// Default accessible label for drop targets (defaults to `"Drop target"`, e.g. `DropTarget`).
    pub drop_target: Arc<str>,
    /// Default prompt heading for drop targets (defaults to `"Drop files here"`, e.g. `DropTarget`).
    pub drop_files_here: Arc<str>,
    /// Default loading status label for search views (defaults to `"Searching"`, e.g. `SearchView`).
    pub searching: Arc<str>,
    /// Default empty-result label for search views (defaults to `"No results"`, e.g. `SearchView`).
    pub no_results: Arc<str>,
    /// Default idle prompt for search views (defaults to `"Enter a search term"`, e.g. `SearchView`).
    pub search_prompt: Arc<str>,
    /// Accessible label for window minimize caption buttons (defaults to `"Minimize"`, e.g. `CaptionButtons`).
    pub minimize: Arc<str>,
    /// Accessible label for window maximize caption buttons (defaults to `"Maximize"`, e.g. `CaptionButtons`).
    pub maximize: Arc<str>,
    /// Accessible label for window restore caption buttons (defaults to `"Restore"`, e.g. `CaptionButtons`).
    pub restore: Arc<str>,
}

impl MaterialStrings {
    /// Sets [`Self::dismiss`].
    pub fn dismiss(mut self, value: impl Into<Arc<str>>) -> Self {
        self.dismiss = value.into();
        self
    }

    /// Sets [`Self::dismiss_message`].
    pub fn dismiss_message(mut self, value: impl Into<Arc<str>>) -> Self {
        self.dismiss_message = value.into();
        self
    }

    /// Sets [`Self::dismiss_overlay`].
    pub fn dismiss_overlay(mut self, value: impl Into<Arc<str>>) -> Self {
        self.dismiss_overlay = value.into();
        self
    }

    /// Sets [`Self::close`].
    pub fn close(mut self, value: impl Into<Arc<str>>) -> Self {
        self.close = value.into();
        self
    }

    /// Sets [`Self::close_actions`].
    pub fn close_actions(mut self, value: impl Into<Arc<str>>) -> Self {
        self.close_actions = value.into();
        self
    }

    /// Sets [`Self::close_search`].
    pub fn close_search(mut self, value: impl Into<Arc<str>>) -> Self {
        self.close_search = value.into();
        self
    }

    /// Sets [`Self::cancel`].
    pub fn cancel(mut self, value: impl Into<Arc<str>>) -> Self {
        self.cancel = value.into();
        self
    }

    /// Sets [`Self::confirm`].
    pub fn confirm(mut self, value: impl Into<Arc<str>>) -> Self {
        self.confirm = value.into();
        self
    }

    /// Sets [`Self::clear`].
    pub fn clear(mut self, value: impl Into<Arc<str>>) -> Self {
        self.clear = value.into();
        self
    }

    /// Sets [`Self::show_password`].
    pub fn show_password(mut self, value: impl Into<Arc<str>>) -> Self {
        self.show_password = value.into();
        self
    }

    /// Sets [`Self::hide_password`].
    pub fn hide_password(mut self, value: impl Into<Arc<str>>) -> Self {
        self.hide_password = value.into();
        self
    }

    /// Sets [`Self::open_options`].
    pub fn open_options(mut self, value: impl Into<Arc<str>>) -> Self {
        self.open_options = value.into();
        self
    }

    /// Sets [`Self::close_options`].
    pub fn close_options(mut self, value: impl Into<Arc<str>>) -> Self {
        self.close_options = value.into();
        self
    }

    /// Sets [`Self::no_matching_options`].
    pub fn no_matching_options(mut self, value: impl Into<Arc<str>>) -> Self {
        self.no_matching_options = value.into();
        self
    }

    /// Sets [`Self::more_actions`].
    pub fn more_actions(mut self, value: impl Into<Arc<str>>) -> Self {
        self.more_actions = value.into();
        self
    }

    /// Sets [`Self::app_bar_actions`].
    pub fn app_bar_actions(mut self, value: impl Into<Arc<str>>) -> Self {
        self.app_bar_actions = value.into();
        self
    }

    /// Sets [`Self::bottom_app_bar`].
    pub fn bottom_app_bar(mut self, value: impl Into<Arc<str>>) -> Self {
        self.bottom_app_bar = value.into();
        self
    }

    /// Sets [`Self::breadcrumb`].
    pub fn breadcrumb(mut self, value: impl Into<Arc<str>>) -> Self {
        self.breadcrumb = value.into();
        self
    }

    /// Sets [`Self::more_breadcrumbs`].
    pub fn more_breadcrumbs(mut self, value: impl Into<Arc<str>>) -> Self {
        self.more_breadcrumbs = value.into();
        self
    }

    /// Sets [`Self::navigation`].
    pub fn navigation(mut self, value: impl Into<Arc<str>>) -> Self {
        self.navigation = value.into();
        self
    }

    /// Sets [`Self::steps`].
    pub fn steps(mut self, value: impl Into<Arc<str>>) -> Self {
        self.steps = value.into();
        self
    }

    /// Sets [`Self::refresh`].
    pub fn refresh(mut self, value: impl Into<Arc<str>>) -> Self {
        self.refresh = value.into();
        self
    }

    /// Sets [`Self::resize_bottom_sheet`].
    pub fn resize_bottom_sheet(mut self, value: impl Into<Arc<str>>) -> Self {
        self.resize_bottom_sheet = value.into();
        self
    }

    /// Sets [`Self::expand`].
    pub fn expand(mut self, value: impl Into<Arc<str>>) -> Self {
        self.expand = value.into();
        self
    }

    /// Sets [`Self::collapse`].
    pub fn collapse(mut self, value: impl Into<Arc<str>>) -> Self {
        self.collapse = value.into();
        self
    }

    /// Sets [`Self::drop_target`].
    pub fn drop_target(mut self, value: impl Into<Arc<str>>) -> Self {
        self.drop_target = value.into();
        self
    }

    /// Sets [`Self::drop_files_here`].
    pub fn drop_files_here(mut self, value: impl Into<Arc<str>>) -> Self {
        self.drop_files_here = value.into();
        self
    }

    /// Sets [`Self::searching`].
    pub fn searching(mut self, value: impl Into<Arc<str>>) -> Self {
        self.searching = value.into();
        self
    }

    /// Sets [`Self::no_results`].
    pub fn no_results(mut self, value: impl Into<Arc<str>>) -> Self {
        self.no_results = value.into();
        self
    }

    /// Sets [`Self::search_prompt`].
    pub fn search_prompt(mut self, value: impl Into<Arc<str>>) -> Self {
        self.search_prompt = value.into();
        self
    }

    /// Sets [`Self::minimize`].
    pub fn minimize(mut self, value: impl Into<Arc<str>>) -> Self {
        self.minimize = value.into();
        self
    }

    /// Sets [`Self::maximize`].
    pub fn maximize(mut self, value: impl Into<Arc<str>>) -> Self {
        self.maximize = value.into();
        self
    }

    /// Sets [`Self::restore`].
    pub fn restore(mut self, value: impl Into<Arc<str>>) -> Self {
        self.restore = value.into();
        self
    }
}

impl Default for MaterialStrings {
    fn default() -> Self {
        static DEFAULT: LazyLock<MaterialStrings> = LazyLock::new(|| MaterialStrings {
            dismiss: Arc::from("Dismiss"),
            dismiss_message: Arc::from("Dismiss message"),
            dismiss_overlay: Arc::from("Dismiss overlay"),
            close: Arc::from("Close"),
            close_actions: Arc::from("Close actions"),
            close_search: Arc::from("Close search"),
            cancel: Arc::from("Cancel"),
            confirm: Arc::from("OK"),
            clear: Arc::from("Clear"),
            show_password: Arc::from("Show password"),
            hide_password: Arc::from("Hide password"),
            open_options: Arc::from("Open options"),
            close_options: Arc::from("Close options"),
            no_matching_options: Arc::from("No matching options"),
            more_actions: Arc::from("More actions"),
            app_bar_actions: Arc::from("App bar actions"),
            bottom_app_bar: Arc::from("Bottom app bar"),
            breadcrumb: Arc::from("Breadcrumb"),
            more_breadcrumbs: Arc::from("More breadcrumbs"),
            navigation: Arc::from("Navigation"),
            steps: Arc::from("Steps"),
            refresh: Arc::from("Refresh"),
            resize_bottom_sheet: Arc::from("Resize bottom sheet"),
            expand: Arc::from("Expand"),
            collapse: Arc::from("Collapse"),
            drop_target: Arc::from("Drop target"),
            drop_files_here: Arc::from("Drop files here"),
            searching: Arc::from("Searching"),
            no_results: Arc::from("No results"),
            search_prompt: Arc::from("Enter a search term"),
            minimize: Arc::from("Minimize"),
            maximize: Arc::from("Maximize"),
            restore: Arc::from("Restore"),
        });
        DEFAULT.clone()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for MaterialStrings {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(serde::Serialize)]
        #[serde(rename = "MaterialStrings")]
        struct SerializedMaterialStrings<'a> {
            dismiss: &'a str,
            dismiss_message: &'a str,
            dismiss_overlay: &'a str,
            close: &'a str,
            close_actions: &'a str,
            close_search: &'a str,
            cancel: &'a str,
            confirm: &'a str,
            clear: &'a str,
            show_password: &'a str,
            hide_password: &'a str,
            open_options: &'a str,
            close_options: &'a str,
            no_matching_options: &'a str,
            more_actions: &'a str,
            app_bar_actions: &'a str,
            bottom_app_bar: &'a str,
            breadcrumb: &'a str,
            more_breadcrumbs: &'a str,
            navigation: &'a str,
            steps: &'a str,
            refresh: &'a str,
            resize_bottom_sheet: &'a str,
            expand: &'a str,
            collapse: &'a str,
            drop_target: &'a str,
            drop_files_here: &'a str,
            searching: &'a str,
            no_results: &'a str,
            search_prompt: &'a str,
            minimize: &'a str,
            maximize: &'a str,
            restore: &'a str,
        }

        SerializedMaterialStrings {
            dismiss: self.dismiss.as_ref(),
            dismiss_message: self.dismiss_message.as_ref(),
            dismiss_overlay: self.dismiss_overlay.as_ref(),
            close: self.close.as_ref(),
            close_actions: self.close_actions.as_ref(),
            close_search: self.close_search.as_ref(),
            cancel: self.cancel.as_ref(),
            confirm: self.confirm.as_ref(),
            clear: self.clear.as_ref(),
            show_password: self.show_password.as_ref(),
            hide_password: self.hide_password.as_ref(),
            open_options: self.open_options.as_ref(),
            close_options: self.close_options.as_ref(),
            no_matching_options: self.no_matching_options.as_ref(),
            more_actions: self.more_actions.as_ref(),
            app_bar_actions: self.app_bar_actions.as_ref(),
            bottom_app_bar: self.bottom_app_bar.as_ref(),
            breadcrumb: self.breadcrumb.as_ref(),
            more_breadcrumbs: self.more_breadcrumbs.as_ref(),
            navigation: self.navigation.as_ref(),
            steps: self.steps.as_ref(),
            refresh: self.refresh.as_ref(),
            resize_bottom_sheet: self.resize_bottom_sheet.as_ref(),
            expand: self.expand.as_ref(),
            collapse: self.collapse.as_ref(),
            drop_target: self.drop_target.as_ref(),
            drop_files_here: self.drop_files_here.as_ref(),
            searching: self.searching.as_ref(),
            no_results: self.no_results.as_ref(),
            search_prompt: self.search_prompt.as_ref(),
            minimize: self.minimize.as_ref(),
            maximize: self.maximize.as_ref(),
            restore: self.restore.as_ref(),
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for MaterialStrings {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Default, serde::Deserialize)]
        #[serde(rename = "MaterialStrings", default)]
        struct SerializedMaterialStrings {
            dismiss: Option<String>,
            dismiss_message: Option<String>,
            dismiss_overlay: Option<String>,
            close: Option<String>,
            close_actions: Option<String>,
            close_search: Option<String>,
            cancel: Option<String>,
            confirm: Option<String>,
            clear: Option<String>,
            show_password: Option<String>,
            hide_password: Option<String>,
            open_options: Option<String>,
            close_options: Option<String>,
            no_matching_options: Option<String>,
            more_actions: Option<String>,
            app_bar_actions: Option<String>,
            bottom_app_bar: Option<String>,
            breadcrumb: Option<String>,
            more_breadcrumbs: Option<String>,
            navigation: Option<String>,
            steps: Option<String>,
            refresh: Option<String>,
            resize_bottom_sheet: Option<String>,
            expand: Option<String>,
            collapse: Option<String>,
            drop_target: Option<String>,
            drop_files_here: Option<String>,
            searching: Option<String>,
            no_results: Option<String>,
            search_prompt: Option<String>,
            minimize: Option<String>,
            maximize: Option<String>,
            restore: Option<String>,
        }

        let raw = SerializedMaterialStrings::deserialize(deserializer)?;
        let defaults = Self::default();
        Ok(Self {
            dismiss: raw.dismiss.map(Into::into).unwrap_or(defaults.dismiss),
            dismiss_message: raw
                .dismiss_message
                .map(Into::into)
                .unwrap_or(defaults.dismiss_message),
            dismiss_overlay: raw
                .dismiss_overlay
                .map(Into::into)
                .unwrap_or(defaults.dismiss_overlay),
            close: raw.close.map(Into::into).unwrap_or(defaults.close),
            close_actions: raw
                .close_actions
                .map(Into::into)
                .unwrap_or(defaults.close_actions),
            close_search: raw
                .close_search
                .map(Into::into)
                .unwrap_or(defaults.close_search),
            cancel: raw.cancel.map(Into::into).unwrap_or(defaults.cancel),
            confirm: raw.confirm.map(Into::into).unwrap_or(defaults.confirm),
            clear: raw.clear.map(Into::into).unwrap_or(defaults.clear),
            show_password: raw
                .show_password
                .map(Into::into)
                .unwrap_or(defaults.show_password),
            hide_password: raw
                .hide_password
                .map(Into::into)
                .unwrap_or(defaults.hide_password),
            open_options: raw
                .open_options
                .map(Into::into)
                .unwrap_or(defaults.open_options),
            close_options: raw
                .close_options
                .map(Into::into)
                .unwrap_or(defaults.close_options),
            no_matching_options: raw
                .no_matching_options
                .map(Into::into)
                .unwrap_or(defaults.no_matching_options),
            more_actions: raw
                .more_actions
                .map(Into::into)
                .unwrap_or(defaults.more_actions),
            app_bar_actions: raw
                .app_bar_actions
                .map(Into::into)
                .unwrap_or(defaults.app_bar_actions),
            bottom_app_bar: raw
                .bottom_app_bar
                .map(Into::into)
                .unwrap_or(defaults.bottom_app_bar),
            breadcrumb: raw
                .breadcrumb
                .map(Into::into)
                .unwrap_or(defaults.breadcrumb),
            more_breadcrumbs: raw
                .more_breadcrumbs
                .map(Into::into)
                .unwrap_or(defaults.more_breadcrumbs),
            navigation: raw
                .navigation
                .map(Into::into)
                .unwrap_or(defaults.navigation),
            steps: raw.steps.map(Into::into).unwrap_or(defaults.steps),
            refresh: raw.refresh.map(Into::into).unwrap_or(defaults.refresh),
            resize_bottom_sheet: raw
                .resize_bottom_sheet
                .map(Into::into)
                .unwrap_or(defaults.resize_bottom_sheet),
            expand: raw.expand.map(Into::into).unwrap_or(defaults.expand),
            collapse: raw.collapse.map(Into::into).unwrap_or(defaults.collapse),
            drop_target: raw
                .drop_target
                .map(Into::into)
                .unwrap_or(defaults.drop_target),
            drop_files_here: raw
                .drop_files_here
                .map(Into::into)
                .unwrap_or(defaults.drop_files_here),
            searching: raw.searching.map(Into::into).unwrap_or(defaults.searching),
            no_results: raw
                .no_results
                .map(Into::into)
                .unwrap_or(defaults.no_results),
            search_prompt: raw
                .search_prompt
                .map(Into::into)
                .unwrap_or(defaults.search_prompt),
            minimize: raw.minimize.map(Into::into).unwrap_or(defaults.minimize),
            maximize: raw.maximize.map(Into::into).unwrap_or(defaults.maximize),
            restore: raw.restore.map(Into::into).unwrap_or(defaults.restore),
        })
    }
}
