# Component conventions

Check the state and callback contract for the public type you choose. The [component catalog](../../../../docs/component.md) lists supported variants, and [State and theme](../../../../docs/architecture.md#state-and-theme) describes the ownership boundaries.

| Family | Contract | Example |
| --- | --- | --- |
| Commands | Build the control in `render`. The application handles activation and owns the outcome. | `Button::new("save", "Save").on_activate(...)` |
| Controlled selection | Pass the current value into the control. Apply the callback's requested value to the owner and call `cx.notify()` when the rendering changes. | `Switch::new(id, label, checked).on_change(...)` |
| Editors and range inputs | Create the state entity in the owning view and pass a reference to the control on each render. Subscribe to the component's events for work triggered by edits. | `TextField::new(id, label, &state)` and `Slider::new(id, label, &state)` |
| Filtered options | Keep stable option IDs across filtering and reordering. The dropdown entity retains editing and navigation state. Apply the requested selection in the application. | `ExposedDropdown::new(id, label, &state, options).on_select(...)` |
| Dialogs and modal sheets | Retain a `DialogState` per modal. Open and close it through the entity to coordinate focus with the modal lifecycle. | `Dialog::new(id, &state, title)` and `Sheet::new(id, &state, title, content)` |
| Snackbar feedback | Mount a retained `SnackbarHost` in the layout. Queue `Snackbar` values on that entity. | `host.update(cx, |host, cx| host.push(message, window, cx))` |

Use semantic IDs for repeated controls and options so state and focus can follow an item when its position changes. For long collections and custom virtualization, follow the full-model navigation and reveal contract described in [Downstream integration](../../../../docs/integration.md#api-and-migration).

Use semantic variants, shape, and size setters where the chosen type exposes them. Use Material theme roles when composing adjacent content. Look up variant-specific constraints and defaults in the [design sources](../../../../docs/design-sources.md) and [pure theme guide](../../../../docs/theme/theme.md).
