//! For widget with stable-key based node pool management

use crate::component::{Component, ComponentLifecycle, ComponentType};
use crate::effect::create_effect;
use crate::properties::{ForItemCount, PropertyMap};
use crate::view::View;
use crate::widget::{
    with_current_ctx, BuildContext, IntoReactiveValue, Mountable, ReactiveValue, Widget,
};
use crate::widgets::keyed_state::{diff_keys, KeyedState};
use indexmap::IndexSet;
use log::warn;
use rudo_gc::handles::HandleScope;
use rudo_gc::heap::current_thread_control_block;
use rudo_gc::{Gc, GcCell, Trace};
use rustc_hash::FxHasher;
use std::hash::{BuildHasherDefault, Hash};

pub struct For<T, K, KF, VF>
where
    T: Clone + Trace + Send + Sync + 'static,
    K: Eq + Hash + Clone + 'static,
    KF: Fn(&T) -> K + Clone + 'static,
    VF: Fn(T) -> crate::ViewStruct + Clone + 'static,
{
    pub items: ReactiveValue<Vec<T>>,
    pub key_fn: KF,
    pub view_fn: VF,
}

impl<T, K, KF, VF> Clone for For<T, K, KF, VF>
where
    T: Clone + Trace + Send + Sync + 'static,
    K: Eq + Hash + Clone + 'static,
    KF: Fn(&T) -> K + Clone + 'static,
    VF: Fn(T) -> crate::ViewStruct + Clone + 'static,
{
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            key_fn: self.key_fn.clone(),
            view_fn: self.view_fn.clone(),
        }
    }
}

unsafe impl<T, K, KF, VF> Trace for For<T, K, KF, VF>
where
    T: Clone + Trace + Send + Sync + 'static,
    K: Eq + Hash + Clone + 'static,
    KF: Fn(&T) -> K + Clone + 'static,
    VF: Fn(T) -> crate::ViewStruct + Clone + 'static,
{
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.items.trace(visitor);
    }
}

impl<T, K, KF, VF> For<T, K, KF, VF>
where
    T: Clone + Trace + Send + Sync + 'static,
    K: Eq + Hash + Clone + 'static,
    KF: Fn(&T) -> K + Clone + 'static,
    VF: Fn(T) -> crate::ViewStruct + Clone + 'static,
{
    pub fn new(items: impl IntoReactiveValue<Vec<T>>, key_fn: KF, view_fn: VF) -> Self {
        Self { items: items.into_reactive(), key_fn, view_fn }
    }
}

pub struct ForState<T, K, KF, VF>
where
    T: Clone + Trace + 'static,
    K: Eq + Hash + Clone + 'static,
    KF: Fn(&T) -> K + Clone + 'static,
    VF: Fn(T) -> crate::ViewStruct + Clone + 'static,
{
    component: Gc<Component>,
    keyed_state: Gc<GcCell<KeyedState<K, T>>>,
    item_count_effect: Option<Gc<crate::effect::Effect>>,
    _phantom: std::marker::PhantomData<(KF, VF)>,
}

unsafe impl<T, K, KF, VF> Trace for ForState<T, K, KF, VF>
where
    T: Clone + Trace + 'static,
    K: Eq + Hash + Clone + 'static,
    KF: Fn(&T) -> K + Clone + 'static,
    VF: Fn(T) -> crate::ViewStruct + Clone + 'static,
{
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.component.trace(visitor);
        self.keyed_state.trace(visitor);
        if let Some(effect) = &self.item_count_effect {
            effect.trace(visitor);
        }
    }
}

