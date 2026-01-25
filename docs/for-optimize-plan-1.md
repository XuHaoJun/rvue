# Rvue Stable-key Based Node Pool Management 技術規格書

**版本**: 1.1  
**日期**: 2026-01-25  
**狀態**: 設計完成，待實施 (已修復 Review 問題)  

---

## 1. 執行摘要

### 1.1 問題描述

Rvue 目前的 `For` 組件依賴組件級別的增刪操作，當列表變更時會觸發全量重建。這違反了核心設計文檔中確立的「Setup Once」模式和細粒度更新哲學。

### 1.2 解決方案概述

借鑑 Leptos 的 `Keyed` 實現，引入 **Stable-key Based Node Pool Management**：
- 使用 `IndexSet` 維護 key → 節點映射
- 實現高效的 diff 算法計算最小操作集
- 利用 `move_in_dom` 優化避免不必要的渲染操作

### 1.3 核心決策

| 選項 | 決定 |
|-----|-----|
| API 風格 | **Leptos 風格** (`For::new(each, key, children)`) |
| 實現策略 | **簡化版本** (IndexSet diff + 精確節點增刪，暫無 Fragment Pool) |
| 測試策略 | **針對性測試** (只測新優化功能) |
| Key 約束 | `K: Eq + Hash + Clone + 'static` |
| 回收機制 | 直接 drop，由 rudo-gc 處理 |

---

## 2. 技術背景

### 2.1 核心設計哲學回顧

根據 `docs/2026-01-17_17-30-40_Gemini_Google_Gemini.md` 確立的設計原則：

> **Widget 定義是靜態的，狀態更新是動態的**
> - 使用宏定義靜態結構，在 setup 時轉換為 Taffy 佈局節點
> - 如果是 Signal，宏會生成細粒度的 Listener，直接修改 Vello Scene
> - 不採用 Flutter 的「重建樹」模式，而是「Retained Mode Widget Graph with Fine-Grained Updates」

### 2.2 現有問題分析

**當前 `For` 實現 (`widgets/for_loop.rs`)**：

```rust
pub struct For {
    item_count: ReactiveValue<usize>,  // ❌ 只支持 item_count，無 keys
}
```

**問題**：
1. 無法追蹤哪些具體 item 新增/刪除/移動
2. `render_children` 遍歷所有 children 重建
3. 違反「Setup Once」原則

### 2.3 Leptos 參考實現

**核心數據結構** (`tachys/src/view/keyed.rs`):

```rust
pub struct KeyedState<K, VFS, V> {
    parent: Option<Element>,
    marker: Placeholder,
    hashed_items: IndexSet<K, BuildHasherDefault<FxHasher>>,  // key → index
    rendered_items: Vec<Option<(VFS, V::State)>>,             // 渲染狀態
}
```

**Diff 算法特點**：
- O(n) 複雜度，使用 IndexSet 實現 O(1) 鍵查找
- `move_in_dom` 標誌智能判斷是否需要實際 DOM 操作
- `group_adjacent_moves()` 合併相鄰移動操作

---

## 3. 系統設計

### 3.1 核心數據結構

#### 3.1.1 `KeyedState` 結構

```rust
// crates/rvue/src/widgets/keyed_state.rs

use indexmap::IndexSet;
use rustc_hash::FxHasher;
use std::hash::{BuildHasherDefault, Hash};
use rudo_gc::{Gc, Trace};

pub trait Effect: Trace + 'static {
    fn unsubscribe(&self);
}

pub struct KeyedState<K, T, VFS> 
where
    K: Eq + Hash + Clone + 'static,
{
    // 父組件引用
    parent: Option<Gc<Component>>,
    
    // 標記節點（用於定位插入點）
    marker: Gc<Component>,
    
    // Stable-key 核心：保持順序的 O(1) 查找
    hashed_items: IndexSet<K, BuildHasherDefault<FxHasher>>,
    
    // 渲染狀態：index → (children_fn, component)
    rendered_items: Vec<Option<ItemEntry<K, T, VFS>>>,
}

pub struct ItemEntry<K, T, VFS> {
    key: K,
    item: T,
    children_fn: VFS,
    component: Gc<Component>,
    effect_handles: Vec<Gc<dyn Effect>>,
}

impl<K, T, VFS> ItemEntry<K, T, VFS> {
    fn unmount(&mut self) {
        for handle in &self.effect_handles {
            handle.unsubscribe();
        }
        self.effect_handles.clear();
        self.component.unmount();
    }

    fn prepare_for_move(&mut self) {}

    fn finalize_move(&mut self) {}
}
```

