use crate::component::{Component, ComponentType};
use rudo_gc::Gc;
use vello::kurbo::{Point, Rect, Size};

pub fn hit_test(root: &Gc<Component>, point: Point) -> Option<Gc<Component>> {
    eprintln!("[DEBUG-HIT-TOP] hit_test called with point=({:.1}, {:.1})", point.x, point.y);
    hit_test_recursive(root, point, Point::new(0.0, 0.0))
}

fn hit_test_recursive(
    component: &Gc<Component>,
    point: Point,
    global_offset: Point,
) -> Option<Gc<Component>> {
    let layout_result = component.layout_node.borrow().as_ref().and_then(|node| node.layout_result);
    let layout = layout_result?;

    let local_size = Size::new(layout.size.width as f64, layout.size.height as f64);
    let local_origin = Point::new(layout.location.x as f64, layout.location.y as f64);

    let global_origin =
        Point::new(global_offset.x + local_origin.x, global_offset.y + local_origin.y);
    let global_bounds = Rect::from_origin_size(global_origin, local_size);

    let contains = global_bounds.contains(point);

    if contains && component.component_type == ComponentType::Button {
        eprintln!("[DEBUG-HIT-BUTTON] BUTTON HIT! id={}, bounds=({:.1},{:.1},{:.1},{:.1}), point=({:.1},{:.1})",
            component.id,
            global_bounds.x0, global_bounds.y0, global_bounds.x1, global_bounds.y1,
            point.x, point.y);
    }

    eprintln!("[DEBUG-HIT] component_type={:?}, component_id={}, children_count={}, global_offset=({:.1},{:.1}), local_origin=({:.1},{:.1}), global_origin=({:.1},{:.1}), size=({:.1},{:.1}), point=({:.1},{:.1}), bounds=({:.1},{:.1},{:.1},{:.1}), contains={}",
        component.component_type,
        component.id,
        component.children.borrow().len(),
        global_offset.x, global_offset.y,
        local_origin.x, local_origin.y,
        global_origin.x, global_origin.y,
        local_size.width, local_size.height,
        point.x, point.y,
        global_bounds.x0, global_bounds.y0, global_bounds.x1, global_bounds.y1,
        contains);

    if !contains {
        return None;
    }

    let new_global_offset =
        Point::new(global_offset.x + local_origin.x, global_offset.y + local_origin.y);
    for child in component.children.borrow().iter().rev() {
        if let Some(hit) = hit_test_recursive(child, point, new_global_offset) {
            return Some(hit);
        }
    }

    if component.accepts_pointer_interaction() {
        Some(Gc::clone(component))
    } else {
        None
    }
}
