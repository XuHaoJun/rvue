use crate::component::Component;
use rudo_gc::Gc;
use vello::kurbo::{Point, Rect, Size};

pub fn hit_test(root: &Gc<Component>, point: Point) -> Option<Gc<Component>> {
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
