use crate::component::Component;
use crate::event::context::EventContext;
use crate::event::focus::find_next_focusable;
use crate::event::hit_test::hit_test;
use crate::event::path::merge_state_up;
use crate::event::types::{KeyState, PointerEvent, TextEvent};
use rudo_gc::Gc;
use winit::keyboard::{Key, NamedKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handled {
    Yes,
    No,
}

pub fn run_pointer_event_pass(
    app_state: &mut (impl crate::app::AppStateLike + crate::event::context::EventContextOps),
    event: &PointerEvent,
) -> Handled {
    eprintln!("[DEBUG-PASS] run_pointer_event_pass ENTRY - event_type={:?}", event_type_str(event));
    eprintln!("[DEBUG-PASS] last_pointer_pos at entry: {:?}", app_state.last_pointer_pos());

    // Ensure state is up to date before dispatching
    if app_state.needs_pointer_pass_update() {
        eprintln!("[DEBUG-PASS] needs_pointer_pass_update=true, running update pass");
        crate::event::update::run_update_pointer_pass(app_state);
        app_state.set_needs_pointer_pass_update(false);
    }

    let target = get_pointer_target(app_state);
    let target_info = target.as_ref().map(|t| (t.id, format!("{:?}", t.component_type)));
    eprintln!(
        "[DEBUG-CLICK] run_pointer_event_pass - event_type={:?}, target={:?}",
        event_type_str(event),
        target_info
    );

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

fn event_type_str(event: &PointerEvent) -> &'static str {
    match event {
        PointerEvent::Down(_) => "Down",
        PointerEvent::Up(_) => "Up",
        PointerEvent::Move(_) => "Move",
        PointerEvent::Scroll(_) => "Scroll",
        PointerEvent::Enter(_) => "Enter",
        PointerEvent::Leave(_) => "Leave",
        PointerEvent::Cancel(_) => "Cancel",
    }
}

fn get_pointer_target(app_state: &impl crate::app::AppStateLike) -> Option<Gc<Component>> {
    eprintln!("[DEBUG-TARGET] get_pointer_target called");
    eprintln!("[DEBUG-TARGET] pointer_capture: {:?}", app_state.pointer_capture().map(|c| c.id));
    eprintln!("[DEBUG-TARGET] last_pointer_pos: {:?}", app_state.last_pointer_pos());

    if let Some(captured) = app_state.pointer_capture() {
        eprintln!("[DEBUG-TARGET] Using captured pointer target: id={}", captured.id);
        return Some(Gc::clone(&captured));
    }

    if let Some(pos) = app_state.last_pointer_pos() {
        eprintln!("[DEBUG-TARGET] Calling hit_test with pos=({:.1}, {:.1})", pos.x, pos.y);
        let result = hit_test(&app_state.root_component(), pos);
        eprintln!("[DEBUG-TARGET] hit_test result: {:?}", result.as_ref().map(|c| c.id));
        return result;
    }

    eprintln!("[DEBUG-TARGET] No pointer target found");
    None
}

fn dispatch_pointer_event(
    app_state: &mut (impl crate::app::AppStateLike + crate::event::context::EventContextOps),
    target: &Gc<Component>,
    event: &PointerEvent,
) -> Handled {
    eprintln!(
        "[DEBUG-CLICK] dispatch_pointer_event - target_id={}, target_type={:?}",
        target.id, target.component_type
    );

    let mut current = Some(Gc::clone(target));
    let mut handled = Handled::No;

    while let Some(component) = current {
        if component.is_disabled() {
            eprintln!(
                "[DEBUG-CLICK] component {:?} is disabled, skipping",
                component.component_type
            );
            current = component.parent.borrow().clone();
            continue;
        }

        let capture_clone = app_state.pointer_capture_mut().clone();
        let mut ctx = EventContext::new(Gc::clone(&component), app_state, capture_clone);

        let handlers = component.event_handlers.borrow();
        match event {
            PointerEvent::Down(e) => {
                eprintln!(
                    "[DEBUG-CLICK] Processing Down - component={:?}, has_handler={}",
                    component.component_type,
                    handlers.get_pointer_down().is_some()
                );
                if let Some(handler) = handlers.get_pointer_down() {
                    handler.call(e, &mut ctx);
                }

                if handlers.get_click().is_some() {
                    ctx.capture_pointer();
                    eprintln!("[DEBUG-CLICK] Click handler exists, captured pointer");
                }
            }
            PointerEvent::Up(e) => {
                eprintln!("[DEBUG-CLICK] Processing Up - component={:?}, has_click_handler={}, is_active={}",
                    component.component_type, handlers.get_click().is_some(), *component.is_active.borrow());
                if let Some(handler) = handlers.get_pointer_up() {
                    handler.call(e, &mut ctx);
                }

                if let Some(handler) = handlers.get_click() {
                    if *component.is_active.borrow() {
                        eprintln!("[DEBUG-CLICK] Calling click handler!");
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
