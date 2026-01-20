use crate::component::Component;
use rudo_gc::Gc;

pub fn get_component_path(
    root: &Gc<Component>,
    target: Option<Gc<Component>>,
) -> Vec<Gc<Component>> {
    match target {
        Some(target) => {
            let mut path = Vec::new();
            let mut current = Some(target);
            while let Some(component) = current {
                path.push(Gc::clone(&component));
                current = component.parent.borrow().clone();
            }
            path
        }
        None => Vec::new(),
    }
}

pub fn merge_state_up(component: &Gc<Component>) {
    if let Some(parent) = component.parent.borrow().clone() {
        let parent_has_hovered = *parent.has_hovered.borrow() || *component.has_hovered.borrow();
        let parent_has_active = *parent.has_active.borrow() || *component.has_active.borrow();
        let parent_has_focus_target =
            *parent.has_focus_target.borrow() || *component.has_focus_target.borrow();

        *parent.has_hovered.borrow_mut() = parent_has_hovered;
        *parent.has_active.borrow_mut() = parent_has_active;
        *parent.has_focus_target.borrow_mut() = parent_has_focus_target;
    }
}

pub fn path_equals(a: &[Gc<Component>], b: &[Gc<Component>]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).all(|(a, b)| Gc::ptr_eq(a, b))
}
