use crate::component::Component;
use indexmap::IndexSet;
use rudo_gc::{Gc, Trace};
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;
use std::hash::Hash;

#[derive(Clone)]
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
    let max_len = std::cmp::max(old_keys.len(), new_keys.len());

    for index in 0..max_len {
        let old_item = old_keys.get_index(index);
        let new_item = new_keys.get_index(index);

        if old_item != new_item {
            if let Some(old) = old_item {
                if !new_keys.contains(old) {
                    removed.push(DiffOpRemove { at: index });
                }
            }

            if let Some(new) = new_item {
                if !old_keys.contains(new) {
                    added.push(DiffOpAdd {
                        at: index,
                        key: new.clone(),
                        mode: DiffOpAddMode::Normal,
                    });
                }
            }

            if let Some(old) = old_item {
                if let Some((new_index, _)) = new_keys.get_full(old) {
                    let moves_forward_by = (new_index as i32) - (index as i32);
                    let net_offset = (added.len() as i32) - (removed.len() as i32);
                    let move_in_dom = moves_forward_by != net_offset;

                    moved.push(DiffOpMove {
                        from: index,
                        len: 1,
                        to: new_index,
                        move_in_dom,
                        key: old.clone(),
                    });
                }
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
        rendered_items.clear();
        return;
    }

    for op in &diff.removed {
        if op.at < rendered_items.len() {
            rendered_items[op.at].take();
        }
    }

    let needed_size = new_items_needed(&diff.added) + rendered_items.len() - diff.removed.len();
    if needed_size > rendered_items.len() {
        rendered_items.resize_with(needed_size, || None);
    }

    for op in &diff.moved {
        if op.move_in_dom {
            for i in 0..op.len {
                let from_idx = op.from + i;
                let to_idx = op.to + i;
                if from_idx < rendered_items.len() && to_idx < rendered_items.len() {
                    rendered_items.swap(from_idx, to_idx);
                }
            }
        }
    }
}

fn new_items_needed<K>(added: &[DiffOpAdd<K>]) -> usize {
    let mut max_at = 0;
    for op in added {
        if op.mode == DiffOpAddMode::Normal && op.at > max_at {
            max_at = op.at;
        }
    }
    max_at + 1
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
        assert!(!diff.moved.is_empty());
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
