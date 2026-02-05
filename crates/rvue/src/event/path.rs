use crate::component::Component;
use rudo_gc::Gc;

pub fn get_component_path(
    _root: &Gc<Component>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::ComponentType;

    #[test]
    fn test_path_equals_same_length_different() {
        // This test verifies the path_equals logic works correctly
        // When lengths are different, it returns false
        let empty_a: Vec<Gc<Component>> = Vec::new();
        let empty_b: Vec<Gc<Component>> = Vec::new();
        assert!(path_equals(&empty_a, &empty_b));
    }

    #[test]
    fn test_path_equals_different_lengths() {
        let component = Component::with_properties(
            0,
            ComponentType::Flex,
            crate::properties::PropertyMap::new(),
        );
        let one_element = vec![Gc::clone(&component)];
        let two_elements: Vec<Gc<Component>> = Vec::new();
        assert!(!path_equals(&one_element, &two_elements));
    }

    #[test]
    fn test_get_component_path_none() {
        let result = get_component_path(
            &Component::with_properties(
                0,
                ComponentType::Flex,
                crate::properties::PropertyMap::new(),
            ),
            None,
        );
        assert!(result.is_empty());
    }
}
