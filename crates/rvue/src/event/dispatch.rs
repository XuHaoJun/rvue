use crate::component::{Component, ComponentType};
use crate::event::context::EventContext;
use crate::event::focus::find_next_focusable;
use crate::event::hit_test::hit_test;
use crate::event::path::merge_state_up;
use crate::event::types::{KeyState, PointerEvent, ScrollDelta, TextEvent};
use crate::style::get_inline_styles;
use rudo_gc::Gc;
use winit::keyboard::{Key, NamedKey};

pub fn find_scroll_container(component: &Gc<Component>) -> Option<Gc<Component>> {
    let mut parent = component.parent.borrow().clone();
    while let Some(p) = parent {
        if matches!(p.component_type, ComponentType::Flex) {
            let inline_styles = get_inline_styles(&p);
            let overflow_x = inline_styles
                .as_ref()
                .and_then(|s| s.overflow_x)
                .unwrap_or(rvue_style::properties::Overflow::Visible);
            let overflow_y = inline_styles
                .as_ref()
                .and_then(|s| s.overflow_y)
                .unwrap_or(rvue_style::properties::Overflow::Visible);

            if overflow_x.should_clip() || overflow_y.should_clip() {
                return Some(Gc::clone(&p));
            }
        }
        parent = p.parent.borrow().clone();
    }
    None
}

/// Find any scroll container in the component tree (for scroll events with no target)
fn find_any_scroll_container(root: &Gc<Component>) -> Option<Gc<Component>> {
    fn find_recursive(comp: &Gc<Component>) -> Option<Gc<Component>> {
        if matches!(comp.component_type, ComponentType::Flex) {
            let inline_styles = get_inline_styles(comp);
            let overflow_x = inline_styles
                .as_ref()
                .and_then(|s| s.overflow_x)
                .unwrap_or(rvue_style::properties::Overflow::Visible);
            let overflow_y = inline_styles
                .as_ref()
                .and_then(|s| s.overflow_y)
                .unwrap_or(rvue_style::properties::Overflow::Visible);

            if overflow_x.should_clip() || overflow_y.should_clip() {
                return Some(Gc::clone(comp));
            }
        }

        for child in comp.children.borrow().iter() {
            if let Some(found) = find_recursive(child) {
                return Some(found);
            }
        }

        None
    }

    find_recursive(root)
}

