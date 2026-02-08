use crate::component::Component;
use rudo_gc::Gc;

pub fn find_next_focusable(
    app_state: &impl crate::app::AppStateLike,
    current: &Gc<Component>,
    forward: bool,
) -> Option<Gc<Component>> {
    let focusables = collect_focusables(&app_state.root_component());

    if focusables.is_empty() {
        return None;
    }

    let current_idx = focusables.iter().position(|c| Gc::ptr_eq(c, current)).unwrap_or(0);

    let len = focusables.len();
    let next_idx = if forward { (current_idx + 1) % len } else { current_idx.saturating_sub(1) };

    Some(Gc::clone(&focusables[next_idx]))
}

fn collect_focusables(root: &Gc<Component>) -> Vec<Gc<Component>> {
    let mut result = Vec::new();
    collect_focusables_recursive(root, &mut result);
    result
}

fn collect_focusables_recursive(component: &Gc<Component>, result: &mut Vec<Gc<Component>>) {
    let flags = component.flags.read();
    if flags.contains(crate::event::status::ComponentFlags::ACCEPTS_FOCUS)
        && !flags.contains(crate::event::status::ComponentFlags::IS_DISABLED)
        && !flags.contains(crate::event::status::ComponentFlags::IS_STASHED)
    {
        result.push(Gc::clone(component));
    }

    for child in component.children.read().iter() {
        collect_focusables_recursive(child, result);
    }
}
