use crate::component::Component;
use crate::event::path::get_component_path;
use crate::event::status::StatusUpdate;
use rudo_gc::Gc;

pub fn run_update_pointer_pass(app_state: &mut impl crate::app::AppStateLike) {
    let prev_active_path = std::mem::take(app_state.active_path());
    let prev_hovered_path = std::mem::take(app_state.hovered_path());

    let hovered_target = app_state.hovered_component().clone();
    let next_hovered_path = get_component_path(&app_state.root_component(), hovered_target);

    let capture_target = app_state.pointer_capture().clone();
    let is_capture_hovered = capture_target
        .as_ref()
        .map_or(false, |capture| next_hovered_path.iter().any(|h| Gc::ptr_eq(h, capture)));

    let next_active_path = if is_capture_hovered {
        get_component_path(&app_state.root_component(), capture_target)
    } else {
        Vec::new()
    };

    for widget in prev_active_path.iter().chain(next_active_path.iter()) {
        let widget_ptr = Gc::as_ptr(widget) as *const Component;
        let should_have_active =
            next_active_path.iter().any(|w| Gc::as_ptr(w) as *const Component == widget_ptr);
        let has_active = *widget.is_active.borrow();

        if should_have_active != has_active {
            *widget.is_active.borrow_mut() = should_have_active;
            let cloned = Gc::clone(widget);
            cloned.on_status_update(&StatusUpdate::ActiveChanged(should_have_active));
        }
    }

    let prev_active_leaf = prev_active_path.first().map(|p| Gc::as_ptr(p) as *const Component);
    let next_active_leaf = next_active_path.first().map(|p| Gc::as_ptr(p) as *const Component);

    if prev_active_leaf != next_active_leaf {
        if let Some(prev) = prev_active_path.first() {
            *prev.is_active.borrow_mut() = false;
            let cloned = Gc::clone(prev);
            cloned.on_status_update(&StatusUpdate::ActiveChanged(false));
        }
        if let Some(next) = next_active_path.first() {
            *next.is_active.borrow_mut() = true;
            let cloned = Gc::clone(next);
            cloned.on_status_update(&StatusUpdate::ActiveChanged(true));
        }
    }

    for widget in prev_hovered_path.iter().chain(next_hovered_path.iter()) {
        let widget_ptr = Gc::as_ptr(widget) as *const Component;
        let should_have_hovered =
            next_hovered_path.iter().any(|w| Gc::as_ptr(w) as *const Component == widget_ptr);
        let has_hovered = *widget.is_hovered.borrow();

        if should_have_hovered != has_hovered {
            *widget.is_hovered.borrow_mut() = should_have_hovered;
            let cloned = Gc::clone(widget);
            cloned.on_status_update(&StatusUpdate::HoveredChanged(should_have_hovered));
        }
    }

    let prev_hovered_leaf = prev_hovered_path.first().map(|p| Gc::as_ptr(p) as *const Component);
    let next_hovered_leaf = next_hovered_path.first().map(|p| Gc::as_ptr(p) as *const Component);

    if prev_hovered_leaf != next_hovered_leaf {
        if let Some(prev) = prev_hovered_path.first() {
            *prev.is_hovered.borrow_mut() = false;
            let cloned = Gc::clone(prev);
            cloned.on_status_update(&StatusUpdate::HoveredChanged(false));
        }
        if let Some(next) = next_hovered_path.first() {
            *next.is_hovered.borrow_mut() = true;
            let cloned = Gc::clone(next);
            cloned.on_status_update(&StatusUpdate::HoveredChanged(true));
        }
    }

    *app_state.active_path() = next_active_path;
    *app_state.hovered_path() = next_hovered_path;
}

pub fn run_update_focus_pass(app_state: &mut impl crate::app::AppStateLike) {
    if let Some(pending) = app_state.pending_focus().take() {
        let prev_focused = app_state.focused().clone();

        if let Some(prev) = prev_focused.as_ref() {
            *prev.is_focused.borrow_mut() = false;
            let cloned = Gc::clone(prev);
            cloned.on_status_update(&StatusUpdate::FocusChanged(false));
        }

        *pending.is_focused.borrow_mut() = true;
        pending.on_status_update(&StatusUpdate::FocusChanged(true));

        *app_state.focused_mut() = Some(pending);
    }

    let focused_target = app_state.focused().clone();
    let next_focused_path = get_component_path(&app_state.root_component(), focused_target);

    let prev_focused_path = std::mem::take(app_state.focused_path());

    for widget in prev_focused_path.iter().chain(next_focused_path.iter()) {
        let widget_ptr = Gc::as_ptr(widget) as *const Component;
        let should_have_focus =
            next_focused_path.iter().any(|w| Gc::as_ptr(w) as *const Component == widget_ptr);
        let has_focus_target = *widget.has_focus_target.borrow();

        if should_have_focus != has_focus_target {
            *widget.has_focus_target.borrow_mut() = should_have_focus;
            let cloned = Gc::clone(widget);
            cloned.on_status_update(&StatusUpdate::ChildFocusChanged(should_have_focus));
        }
    }

    *app_state.focused_path() = next_focused_path;
}
