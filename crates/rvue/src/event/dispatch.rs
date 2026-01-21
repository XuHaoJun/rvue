use crate::component::Component;
use crate::event::context::EventContext;
use crate::event::focus::find_next_focusable;
use crate::event::hit_test::hit_test;
use crate::event::path::merge_state_up;
use crate::event::types::{KeyState, PointerButtonEvent, PointerEvent, TextEvent};
use rudo_gc::Gc;
use vello::kurbo::Point;
use winit::keyboard::{Key, ModifiersState, NamedKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handled {
    Yes,
    No,
}

pub fn run_pointer_event_pass(
    app_state: &mut (impl crate::app::AppStateLike + crate::event::context::EventContextOps),
    event: &PointerEvent,
) -> Handled {
    let target = get_pointer_target(app_state);

    if let Some(target) = target {
        let result = dispatch_pointer_event(app_state, &target, event);

        if matches!(event, PointerEvent::Down(_)) {
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
        return hit_test(&app_state.root_component(), pos);
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
        let capture_clone = app_state.pointer_capture_mut().clone();
        let mut ctx = EventContext::new(Gc::clone(&component), app_state, capture_clone);

        let handlers = component.event_handlers.borrow();
        match event {
            PointerEvent::Down(e) => {
                if let Some(handler) = handlers.get_pointer_down() {
                    handler.call(e, &mut ctx);
                }
            }
            PointerEvent::Up(e) => {
                if let Some(handler) = handlers.get_pointer_up() {
                    handler.call(e, &mut ctx);
                }
            }
            PointerEvent::Move(e) => {
                if let Some(handler) = handlers.get_pointer_move() {
                    handler.call(e, &mut ctx);
                }
            }
            PointerEvent::Enter(_) => {
                if let Some(handler) = handlers.on_pointer_enter.as_ref() {
                    handler.call(
                        &PointerButtonEvent {
                            button: crate::event::types::PointerButton::Primary,
                            position: Point::ZERO,
                            click_count: 0,
                            modifiers: ModifiersState::default().into(),
                        },
                        &mut ctx,
                    );
                }
            }
            PointerEvent::Leave(_) => {
                if let Some(handler) = handlers.on_pointer_leave.as_ref() {
                    handler.call(
                        &PointerButtonEvent {
                            button: crate::event::types::PointerButton::Primary,
                            position: Point::ZERO,
                            click_count: 0,
                            modifiers: ModifiersState::default().into(),
                        },
                        &mut ctx,
                    );
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