#### 3.1.2 Diff 操作結構

```rust
#[derive(Debug, Default)]
pub struct KeyedDiff<K> {
    removed: Vec<DiffOpRemove>,
    moved: Vec<DiffOpMove<K>>,
    added: Vec<DiffOpAdd<K>>,
    clear: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct DiffOpMove<K> {
    from: usize,
    len: usize,
    to: usize,
    move_in_dom: bool,  // 關鍵優化標誌
    key: K,
}

#[derive(Debug)]
pub struct DiffOpRemove {
    at: usize,
}

#[derive(Debug)]
pub struct DiffOpAdd<K> {
    at: usize,
    key: K,
    mode: DiffOpAddMode,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum DiffOpAddMode {
    #[default]
    Normal,
    Append,
}
```

### 3.2 Diff 算法

#### 3.2.1 核心 `diff_keys()` 函數

```rust
/// 計算從 old_keys 到 new_keys 所需的操作
pub fn diff_keys<K: Eq + Hash + Clone>(
    old_keys: &IndexSet<K, BuildHasherDefault<FxHasher>>,
    new_keys: &IndexSet<K, BuildHasherDefault<FxHasher>>,
) -> KeyedDiff<K> {
    // 快速路徑
    if old_keys.is_empty() && new_keys.is_empty() {
        return KeyedDiff::default();
    }
    if new_keys.is_empty() {
        return KeyedDiff { clear: true, ..Default::default() };
    }
    if old_keys.is_empty() {
        return KeyedDiff {
            added: new_keys.iter()
                .enumerate()
                .map(|(at, k)| DiffOpAdd { at, key: k.clone(), mode: DiffOpAddMode::Append })
                .collect(),
            ..Default::default()
        };
    }

    let mut removed = Vec::new();
    let mut moved = Vec::new();
    let mut added = Vec::new();

    // Collect remove positions first, then sort descending to avoid cascading issues
    // Example: [A, B, C, D] removing A, B, C → positions [0, 1, 2]
    // Sorted descending: [2, 1, 0] → removing in this order keeps indices valid
    let mut remove_positions: Vec<usize> = old_keys.iter()
        .enumerate()
        .filter(|(pos, old_key)| !new_keys.contains(old_key))
        .map(|(pos, _)| pos)
        .collect();

    remove_positions.sort_unstable_by(|a, b| b.cmp(a));  // Descending order

    for pos in remove_positions {
        removed.push(DiffOpRemove { at: pos });
    }

    for (pos, new_key) in new_keys.iter().enumerate() {
        if !old_keys.contains(new_key) {
            added.push(DiffOpAdd {
                at: pos,
                key: new_key.clone(),
                mode: DiffOpAddMode::Normal,
            });
        }
    }

    let old_keys_vec: Vec<_> = old_keys.iter().cloned().collect();
    for (pos, old_key) in old_keys_vec.iter().enumerate() {
        if !new_keys.contains(old_key) {
            continue;
        }

        if let Some((new_index, _)) = new_keys.get_full(old_key) {
            let removed_before = removed.iter().filter(|r| r.at < pos).count();
            let expected_without_additions = pos.saturating_sub(removed_before);
            let added_before = added.iter().filter(|a| a.at <= expected_without_additions).count();
            let expected = expected_without_additions + added_before;
            let actual = new_index;

            if expected != actual {
                let moves_forward_by = (actual as i32) - (expected as i32);
                let net_offset = (added_before as i32) - (removed_before as i32);
                let move_in_dom = moves_forward_by != net_offset;

                moved.push(DiffOpMove {
                    from: pos,       // FIXED: original position in old_keys
                    len: 1,
                    to: actual,      // FIXED: calculated position in new_keys
                    move_in_dom,
                    key: old_key.clone(),
                });
            }
        }
    }

    let moved = group_adjacent_moves(moved);

    KeyedDiff { removed, moved, added, clear: false }
}
```

