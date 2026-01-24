use crate::component::Component;
use rudo_gc::Gc;
use vello::kurbo::{Affine, Point, Rect, Size};

pub fn hit_test(root: &Gc<Component>, point: Point) -> Option<Gc<Component>> {
    hit_test_recursive(root, point, Affine::IDENTITY)
}

fn hit_test_recursive(
    component: &Gc<Component>,
    point: Point,
    parent_transform: Affine,
) -> Option<Gc<Component>> {
    let layout_result = component.layout_node.borrow().as_ref().and_then(|node| node.layout_result);

    let layout = layout_result?;

    let local_origin = Point::new(layout.location.x as f64, layout.location.y as f64);
    let local_size = Size::new(layout.size.width as f64, layout.size.height as f64);
    let transform = parent_transform * Affine::translate(local_origin.to_vec2());
    let bounds = Rect::from_origin_size(Point::ZERO, local_size);

    let local_point = transform.inverse() * point;
    if !bounds.contains(local_point) {
        return None;
    }

    for child in component.children.borrow().iter().rev() {
        if let Some(hit) = hit_test_recursive(child, point, transform) {
            return Some(hit);
        }
    }

    if component.accepts_pointer_interaction() {
        Some(Gc::clone(component))
    } else {
        None
    }
}