impl<T, K, KF, VF> ForState<T, K, KF, VF>
where
    T: Clone + Trace + rudo_gc::cell::GcCapture + 'static,
    K: Eq + Hash + Clone + 'static,
    KF: Fn(&T) -> K + Clone + 'static,
    VF: Fn(T) -> crate::ViewStruct + Clone + 'static,
{
    pub fn component(&self) -> &Gc<Component> {
        &self.component
    }

    fn get_current_keys(&self) -> IndexSet<K, BuildHasherDefault<FxHasher>> {
        (*self.keyed_state).borrow().hashed_items.clone()
    }

    fn update_keyed_items(
        &self,
        new_items: Vec<T>,
        key_fn: &KF,
        view_fn: &VF,
        ctx: &mut BuildContext,
    ) -> IndexSet<K, BuildHasherDefault<FxHasher>> {
        let mut new_keys: IndexSet<K, BuildHasherDefault<FxHasher>> =
            IndexSet::with_hasher(Default::default());

        let mut seen_keys = std::collections::HashSet::new();
        for item in &new_items {
            let key = key_fn(item);
            if !seen_keys.insert(key.clone()) {
                warn!("Duplicate key found in For component - using first occurrence");
                continue;
            }
            new_keys.insert(key);
        }

        let old_keys = self.get_current_keys();
        let diff = diff_keys(&old_keys, &new_keys);

        {
            let keyed_state = self.keyed_state.borrow_mut();
            let mut rendered_items_mut = keyed_state.rendered_items.borrow_mut_gen_only();

            for op in &diff.removed {
                if op.at < rendered_items_mut.len() {
                    if let Some(entry) = rendered_items_mut[op.at].take() {
                        entry.component.unmount();
                    }
                }
            }

            if diff.clear {
                rendered_items_mut.clear();
            }

            if rendered_items_mut.len() < new_keys.len() {
                rendered_items_mut.resize_with(new_keys.len(), || None);
            }
        }

        {
            let mut keyed_state = self.keyed_state.borrow_mut();
            let mut rendered_items_mut = keyed_state.rendered_items.borrow_mut_gen_only();

            for op in &diff.added {
                if let Some(item) = new_items.get(op.at) {
                    let key = key_fn(item);
                    let view = with_current_ctx(ctx.id_counter, || view_fn(item.clone()));
                    let child_component = view.into_component();

                    while rendered_items_mut.len() <= op.at {
                        rendered_items_mut.push(None);
                    }

                    rendered_items_mut[op.at] = Some(crate::widgets::keyed_state::ItemEntry {
                        key: key.clone(),
                        item: item.clone(),
                        component: Gc::clone(&child_component),
                        mounted: false,
                    });
                    child_component.set_parent(Some(Gc::clone(&keyed_state.marker)));
                    child_component.mount(None);
                }
            }

            for op in &diff.moved {
                for i in 0..op.len {
                    let from_idx = op.from + i;
                    let to_idx = op.to + i;
                    if from_idx < rendered_items_mut.len() && to_idx < rendered_items_mut.len() {
                        let (adjusted_from_idx, adjusted_to_idx) = if op.move_in_dom {
                            (from_idx, to_idx)
                        } else {
                            let removed_before_from =
                                diff.removed.iter().filter(|r| r.at < from_idx).count();
                            let adjusted_from_idx = from_idx - removed_before_from;
                            let removed_before_to =
                                diff.removed.iter().filter(|r| r.at < to_idx).count();
                            let adjusted_to_idx = to_idx - removed_before_to;
                            (adjusted_from_idx, adjusted_to_idx)
                        };

                        if adjusted_from_idx < rendered_items_mut.len()
                            && adjusted_to_idx < rendered_items_mut.len()
                        {
                            rendered_items_mut.swap(adjusted_from_idx, adjusted_to_idx);
                        }
                    }
                }
            }

            reorder_children(&keyed_state.marker, &rendered_items_mut);

            drop(rendered_items_mut);
            keyed_state.hashed_items = new_keys.clone();
        }

        new_keys
    }
}

fn reorder_children<K, T>(
    parent: &Gc<Component>,
    rendered_items: &[Option<crate::widgets::keyed_state::ItemEntry<K, T>>],
) {
    let mut children = parent.children.borrow_mut_gen_only();
    children.clear();
    for item in rendered_items.iter().flatten() {
        children.push(Gc::clone(&item.component));
    }
}

impl<T, K, KF, VF> Mountable for ForState<T, K, KF, VF>
where
    T: Clone + Trace + 'static,
    K: Eq + Hash + Clone + 'static,
    KF: Fn(&T) -> K + Clone + 'static,
    VF: Fn(T) -> crate::ViewStruct + Clone + 'static,
{
    fn mount(&self, parent: Option<Gc<Component>>) {
        self.component.set_parent(parent.clone());
        if let Some(parent) = parent {
            parent.add_child(Gc::clone(&self.component));
        }
    }

    fn unmount(&self) {
        self.component.set_parent(None);
    }
}