#### 3.2.2 `move_in_dom` 優化邏輯

關鍵洞察：如果元素因為其他元素的插入/刪除而「被動移動」，則不需要實際 DOM 操作。

演算法核心：
- `expected` = 該元素在考慮「之前所有移除」後應該在的位置
- `actual` = 該元素在新列表中的實際位置
- 如果 `expected == actual`，表示位移是「被動」的（被插入/刪除推動），`move_in_dom = false`
- 如果 `expected != actual`，表示是「主動」移動，`move_in_dom = true`

**Vello 兼容性注意**：Vello 使用 retained mode scene graph。`move_in_dom = true` 仍需要更新 scene graph 結構。
對於 GPU 渲染上下文，可考慮 `move_in_dom = true` 作為所有 move 操作的保守估計，
或驗證 Vello 是否支持廉價的節點重新定位。DOM 渲染器可直接跳過 `move_in_dom = false` 的操作。

```
舊: [A, B, C, D]
新: [X, A, B, C, D]  (在前面插入 X)

對於 A:
- pos = 0, removed_before = 0
- expected_without_additions = 0 - 0 = 0
- added_before = 1 (X at index 0, and 0 <= 0)
- expected = 0 + 1 = 1
- actual = 1 (A's new position)
- expected == actual → move_in_dom = false

結論：A 的位置變化是由 X 的插入「推動」的，不需要額外 DOM 操作
```

#### 3.2.3 `group_adjacent_moves()` 合併優化

```rust
/// 將連續的移動操作合併為單一區塊
/// 例如：移動 [2,3,5,6] → [1,2,3,4,5,6] 會產生兩個區塊：(2,3) 和 (5,6)
fn group_adjacent_moves<K: Clone>(mut moves: Vec<DiffOpMove<K>>) -> Vec<DiffOpMove<K>> {
    if moves.is_empty() {
        return moves;
    }

    moves.sort_unstable_by(|a, b| {
        a.from.cmp(&b.from)
    });

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
```

### 3.3 Apply Diff 執行順序

執行順序經過精心設計，避免操作衝突：

```rust
fn apply_diff<K, T, VFS, V>(
    parent: Option<&Gc<Component>>,
    marker: &Gc<Component>,
    diff: KeyedDiff<K>,
    children: &mut Vec<Option<ItemEntry<K, T, VFS>>>,
    children_fn: impl Fn(usize, T) -> (VFS, V),
    mut items: Vec<Option<T>>,
) where K: Eq + Hash + Clone, V: Widget {
    if diff.clear {
        for child in children.iter_mut() {
            if let Some(entry) = child.take() {
                entry.unmount();
            }
        }
        children.clear();
        return;
    }

    let mut move_entries: Vec<(usize, ItemEntry<K, T, VFS>)> = Vec::new();

    for op in &diff.moved {
        if let Some(entry) = children[op.from].take() {
            move_entries.push((op.from, entry));
        }
    }

    for op in diff.removed.into_iter().rev() {
        if let Some(entry) = children[op.at].take() {
            entry.unmount();
        }
        children.remove(op.at);
    }

    for op in move_entries.into_iter() {
        children.insert(op.0, Some(op.1));
    }

    for op in diff.added {
        let item = items.get(op.at).and_then(|i| i.clone()).unwrap_or_default();
        let (vfs, _) = children_fn(op.at, item);
        let component = Gc::new(Component::new());
        children.insert(op.at, Some(ItemEntry {
            key: op.key,
            item,
            children_fn: vfs,
            component,
            effect_handles: Vec::new(),
        }));
    }
}
```

**關鍵設計**：在計算 diff 時使用 `from` 索引（舊位置），但在執行時：
1. 先按 `from` 順序收集所有待移動的 entries
2. 先處理 removals（按降序避免索引偏移）
3. 再按新位置插入 move entries

---

## 4. API 設計

### 4.1 新的 `For` 組件 API

