use crate::component::{Component, ComponentLifecycle};
use indexmap::IndexSet;
use rudo_gc::{Gc, Trace};
use rudo_gc_derive::GcCell;
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;
use std::hash::Hash;

#[derive(Clone, GcCell)]
pub struct KeyedState<K, T>
where
    K: Eq + Hash + Clone + 'static,
{
    pub parent: Option<Gc<Component>>,
    pub marker: Gc<Component>,
    pub hashed_items: IndexSet<K, BuildHasherDefault<FxHasher>>,
    pub rendered_items: Vec<Option<ItemEntry<K, T>>>,
}

unsafe impl<K, T> Trace for KeyedState<K, T>
where
    K: Eq + Hash + Clone + 'static,
    T: Trace + Clone + 'static,
{
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.parent.trace(visitor);
        self.marker.trace(visitor);
        self.rendered_items.trace(visitor);
    }
}

#[derive(Clone)]
pub struct ItemEntry<K, T> {
    pub key: K,
    pub item: T,
    pub component: Gc<Component>,
    pub mounted: bool,
}

impl<K, T> ItemEntry<K, T> {
    pub fn unmount(&mut self) {
        ComponentLifecycle::unmount(&*self.component);
    }

    pub fn prepare_for_move(&mut self) {}

    pub fn finalize_move(&mut self) {}
}

unsafe impl<K, T> Trace for ItemEntry<K, T>
where
    K: Eq + Hash + Clone + 'static,
    T: Trace + Clone + 'static,
{
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.item.trace(visitor);
        self.component.trace(visitor);
    }
}

#[derive(Debug, Clone)]
pub struct KeyedDiff<K> {
    pub removed: Vec<DiffOpRemove>,
    pub moved: Vec<DiffOpMove<K>>,
    pub added: Vec<DiffOpAdd<K>>,
    pub clear: bool,
}

