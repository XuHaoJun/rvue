use crate::component::Component;
use crate::style::get_inline_styles;
use rudo_gc::Gc;
use rvue_style::properties::Overflow;
use vello::kurbo::{Point, Rect, Size};

pub fn hit_test(root: &Gc<Component>, point: Point) -> Option<Gc<Component>> {
    hit_test_recursive(root, point, Point::new(0.0, 0.0))
}

fn get_overflow_for_component(component: &Gc<Component>) -> (Overflow, Overflow) {
    let inline_styles = get_inline_styles(component);
    let overflow_x = inline_styles.as_ref().and_then(|s| s.overflow_x).unwrap_or(Overflow::Visible);
    let overflow_y = inline_styles.as_ref().and_then(|s| s.overflow_y).unwrap_or(Overflow::Visible);
    (overflow_x, overflow_y)
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

    let (overflow_x, overflow_y) = get_overflow_for_component(component);
    let should_clip = overflow_x.should_clip() || overflow_y.should_clip();

    let scroll_offset = if should_clip {
        let scroll_state = component.scroll_state();
        Some(Point::new(scroll_state.scroll_offset_x as f64, scroll_state.scroll_offset_y as f64))
    } else {
        None
    };

    let visible_bounds = if should_clip {
        let scroll_state = component.scroll_state();
        let visible_width = scroll_state.container_width as f64;
        let visible_height = scroll_state.container_height as f64;
        let visible_x = global_origin.x;
        let visible_y = global_origin.y;
        Some(Rect::new(visible_x, visible_y, visible_x + visible_width, visible_y + visible_height))
    } else {
        None
    };

    let new_global_offset =
        Point::new(global_offset.x + local_origin.x, global_offset.y + local_origin.y);

    for child in component.children.borrow().iter().rev() {
        let adjusted_offset = if let Some(scroll) = scroll_offset {
            Point::new(new_global_offset.x - scroll.x, new_global_offset.y - scroll.y)
        } else {
            new_global_offset
        };

        if let Some(hit) = hit_test_recursive(child, point, adjusted_offset) {
            if let Some(bounds) = visible_bounds {
                let child_layout =
                    child.layout_node.borrow().as_ref().and_then(|n| n.layout_result);
                if let Some(child_layout) = child_layout {
                    let child_origin = Point::new(
                        adjusted_offset.x + child_layout.location.x as f64,
                        adjusted_offset.y + child_layout.location.y as f64,
                    );
                    let child_size =
                        Size::new(child_layout.size.width as f64, child_layout.size.height as f64);
                    let child_bounds = Rect::from_origin_size(child_origin, child_size);
                    if bounds.contains_rect(child_bounds)
                        || child_bounds.intersect(bounds).area() > 0.0
                    {
                        return Some(hit);
                    }
                } else {
                    return Some(hit);
                }
            } else {
                return Some(hit);
            }
        }
    }

    if component.accepts_pointer_interaction() {
        Some(Gc::clone(component))
    } else {
        None
    }
}