```rust
// crates/rvue/src/widgets/for_loop.rs

use crate::prelude::*;
use std::hash::Hash;

pub struct For<T, I, K, KF, VF> {
    items: ReactiveValue<I>,
    key_fn: KF,
    children_fn: VF,
    _marker: std::marker::PhantomData<(T, K)>,
}

impl<T, I, K, KF, VF> For<T, I, K, KF, VF>
where
    T: Clone + 'static,
    I: IntoIterator<Item = T> + Clone + 'static,
    K: Eq + Hash + Clone + 'static,
    KF: Fn(&T) -> K + Clone + 'static,
    VF: Fn(T) -> ViewStruct + Clone + 'static,
{
    pub fn new(
        items: impl IntoReactiveValue<I>,
        key_fn: KF,
        children_fn: VF,
    ) -> Self {
        let items_reactive = items.into_reactive();

        if let Some(iter) = items_reactive.peek() {
            validate_no_duplicate_keys(&iter, &key_fn);
        }

        Self {
            items: items_reactive,
            key_fn,
            children_fn,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn validate_keys(&self) {
        if let Some(iter) = self.items.peek() {
            validate_no_duplicate_keys(&iter, &self.key_fn);
        }
    }
}

/// Validates that all keys are unique, warning on duplicates.
/// Duplicate keys cause undefined behavior where only one item will render.
fn validate_no_duplicate_keys<T, I, K, KF>(items: &I, key_fn: &KF)
where
    KF: Fn(&T) -> K,
    K: Hash,
{
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    for item in items {
        if !seen.insert(key_fn(item)) {
            tracing::warn!(
                "Duplicate key detected in For component. \
                 This may cause undefined behavior (only one item will render)."
            );
        }
    }
}
```

### 4.2 使用示例

```rust
#[derive(Clone)]
struct Item {
    id: i32,
    name: String,
}

fn create_item_list_view() -> ViewStruct {
    let items = create_signal(vec![
        Item { id: 1, name: "Alice".to_string() },
        Item { id: 2, name: "Bob".to_string() },
        Item { id: 3, name: "Charlie".to_string() },
    ]);

    view! {
        <Flex direction="column" gap=10.0>
            <Text content="Item List:" />
            <For
                each={items}
                key={|item| item.id}
                children={|item|
                    view! { <Text content={item.name.clone()} /> }
                }
            />
        </Flex>
    }
}
```

### 4.3 與現有系統整合

#### 4.3.1 擴展 `ComponentType`

```rust
// crates/rvue/src/component.rs

#[derive(Clone, PartialEq)]
pub enum ComponentType {
    // ... 現有類型 ...
    Keyed,  // 新增
}
```

#### 4.3.2 擴展 `ComponentProps`

```rust
#[derive(Clone)]
pub enum ComponentProps {
    // ... 現有類型 ...
    Keyed {
        item_count: usize,
    },
}
```

---

## 5. 依賴與配置

### 5.1 新增依賴

```toml
# crates/rvue/Cargo.toml

[dependencies]
# 新增
indexmap = { version = "2", features = ["std"] }
rustc-hash = "2"

# 現有保持不變
taffy = "0.6"
vello = "0.5"
rudo-gc = { path = "../../learn-projects/rudo/crates/rudo-gc" }
```

### 5.2 文件結構

```
crates/rvue/src/
├── widgets/
│   ├── mod.rs              # 添加 keyed_state 導出
│   ├── for_loop.rs         # 重構支持 stable-key
│   ├── keyed_state.rs      # 新建：KeyedState + diff 算法
│   └── ...
├── component.rs            # 添加 Keyed 類型
└── render/
    └── widget.rs           # 支持 key_order 排序渲染
```

---

## 6. 實施計劃

### Phase 1: 基礎設施 (預估 2 小時)

| 任務 | 描述 | 文件 |
|-----|------|------|
| 添加依賴 | `indexmap = "2"`, `rustc-hash = "2"` | `crates/rvue/Cargo.toml` |
| 創建模塊 | 新建 keyed_state 模塊 | `widgets/keyed_state.rs` |
| 定義類型 | `KeyedState`, `KeyedDiff`, 操作結構 | `keyed_state.rs` |

### Phase 2: Diff 算法 (預估 3 小時)