impl<K> Default for KeyedDiff<K> {
    fn default() -> Self {
        Self { removed: Vec::new(), moved: Vec::new(), added: Vec::new(), clear: false }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DiffOpMove<K> {
    pub from: usize,
    pub len: usize,
    pub to: usize,
    pub move_in_dom: bool,
    pub key: K,
}

#[derive(Debug, Clone)]
pub struct DiffOpRemove {
    pub at: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiffOpAdd<K> {
    pub at: usize,
    pub key: K,
    pub mode: DiffOpAddMode,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum DiffOpAddMode {
    #[default]
    Normal,
    Append,
}

pub fn diff_keys<K: Eq + Hash + Clone>(
    old_keys: &IndexSet<K, BuildHasherDefault<FxHasher>>,
    new_keys: &IndexSet<K, BuildHasherDefault<FxHasher>>,
) -> KeyedDiff<K> {
    if old_keys.is_empty() && new_keys.is_empty() {
        return KeyedDiff::default();
    }
    if new_keys.is_empty() {
        return KeyedDiff { clear: true, ..KeyedDiff::default() };
    }
    if old_keys.is_empty() {
        return KeyedDiff {
            added: new_keys
                .iter()
                .enumerate()
                .map(|(at, k)| DiffOpAdd { at, key: k.clone(), mode: DiffOpAddMode::Append })
                .collect(),
            ..KeyedDiff::default()
        };
    }

    let mut removed = Vec::new();
    let mut moved = Vec::new();
    let mut added = Vec::new();

    for (pos, old_key) in old_keys.iter().enumerate() {
        if !new_keys.contains(old_key) {
            removed.push(DiffOpRemove { at: pos });
        }
    }
    removed.sort_by_key(|a| std::cmp::Reverse(a.at));

    for (pos, new_key) in new_keys.iter().enumerate() {
        if !old_keys.contains(new_key) {
            added.push(DiffOpAdd { at: pos, key: new_key.clone(), mode: DiffOpAddMode::Normal });
        }
    }

    let old_keys_vec: Vec<_> = old_keys.iter().cloned().collect();
    for (pos, old_key) in old_keys_vec.iter().enumerate() {
        if !new_keys.contains(old_key) {
            continue;
        }

        if let Some((new_index, _)) = new_keys.get_full(old_key) {
            let removed_before = removed.iter().filter(|r| r.at < pos).count();
            let removals_at_pos = removed.iter().filter(|r| r.at == pos).count();
            let expected_without_additions = pos.saturating_sub(removed_before + removals_at_pos);
            let added_before = added.iter().filter(|a| a.at <= expected_without_additions).count();
            let expected = expected_without_additions + added_before;
            let actual = new_index;

            if expected != actual {
                let removals_before_pos = removed_before + removals_at_pos;
                let insertions_before_expected = added.iter().filter(|a| a.at < expected).count();
                let move_in_dom =
                    (removals_before_pos as i32) != (insertions_before_expected as i32);

                let removed_before_at_pos = removed
                    .iter()
                    .filter(|r| r.at < pos || (r.at == pos && removals_at_pos > 0))
                    .count();
                let adjusted_from =
                    if removed_before_at_pos > 0 { pos - removed_before_at_pos } else { pos };

                moved.push(DiffOpMove {
                    from: adjusted_from,
                    len: 1,
                    to: actual,
                    move_in_dom,
                    key: old_key.clone(),
                });
            }
        }
    }

    let moved = group_adjacent_moves(moved);

    KeyedDiff { removed, moved, added, clear: false }
}

fn group_adjacent_moves<K: Clone>(moves: Vec<DiffOpMove<K>>) -> Vec<DiffOpMove<K>> {
    if moves.is_empty() {
        return moves;
    }

    let mut result = Vec::with_capacity(moves.len());
    let mut current = moves[0].clone();

    for m in moves.into_iter().skip(1) {
        if m.from == current.from + current.len && m.to == current.to + current.len {
            current.len += 1;
        } else {
            result.push(current);
            current = m;
        }
    }

    result.push(current);
    result
}

pub fn apply_diff<K, T>(
    _parent: Option<&Gc<Component>>,
    _marker: &Gc<Component>,
    diff: KeyedDiff<K>,
    rendered_items: &mut Vec<Option<ItemEntry<K, T>>>,
) where
    K: Eq + Hash + Clone,
{
    if diff.clear {
        // Clean up all items before clearing
        for entry in rendered_items.iter_mut().flatten() {
            entry.unmount();
        }
        rendered_items.clear();
        return;
    }

    // Step 2: Removals - unmount and remove nodes
    for op in &diff.removed {
        if op.at < rendered_items.len() {
            if let Some(mut entry) = rendered_items[op.at].take() {
                entry.unmount();
            }
        }
    }

    let needed_size = rendered_items.len() - diff.removed.len() + diff.added.len();
    if needed_size > rendered_items.len() {
        rendered_items.resize_with(needed_size, || None);
    }

    // Step 5: Move in - handle moves (for moves that don't require DOM operations)
    for op in &diff.moved {
        if op.move_in_dom {
            for i in 0..op.len {
                let from_idx = op.from + i;
                let to_idx = op.to + i;
                if from_idx < rendered_items.len() && to_idx < rendered_items.len() {
                    if let Some(e) = rendered_items[from_idx].as_mut() {
                        e.prepare_for_move();
                    }
                    rendered_items.swap(from_idx, to_idx);
                    if let Some(e) = rendered_items[to_idx].as_mut() {
                        e.finalize_move();
                    }
                }
            }
        } else {
            for i in 0..op.len {
                let from_idx = op.from + i;
                let to_idx = op.to + i;
                if from_idx < rendered_items.len() && to_idx < rendered_items.len() {
                    let removed_before_from =
                        diff.removed.iter().filter(|r| r.at < from_idx).count();
                    let adjusted_from_idx = from_idx - removed_before_from;
                    let removed_before_to = diff.removed.iter().filter(|r| r.at < to_idx).count();
                    let adjusted_to_idx = to_idx - removed_before_to;
                    if adjusted_from_idx < rendered_items.len()
                        && adjusted_to_idx < rendered_items.len()
                    {
                        if let Some(e) = rendered_items[adjusted_from_idx].as_mut() {
                            e.prepare_for_move();
                        }
                        rendered_items.swap(adjusted_from_idx, adjusted_to_idx);
                        if let Some(e) = rendered_items[adjusted_to_idx].as_mut() {
                            e.finalize_move();
                        }
                    }
                }
            }
        }
    }

    // Step 7: Remove holes - compact Vec by removing None entries
    rendered_items.retain(|entry| entry.is_some());
}

#[cfg(test)]
mod keyed_diff_tests {
    use super::*;

    fn make_set<K: Eq + Hash + Clone>(items: &[K]) -> IndexSet<K, BuildHasherDefault<FxHasher>> {
        let mut set: IndexSet<K, BuildHasherDefault<FxHasher>> =
            IndexSet::with_hasher(Default::default());
        for item in items {
            set.insert(item.clone());
        }
        set
    }

    #[test]
    fn test_insert_at_beginning() {
        let old = make_set(&["A", "B", "C"]);
        let new = make_set(&["X", "A", "B", "C"]);
        let diff = diff_keys(&old, &new);

        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.removed.len(), 0);
        for m in &diff.moved {
            assert!(!m.move_in_dom, "A, B, C should not move in DOM");
        }
    }

    #[test]
    fn test_insert_at_end() {
        let old = make_set(&["A", "B", "C"]);
        let new = make_set(&["A", "B", "C", "D"]);
        let diff = diff_keys(&old, &new);

        assert_eq!(diff.added.len(), 1);
        assert!(diff.moved.is_empty(), "No moves expected when appending");
    }

    #[test]
    fn test_remove_from_middle() {
        let old = make_set(&["A", "B", "C", "D"]);
        let new = make_set(&["A", "C", "D"]);
        let diff = diff_keys(&old, &new);

        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.removed[0].at, 1);
    }

    #[test]
    fn test_swap_items() {
        let old = make_set(&["A", "B", "C"]);
        let new = make_set(&["C", "B", "A"]);
        let diff = diff_keys(&old, &new);

        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert_eq!(diff.moved.len(), 2, "A and C should be moved (B stays)");
        assert!(diff.moved.iter().any(|m| m.key == "A"));
        assert!(diff.moved.iter().any(|m| m.key == "C"));
    }

    #[test]
    fn test_clear_list() {
        let old = make_set(&["A", "B", "C"]);
        let new: IndexSet<_, BuildHasherDefault<FxHasher>> =
            IndexSet::with_hasher(Default::default());
        let diff = diff_keys(&old, &new);

        assert!(diff.clear);
    }

    #[test]
    fn test_shrink_from_beginning() {
        let old = make_set(&["A", "B", "C", "D"]);
        let new = make_set(&["C", "D"]);
        let diff = diff_keys(&old, &new);

        assert_eq!(diff.added.len(), 0);
        assert_eq!(diff.removed.len(), 2);
        // Removals are sorted in descending order
        assert_eq!(diff.removed[0].at, 1);
        assert_eq!(diff.removed[1].at, 0);
        assert!(
            diff.moved.is_empty(),
            "C and D should not be moved since their position change is due to removals"
        );
    }

    #[test]
    fn test_shrink_from_beginning_single_remaining() {
        let old = make_set(&["A", "B", "C"]);
        let new = make_set(&["C"]);
        let diff = diff_keys(&old, &new);

        assert_eq!(diff.removed.len(), 2);
        // Removals are sorted in descending order
        assert_eq!(diff.removed[0].at, 1);
        assert_eq!(diff.removed[1].at, 0);
        assert!(diff.moved.is_empty(), "C should not be marked as moved");
    }

    #[test]
    fn test_shrink_from_middle_then_grow() {
        let old = make_set(&["A", "B", "C", "D", "E"]);
        let new = make_set(&["A", "E"]);
        let diff = diff_keys(&old, &new);

        assert_eq!(diff.removed.len(), 3);
        // Removals are sorted in descending order
        assert_eq!(diff.removed[0].at, 3);
        assert_eq!(diff.removed[1].at, 2);
        assert_eq!(diff.removed[2].at, 1);
        assert!(diff.moved.is_empty(), "A stays, E's shift is due to removals");
    }

    #[test]
    fn test_actual_move_after_shrink() {
        let old = make_set(&["A", "B", "C", "D"]);
        let new = make_set(&["D", "A", "B", "C"]);
        let diff = diff_keys(&old, &new);

        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        // D moves from index 3 to 0
        // A, B, C are "pushed" but don't need DOM moves (move_in_dom = false)
        // However, group_adjacent_moves might merge them with D's move
        assert!(!diff.moved.is_empty(), "Should have at least D's move");
        assert!(diff.moved.iter().any(|m| m.key == "D"), "D should be in moved");
    }

    #[test]
    fn test_move_index_adjusted_after_remove() {
        let old = make_set(&["A", "B", "C", "D"]);
        let new = make_set(&["A", "C", "D", "B"]);
        let diff = diff_keys(&old, &new);

        // B was at index 1, now at index 3
        // A stays at 0
        // C and D shift down by 1 due to B's removal (but don't move in DOM)
        assert!(diff.added.is_empty());
        // B is not removed - it's still in the list, just at a different position
        assert!(diff.removed.is_empty(), "B should not be removed - it moved");

        // B should be marked as moved
        let b_move = diff.moved.iter().find(|m| m.key == "B");
        assert!(b_move.is_some(), "B should be in moved list");
        if let Some(m) = b_move {
            // B moves from original index 1 to index 3
            // Since there's only one item before B (A at 0), and no removals before it
            // from should be 1
            assert_eq!(m.from, 1, "B's from should be its original index 1");
            assert_eq!(m.to, 3, "B's to position should be 3");
        }
    }

    #[test]
    fn test_removed_sorted_descending() {
        let old = make_set(&["A", "B", "C", "D", "E"]);
        let new = make_set(&["A", "E"]);
        let diff = diff_keys(&old, &new);

        assert_eq!(diff.removed.len(), 3);
        // Removals should be sorted in descending order
        assert!(diff.removed.windows(2).all(|w| w[0].at >= w[1].at));
    }

    #[test]
    fn test_group_adjacent_moves() {
        let moves = vec![
            DiffOpMove { from: 2, len: 1, to: 4, move_in_dom: true, key: "A" },
            DiffOpMove { from: 3, len: 1, to: 5, move_in_dom: true, key: "B" },
        ];
        let grouped = group_adjacent_moves(moves);
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].from, 2);
        assert_eq!(grouped[0].len, 2);
        assert_eq!(grouped[0].to, 4);
    }
}