/// Find a component by its ID in the tree
fn find_component_by_id(root: &Gc<Component>, id: u64) -> Option<Gc<Component>> {
    if root.id == id {
        return Some(Gc::clone(root));
    }
    for child in root.children.borrow().iter() {
        if let Some(found) = find_component_by_id(child, id) {
            return Some(found);
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handled {
    Yes,
    No,
}

pub fn run_pointer_event_pass(
    app_state: &mut (impl crate::app::AppStateLike + crate::event::context::EventContextOps),
    event: &PointerEvent,
) -> Handled {
    // Handle scroll drag operations first
    if let Some(drag_state) = app_state.scroll_drag_state() {
        match event {
            PointerEvent::Move(e) => {
                let current_pos = if drag_state.is_vertical { e.position.y } else { e.position.x };
                let new_offset = drag_state.calculate_new_offset(current_pos);

                // Find the component and update its scroll state
                let root = app_state.root_component();
                if let Some(component) = find_component_by_id(&root, drag_state.component_id) {
                    let mut scroll_state = component.scroll_state();
                    if drag_state.is_vertical {
                        scroll_state.scroll_offset_y = new_offset as f32;
                    } else {
                        scroll_state.scroll_offset_x = new_offset as f32;
                    }
                    component.set_scroll_state(scroll_state);
                    component.mark_dirty();
                }
                return Handled::Yes;
            }
            PointerEvent::Up(_) | PointerEvent::Cancel(_) => {
                app_state.set_scroll_drag_state(None);
                return Handled::Yes;
            }
            _ => {}
        }
    }

    // Ensure state is up to date before dispatching
    if app_state.needs_pointer_pass_update() {
        crate::event::update::run_update_pointer_pass(app_state);
        app_state.set_needs_pointer_pass_update(false);
    }

    let target = get_pointer_target(app_state);

    // Special handling for scroll events: if no target found, find scroll container
    if target.is_none() && matches!(event, PointerEvent::Scroll(_)) {
        let root = app_state.root_component();
        if let Some(container) = find_any_scroll_container(&root) {
            return dispatch_pointer_event(app_state, &container, event);
        }
        return Handled::No;
    }

    if let Some(target) = target {
        let result = dispatch_pointer_event(app_state, &target, event);

        if matches!(event, PointerEvent::Down(_) | PointerEvent::Up(_) | PointerEvent::Move(_)) {
            app_state.set_needs_pointer_pass_update(true);
        }

        if matches!(event, PointerEvent::Up(_) | PointerEvent::Cancel(_)) {
            app_state.clear_pointer_capture();
        }

        result
    } else {
        Handled::No
    }
}

fn get_pointer_target(app_state: &impl crate::app::AppStateLike) -> Option<Gc<Component>> {
    if let Some(captured) = app_state.pointer_capture() {
        return Some(Gc::clone(&captured));
    }

    if let Some(pos) = app_state.last_pointer_pos() {
        let result = hit_test(&app_state.root_component(), pos);
        return result;
    }

    None
}

fn dispatch_pointer_event(
    app_state: &mut (impl crate::app::AppStateLike + crate::event::context::EventContextOps),
    target: &Gc<Component>,
    event: &PointerEvent,
) -> Handled {
    let mut current = Some(Gc::clone(target));
    let mut handled = Handled::No;

    while let Some(component) = current {
        if component.is_disabled() {
            current = component.parent.borrow().clone();
            continue;
        }

        let capture_clone = app_state.pointer_capture_mut().clone();
        let mut ctx = EventContext::new(Gc::clone(&component), app_state, capture_clone);

        let handlers = component.event_handlers.borrow();
        match event {
            PointerEvent::Down(e) => {
                if let Some(handler) = handlers.get_pointer_down() {
                    handler.call(e, &mut ctx);
                }

                if matches!(
                    component.component_type,
                    ComponentType::TextInput | ComponentType::NumberInput
                ) {
                    ctx.request_focus();
                }

                if handlers.get_click().is_some() {
                    ctx.capture_pointer();
                }
            }
            PointerEvent::Up(e) => {
                if let Some(handler) = handlers.get_pointer_up() {
                    handler.call(e, &mut ctx);
                }

                if let Some(handler) = handlers.get_click() {
                    if *component.is_active.borrow() {
                        handler.call(e, &mut ctx);
                    }
                }
            }
            PointerEvent::Move(e) => {
                if let Some(handler) = handlers.get_pointer_move() {
                    handler.call(e, &mut ctx);
                }
            }
            PointerEvent::Enter(e) => {
                if let Some(handler) = handlers.get_pointer_enter() {
                    handler.call(e, &mut ctx);
                }
            }
            PointerEvent::Leave(e) => {
                if let Some(handler) = handlers.get_pointer_leave() {
                    handler.call(e, &mut ctx);
                }
            }
            PointerEvent::Scroll(e) => {
                let delta_y = match e.delta {
                    ScrollDelta::Line(lines) => (lines * 20.0) as f32,
                    ScrollDelta::Pixel(_, dy) => (dy * 20.0) as f32,
                };
                let delta_x = match e.delta {
                    ScrollDelta::Line(_) => 0.0,
                    ScrollDelta::Pixel(dx, _) => (dx * 20.0) as f32,
                };

                let scroll_state = component.scroll_state();
                let can_scroll_y = scroll_state.scroll_height > 0.0;
                let can_scroll_x = scroll_state.scroll_width > 0.0;

                if can_scroll_y || can_scroll_x {
                    let mut new_state = scroll_state;
                    if can_scroll_y {
                        new_state.scroll_offset_y = (new_state.scroll_offset_y - delta_y)
                            .clamp(0.0, scroll_state.scroll_height.max(0.0));
                    }
                    if can_scroll_x {
                        new_state.scroll_offset_x = (new_state.scroll_offset_x + delta_x)
                            .clamp(0.0, scroll_state.scroll_width.max(0.0));
                    }
                    component.set_scroll_state(new_state);
                    component.mark_dirty();

                    if let Some(parent) = component.parent.borrow().clone() {
                        parent.mark_dirty();
                    }

                    handled = Handled::Yes;
                    ctx.stop_propagation();
                    break;
                }

                if let Some(scroll_container) = find_scroll_container(&component) {
                    let container_scroll_state = scroll_container.scroll_state();
                    let container_can_scroll_y = container_scroll_state.scroll_height > 0.0;
                    let container_can_scroll_x = container_scroll_state.scroll_width > 0.0;

                    if container_can_scroll_y || container_can_scroll_x {
                        let mut new_state = container_scroll_state;
                        if container_can_scroll_y {
                            new_state.scroll_offset_y = (new_state.scroll_offset_y - delta_y)
                                .clamp(0.0, container_scroll_state.scroll_height.max(0.0));
                        }
                        if container_can_scroll_x {
                            new_state.scroll_offset_x = (new_state.scroll_offset_x + delta_x)
                                .clamp(0.0, container_scroll_state.scroll_width.max(0.0));
                        }
                        scroll_container.set_scroll_state(new_state);
                        scroll_container.mark_dirty();

                        if let Some(parent) = scroll_container.parent.borrow().clone() {
                            parent.mark_dirty();
                        }

                        handled = Handled::Yes;
                        ctx.stop_propagation();
                        break;
                    }
                }

                if let Some(handler) = handlers.get_scroll() {
                    handler.call(e, &mut ctx);
                }
            }
            _ => {}
        }

        if ctx.is_handled() {
            handled = Handled::Yes;
            break;
        }

        merge_state_up(&component);
        current = component.parent.borrow().clone();
    }

    handled
}

pub fn run_text_event_pass(
    app_state: &mut (impl crate::app::AppStateLike + crate::event::context::EventContextOps),
    event: &TextEvent,
) -> Handled {
    // Check focused first, then pending_focus (which might have been set by a recent click)
    let target = app_state
        .focused()
        .clone()
        .or_else(|| app_state.pending_focus().clone())
        .or_else(|| app_state.fallback().clone());

    if let Some(target) = target {
        if let TextEvent::Keyboard(key_event) = event {
            if key_event.key == Key::Named(NamedKey::Tab) && key_event.state == KeyState::Down {
                let forward = !key_event.modifiers.shift;
                if let Some(next) = find_next_focusable(app_state, &target, forward) {
                    *app_state.pending_focus() = Some(next);
                    return Handled::Yes;
                }
            }
        }

        dispatch_text_event(app_state, &target, event)
    } else {
        Handled::No
    }
}

fn dispatch_text_event(
    app_state: &mut (impl crate::app::AppStateLike + crate::event::context::EventContextOps),
    target: &Gc<Component>,
    event: &TextEvent,
) -> Handled {
    let mut current = Some(Gc::clone(target));
    let mut handled = Handled::No;

    while let Some(component) = current {
        if component.is_disabled() {
            current = component.parent.borrow().clone();
            continue;
        }

        let capture_clone = app_state.pointer_capture_mut().clone();
        let mut ctx = EventContext::new(Gc::clone(&component), app_state, capture_clone);

        let handlers = component.event_handlers.borrow();
        match event {
            TextEvent::Keyboard(e) => {
                let is_text_input = matches!(component.component_type, ComponentType::TextInput);

                if is_text_input {
                    handle_text_input_keyboard_event(&component, e, &mut ctx);
                }

                if !ctx.is_handled() {
                    match e.state {
                        KeyState::Down => {
                            if let Some(handler) = handlers.get_key_down() {
                                handler.call(e, &mut ctx);
                            }
                        }
                        KeyState::Up => {
                            if let Some(handler) = handlers.get_key_up() {
                                handler.call(e, &mut ctx);
                            }
                        }
                    }
                }
            }
            TextEvent::Ime(e) => {
                if matches!(component.component_type, ComponentType::TextInput) {
                    handle_ime_event(&component, e, &mut ctx);
                }
            }
            TextEvent::Paste(text) => {
                if matches!(component.component_type, ComponentType::TextInput) {
                    if let Some(editor) = component.text_editor() {
                        editor.editor().insert_text(text);
                        component.reset_cursor_blink();
                        component.mark_dirty();
                        update_text_input_value(&component);
                        ctx.stop_propagation();
                    }
                }
            }
        }

        if ctx.is_handled() {
            handled = Handled::Yes;
            break;
        }

        merge_state_up(&component);
        current = component.parent.borrow().clone();
    }

    handled
}

fn handle_ime_event(
    component: &Gc<Component>,
    event: &crate::event::types::ImeEvent,
    ctx: &mut EventContext,
) {
    if let Some(editor) = component.text_editor() {
        match event {
            crate::event::types::ImeEvent::Preedit(text, cursor_range) => {
                if text.is_empty() {
                    editor.editor().clear_composition();
                } else {
                    editor.editor().set_composition(text, *cursor_range);
                }
                component.reset_cursor_blink();
                component.mark_dirty();
                ctx.stop_propagation();
            }
            crate::event::types::ImeEvent::Commit(text) => {
                editor.editor().insert_text(text);
                component.reset_cursor_blink();
                component.mark_dirty();
                update_text_input_value(component);
                ctx.stop_propagation();
            }
            crate::event::types::ImeEvent::Disabled => {
                editor.editor().clear_composition();
                component.mark_dirty();
                ctx.stop_propagation();
            }
            crate::event::types::ImeEvent::Enabled(_) => {
                ctx.stop_propagation();
            }
        }
    }
}

fn handle_text_input_keyboard_event(
    component: &Gc<Component>,
    event: &crate::event::types::KeyboardEvent,
    ctx: &mut EventContext,
) {
    if event.state != KeyState::Down {
        return;
    }

    if let Some(editor) = component.text_editor() {
        let text_editor = editor.editor();

        match &event.key {
            Key::Character(ch) => {
                if !event.modifiers.alt && !event.modifiers.ctrl && !event.modifiers.logo {
                    text_editor.insert_text(ch);
                    component.reset_cursor_blink();
                    component.mark_dirty();
                    update_text_input_value(component);
                    ctx.stop_propagation();
                }
            }
            Key::Named(NamedKey::Backspace) => {
                text_editor.backspace();
                component.reset_cursor_blink();
                component.mark_dirty();
                update_text_input_value(component);
                ctx.stop_propagation();
            }
            Key::Named(NamedKey::Delete) => {
                text_editor.delete();
                component.reset_cursor_blink();
                component.mark_dirty();
                update_text_input_value(component);
                ctx.stop_propagation();
            }
            Key::Named(NamedKey::Enter) => {
                ctx.stop_propagation();
            }
            Key::Named(NamedKey::ArrowLeft) => {
                text_editor.move_cursor(-1);
                component.reset_cursor_blink();
                component.mark_dirty();
                ctx.stop_propagation();
            }
            Key::Named(NamedKey::ArrowRight) => {
                text_editor.move_cursor(1);
                component.reset_cursor_blink();
                component.mark_dirty();
                ctx.stop_propagation();
            }
            Key::Named(NamedKey::ArrowUp) => {
                text_editor.move_to_start();
                component.reset_cursor_blink();
                component.mark_dirty();
                ctx.stop_propagation();
            }
            Key::Named(NamedKey::ArrowDown) => {
                text_editor.move_to_end();
                component.reset_cursor_blink();
                component.mark_dirty();
                ctx.stop_propagation();
            }
            Key::Named(NamedKey::Home) => {
                text_editor.move_to_start();
                component.reset_cursor_blink();
                component.mark_dirty();
                ctx.stop_propagation();
            }
            Key::Named(NamedKey::End) => {
                text_editor.move_to_end();
                component.reset_cursor_blink();
                component.mark_dirty();
                ctx.stop_propagation();
            }
            Key::Named(NamedKey::Tab) => {
                ctx.stop_propagation();
            }
            _ => {}
        }
    }
}

fn update_text_input_value(component: &Gc<Component>) {
    if let Some(editor) = component.text_editor() {
        let content = editor.editor().content();
        component.set_text_input_value(content);
    }
}

pub fn update_cursor_blink_states(component: &Gc<Component>, interval_ms: u64) {
    let is_focused = *component.is_focused.borrow();

    if let Some(blink) = component.cursor_blink() {
        if blink.update(interval_ms, is_focused) {
            component.mark_dirty();
        }
    }

    for child in component.children.borrow().iter() {
        update_cursor_blink_states(child, interval_ms);
    }
}