| 任務 | 描述 | 文件 |
|-----|------|------|
| `diff_keys()` | 實現 IndexSet 差異計算 | `keyed_state.rs` |
| `apply_diff()` | 7 步執行順序邏輯 | `keyed_state.rs` |
| `group_adjacent_moves()` | 相鄰移動合併優化 | `keyed_state.rs` |
| 單元測試 | 核心場景測試 | `keyed_state.rs` (tests) |

### Phase 3: For 組件重構 (預估 3 小時)

| 任務 | 描述 | 文件 |
|-----|------|------|
| 重構 `For` | 支持泛型 items + key_fn + children_fn | `widgets/for_loop.rs` |
| 更新 `ComponentType` | 添加 `Keyed` 變體 | `component.rs` |
| 更新渲染 | 支持 key_order 順序渲染 | `render/widget.rs` |

### Phase 4: 整合測試 (預估 2 小時)

| 任務 | 描述 | 文件 |
|-----|------|------|
| 示例更新 | 更新 list example 為 keyed list demo | `examples/list/src/main.rs` |
| 基本測試 | 列表增刪移動測試 | `tests/` |

---

## 7. 測試策略

### 7.1 必須覆蓋的場景

```rust
#[cfg(test)]
mod keyed_diff_tests {
    use super::*;

    #[test]
    fn test_insert_at_beginning() {
        // [A, B, C] → [X, A, B, C]
        let old = indexset!["A", "B", "C"];
        let new = indexset!["X", "A", "B", "C"];
        let diff = diff_keys(&old, &new);
        
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.removed.len(), 0);
        // A, B, C 應該 move_in_dom = false
        for m in &diff.moved {
            assert!(!m.move_in_dom);
        }
    }

    #[test]
    fn test_insert_at_end() {
        // [A, B, C] → [A, B, C, D]
        let old = indexset!["A", "B", "C"];
        let new = indexset!["A", "B", "C", "D"];
        let diff = diff_keys(&old, &new);
        
        assert_eq!(diff.added.len(), 1);
        assert!(diff.moved.is_empty()); // 無移動
    }

    #[test]
    fn test_remove_from_middle() {
        // [A, B, C, D] → [A, C, D]
        let old = indexset!["A", "B", "C", "D"];
        let new = indexset!["A", "C", "D"];
        let diff = diff_keys(&old, &new);
        
        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.removed[0].at, 1);
    }

    #[test]
    fn test_swap_items() {
        let old = indexset!["A", "B", "C"];
        let new = indexset!["C", "B", "A"];
        let diff = diff_keys(&old, &new);

        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert_eq!(diff.moved.len(), 2, "A and C should be moved (B stays)");
        assert!(diff.moved.iter().any(|m| m.key == "A"));
        assert!(diff.moved.iter().any(|m| m.key == "C"));
    }

    #[test]
    fn test_clear_list() {
        // [A, B, C] → []
        let old = indexset!["A", "B", "C"];
        let new = IndexSet::default();
        let diff = diff_keys(&old, &new);

        assert!(diff.clear);
    }

    #[test]
    fn test_shrink_from_beginning() {
        // [A, B, C, D] → [C, D]
        // C and D should NOT be marked as moved - their position changes
        // are due to A and B being removed, not actual DOM moves
        let old = indexset!["A", "B", "C", "D"];
        let new = indexset!["C", "D"];
        let diff = diff_keys(&old, &new);

        assert_eq!(diff.added.len(), 0);
        assert_eq!(diff.removed.len(), 2);
        assert_eq!(diff.removed[0].at, 0);
        assert_eq!(diff.removed[1].at, 1);
        assert!(diff.moved.is_empty(), "C and D should not be moved since their position change is due to removals");
    }

    #[test]
    fn test_shrink_from_beginning_single_remaining() {
        // [A, B, C] → [C]
        let old = indexset!["A", "B", "C"];
        let new = indexset!["C"];
        let diff = diff_keys(&old, &new);

        assert_eq!(diff.removed.len(), 2);
        assert_eq!(diff.removed[0].at, 0);
        assert_eq!(diff.removed[1].at, 1);
        assert!(diff.moved.is_empty(), "C should not be marked as moved");
    }

    #[test]
    fn test_shrink_from_middle_then_grow() {
        // [A, B, C, D, E] → [A, E]
        // A stays, E moves forward, B/C/D removed
        let old = indexset!["A", "B", "C", "D", "E"];
        let new = indexset!["A", "E"];
        let diff = diff_keys(&old, &new);

        assert_eq!(diff.removed.len(), 3);
        assert_eq!(diff.removed[0].at, 1);
        assert_eq!(diff.removed[1].at, 2);
        assert_eq!(diff.removed[2].at, 3);
        assert!(diff.moved.is_empty(), "A stays, E's shift is due to removals");
    }

    #[test]
    fn test_actual_move_after_shrink() {
        // [A, B, C, D] → [D, A, B, C]
        // D actually moves (not just shifted by removals)
        let old = indexset!["A", "B", "C", "D"];
        let new = indexset!["D", "A", "B", "C"];
        let diff = diff_keys(&old, &new);

        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert!(!diff.moved.is_empty(), "Should have at least D's move");
        assert!(diff.moved.iter().any(|m| m.key == "D"), "D should be in moved");
    }
}
```