impl<T, K, KF, VF> Widget for For<T, K, KF, VF>
where
    T: Clone + Trace + rudo_gc::cell::GcCapture + Send + Sync + 'static,
    K: Eq + Hash + Clone + 'static,
    KF: Fn(&T) -> K + Clone + 'static,
    VF: Fn(T) -> crate::ViewStruct + Clone + 'static,
{
    type State = ForState<T, K, KF, VF>;

    fn build(self, ctx: &mut BuildContext) -> Self::State {
        let id = crate::component::next_component_id();
        let initial_items = self.items.get();
        let initial_count = initial_items.len();
        let is_reactive = self.items.is_reactive();

        let properties = if is_reactive {
            PropertyMap::new()
        } else {
            PropertyMap::with(ForItemCount(initial_count))
        };

        let component = Component::with_properties(id, ComponentType::For, properties);

        let rendered_items_cell: GcCell<Vec<Option<crate::widgets::keyed_state::ItemEntry<K, T>>>> =
            GcCell::new(Vec::with_capacity(initial_items.len()));

        for item in initial_items.iter() {
            let key = (self.key_fn)(item);

            let view = with_current_ctx(ctx.id_counter, || (self.view_fn)(item.clone()));
            let child_component = view.into_component();

            let entry_component = Gc::clone(&child_component);
            {
                let mut rendered_items_mut = rendered_items_cell.borrow_mut();
                rendered_items_mut.push(Some(crate::widgets::keyed_state::ItemEntry {
                    key: key.clone(),
                    item: item.clone(),
                    component: entry_component,
                    mounted: false,
                }));
            }

            child_component.set_parent(Some(Gc::clone(&component)));
            child_component.mount(None);
        }

        let mut keyed_state = KeyedState {
            parent: None,
            marker: Gc::clone(&component),
            hashed_items: IndexSet::with_hasher(Default::default()),
            rendered_items: rendered_items_cell,
        };

        for item in initial_items.iter() {
            let key = (self.key_fn)(item);
            keyed_state.hashed_items.insert(key.clone());
        }

        reorder_children(&component, &keyed_state.rendered_items.borrow());

        let keyed_state_gc = GcCell::new(keyed_state);
        let tcb = current_thread_control_block().expect("GC not initialized");
        let scope = HandleScope::new(&tcb);

        let keyed_state_gc_gc = Gc::new(keyed_state_gc);
        let handle = scope.handle(&keyed_state_gc_gc);
        let keyed_state_gc_shared = handle.to_gc();
        let comp_clone = Gc::clone(&component);
        let key_fn_clone = self.key_fn;
        let view_fn_clone = self.view_fn;
        let items_reactive = self.items.clone();
        let ctx_id = *ctx.id_counter;
        let keyed_state_for_effect = Gc::clone(&keyed_state_gc_shared);

        let item_count_effect = if self.items.is_reactive() {
            let effect = create_effect(move || {
                let new_items = items_reactive.get();
                let new_count = new_items.len();
                {
                    let state = ForState {
                        component: Gc::clone(&comp_clone),
                        keyed_state: Gc::clone(&keyed_state_for_effect),
                        item_count_effect: None,
                        _phantom: std::marker::PhantomData,
                    };
                    let mut temp_taffy = taffy::TaffyTree::new();
                    let mut temp_text_context = crate::text::TextContext::new();
                    let mut temp_id_counter = ctx_id;
                    let mut temp_ctx = BuildContext::new(
                        &mut temp_taffy,
                        &mut temp_text_context,
                        &mut temp_id_counter,
                    );
                    let _new_keys = state.update_keyed_items(
                        new_items,
                        &key_fn_clone,
                        &view_fn_clone,
                        &mut temp_ctx,
                    );
                }
                comp_clone.properties.borrow_mut_gen_only().insert(ForItemCount(new_count));
                comp_clone.mark_dirty();
            });
            component.add_effect(Gc::clone(&effect));
            Some(effect)
        } else {
            None
        };

        ForState {
            component,
            keyed_state: keyed_state_gc_shared,
            item_count_effect,
            _phantom: std::marker::PhantomData,
        }
    }

    fn rebuild(self, state: &mut Self::State) {
        if self.items.is_reactive() {
            let comp = Gc::clone(&state.component);
            let key_fn = self.key_fn;
            let view_fn = self.view_fn;
            let items = self.items.clone();
            let keyed_state_gc_shared = Gc::clone(&state.keyed_state);
            let ctx_id = state.component.id;

            let effect = create_effect(move || {
                let new_items = items.get();
                let new_count = new_items.len();
                {
                    let state = ForState {
                        component: Gc::clone(&comp),
                        keyed_state: Gc::clone(&keyed_state_gc_shared),
                        item_count_effect: None,
                        _phantom: std::marker::PhantomData,
                    };
                    let mut temp_taffy = taffy::TaffyTree::new();
                    let mut temp_text_context = crate::text::TextContext::new();
                    let mut temp_id_counter = ctx_id;
                    let mut temp_ctx = BuildContext::new(
                        &mut temp_taffy,
                        &mut temp_text_context,
                        &mut temp_id_counter,
                    );
                    let _new_keys =
                        state.update_keyed_items(new_items, &key_fn, &view_fn, &mut temp_ctx);
                }
                comp.properties.borrow_mut_gen_only().insert(ForItemCount(new_count));
                comp.mark_dirty();
            });
            state.component.add_effect(Gc::clone(&effect));
            state.item_count_effect = Some(effect);
        } else {
            let new_items = self.items.get();
            let new_count = new_items.len();
            let mut temp_taffy = taffy::TaffyTree::new();
            let mut temp_text_context = crate::text::TextContext::new();
            let mut temp_id_counter = self.items.get().len() as u64 + 1000;
            let mut temp_ctx =
                BuildContext::new(&mut temp_taffy, &mut temp_text_context, &mut temp_id_counter);
            let _ = state.update_keyed_items(new_items, &self.key_fn, &self.view_fn, &mut temp_ctx);
            state.component.properties.borrow_mut_gen_only().insert(ForItemCount(new_count));
            state.component.mark_dirty();
        }
    }
}
