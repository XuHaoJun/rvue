use crate::component::{Component, ComponentProps};
use crate::event::context::EventContext;
use crate::event::focus::find_next_focusable;
use crate::event::hit_test::hit_test;
use crate::event::path::merge_state_up;
use crate::event::types::{KeyState, PointerEvent, ScrollDelta, TextEvent};
use rudo_gc::Gc;
use winit::keyboard::{Key, NamedKey};

/// Find the nearest parent component that is a scroll container
fn find_scroll_container(component: &Gc<Component>) -> Option<Gc<Component>> {
    let mut parent = component.parent.borrow().clone();
    while let Some(p) = parent {
        let props = p.props.borrow();
        if let ComponentProps::Flex { styles, .. } = &*props {
            if let Some(computed_styles) = styles {
                let overflow_x =
                    computed_styles.overflow_x.unwrap_or(rvue_style::properties::Overflow::Visible);
                let overflow_y =
                    computed_styles.overflow_y.unwrap_or(rvue_style::properties::Overflow::Visible);
                if overflow_x.should_clip() || overflow_y.should_clip() {
                    return Some(Gc::clone(&p));
                }
            }
        }
        parent = p.parent.borrow().clone();
    }
    None
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
                    eprintln!("[DEBUG-SCROLL] Drag scroll - new_offset: {}", new_offset);
                }
                return Handled::Yes;
            }
            PointerEvent::Up(_) | PointerEvent::Cancel(_) => {
                eprintln!("[DEBUG-SCROLL] Ending scroll drag");
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
                // Calculate scroll delta (convert to f32)
                let delta_y = match e.delta {
                    ScrollDelta::Line(lines) => (lines * 20.0) as f32,
                    ScrollDelta::Pixel(_, dy) => (dy * 20.0) as f32,
                };
                let delta_x = match e.delta {
                    ScrollDelta::Line(_) => 0.0,
                    ScrollDelta::Pixel(dx, _) => (dx * 20.0) as f32,
                };

                // Try to scroll this component first
                let scroll_state = component.scroll_state();
                let can_scroll_y = scroll_state.scroll_height > 0.0;
                let can_scroll_x = scroll_state.scroll_width > 0.0;

                eprintln!("[DEBUG-SCROLL] PointerScrollEvent - Component ID: {:?}", component.id);
                eprintln!("[DEBUG-SCROLL]   Current scroll_state - offset_x: {}, offset_y: {}, scroll_w: {}, scroll_h: {}, container_w: {}, container_h: {}",
                    scroll_state.scroll_offset_x, scroll_state.scroll_offset_y,
                    scroll_state.scroll_width, scroll_state.scroll_height,
                    scroll_state.container_width, scroll_state.container_height);
                eprintln!(
                    "[DEBUG-SCROLL]   can_scroll_x: {}, can_scroll_y: {}",
                    can_scroll_x, can_scroll_y
                );

                if can_scroll_y || can_scroll_x {
                    // Update scroll offset
                    let mut new_state = scroll_state;
                    if can_scroll_y {
                        new_state.scroll_offset_y = (new_state.scroll_offset_y + delta_y)
                            .clamp(0.0, scroll_state.scroll_height.max(0.0));
                    }
                    if can_scroll_x {
                        new_state.scroll_offset_x = (new_state.scroll_offset_x + delta_x)
                            .clamp(0.0, scroll_state.scroll_width.max(0.0));
                    }
                    eprintln!("[DEBUG-SCROLL]   Scroll applied to component - delta_x: {}, delta_y: {}, new_offset_x: {}, new_offset_y: {}",
                        delta_x, delta_y, new_state.scroll_offset_x, new_state.scroll_offset_y);
                    component.set_scroll_state(new_state);
                    eprintln!("[DEBUG-SCROLL]   Component marked dirty: {:?}", component.id);
                    component.mark_dirty();

                    // Mark as handled (stop propagation)
                    handled = Handled::Yes;
                    ctx.stop_propagation();
                    break;
                }

                // If this component can't scroll, try to delegate to parent scroll container
                if let Some(scroll_container) = find_scroll_container(&component) {
                    let container_scroll_state = scroll_container.scroll_state();
                    let container_can_scroll_y = container_scroll_state.scroll_height > 0.0;
                    let container_can_scroll_x = container_scroll_state.scroll_width > 0.0;

                    eprintln!(
                        "[DEBUG-SCROLL]   Delegating to parent scroll container ID: {:?}",
                        scroll_container.id
                    );
                    eprintln!("[DEBUG-SCROLL]   Container scroll_state - offset_x: {}, offset_y: {}, scroll_w: {}, scroll_h: {}",
                        container_scroll_state.scroll_offset_x, container_scroll_state.scroll_offset_y,
                        container_scroll_state.scroll_width, container_scroll_state.scroll_height);

                    if container_can_scroll_y || container_can_scroll_x {
                        // Update container scroll offset
                        let mut new_state = container_scroll_state;
                        if container_can_scroll_y {
                            new_state.scroll_offset_y = (new_state.scroll_offset_y + delta_y)
                                .clamp(0.0, container_scroll_state.scroll_height.max(0.0));
                        }
                        if container_can_scroll_x {
                            new_state.scroll_offset_x = (new_state.scroll_offset_x + delta_x)
                                .clamp(0.0, container_scroll_state.scroll_width.max(0.0));
                        }
                        eprintln!("[DEBUG-SCROLL]   Scroll applied to container - delta_x: {}, delta_y: {}, new_offset_x: {}, new_offset_y: {}",
                            delta_x, delta_y, new_state.scroll_offset_x, new_state.scroll_offset_y);
                        scroll_container.set_scroll_state(new_state);
                        eprintln!(
                            "[DEBUG-SCROLL]   Container marked dirty: {:?}",
                            scroll_container.id
                        );
                        scroll_container.mark_dirty();

                        // Mark as handled (stop propagation)
                        handled = Handled::Yes;
                        ctx.stop_propagation();
                        break;
                    }
                }

                // If not a scroll container, call on_scroll handler if exists
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
    let target = app_state.focused().clone().or_else(|| app_state.fallback().clone());

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
            TextEvent::Keyboard(e) => match e.state {
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
            },
            TextEvent::Ime(_) => {}
            TextEvent::Paste(_) => {}
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