### 7.2 GC 壓力測試 (可選)

```rust
#[test]
fn test_gc_pressure() {
    // 連續快速增刪 10000 個 items
    let items = create_signal(Vec::<i32>::new());
    
    for i in 0..10000 {
        items.update(|v| v.push(i));
    }
    
    for i in (0..10000).rev() {
        items.update(|v| v.remove(i));
    }
    
    // 驗證 GC 正常回收
    rudo_gc::force_gc();
}
```

---

## 8. 風險與緩解措施

| 風險 | 嚴重程度 | 緩解措施 |
|-----|---------|---------|
| `indexmap` 性能開銷 | 低 | 使用 `FxHasher` 減少哈希計算 |
| Vello append-only 限制 | 中 | 通過 Taffy 佈局順序控制渲染，不改變 Scene |
| Key 衝突導致 visual bugs | 中 | 添加重複 key 檢測（初始化 + `validate_keys()`）|
| 與現有代碼衝突 | 低 | 保持原有 API 兼容，新增泛型版本 |

---

## 9. 參考資料

### 9.1 Leptos 相關文件

- `/home/noah/Desktop/rvue/learn-projects/leptos/tachys/src/view/keyed.rs` - 核心 diff 算法
- `/home/noah/Desktop/rvue/learn-projects/leptos/leptos/src/for_loop.rs` - For 組件 API
- `/home/noah/Desktop/rvue/learn-projects/leptos/leptos_hot_reload/src/diff.rs` - hot reload diff

### 9.2 設計文檔

- `/home/noah/Desktop/rvue/docs/2026-01-17_17-30-40_Gemini_Google_Gemini.md` - 核心設計哲學
- `/home/noah/Desktop/rvue/docs/mvp-review-3.md` - MVP 評審，提出優化需求

---

## 10. 附錄：專家共識

基於 Alex Crichton, Leptos Team, 尤雨溪, Ryan Carniato 的「平行世界協作」討論：

### Key 的 `'static` 約束 (Alex Crichton)

> 「在 GC 環境下，Key 必須滿足 `Eq + Hash + Clone + 'static`。如果不是 'static，閉包的生命週期會傳染，導致整個 For 組件無法 Send/Sync。這是 Rust + GC 混合系統的安全邊界，不能妥協。」

### 回收機制 (Leptos Team)

> 「簡化版本直接 drop 就好，rudo-gc 會在下一次 GC 時回收內存。不要自己維護回收池，GC 就是乾這個的。Fragment Pool 是當 create_fragment 的成本很高時才需要考慮的。」

### 實現策略 (Ryan Carniato)

> 「關鍵的漸進式目標：
> 1. 先證明 stable key 能減少 Effect 數量——這是『編譯時靜態檢查』的核心價值
> 2. 再證明 diff 算法比全量重建快——用 Vello 的 render timing 說話
> 3. 最後再考慮 DOM/Vello 操作的合併優化——這是 Cherry on top。」

---

**文檔結束**
