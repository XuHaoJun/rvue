# Rvue Vello Fragment 緩存技術設計文檔

**Date:** 2026-01-26  
**Author:** Rvue Core Team  
**Status:** Draft

---

## 1. 背景與問題陳述

### 1.1 當前架構分析

Rvue 當前採用 Vello 作為 GPU 加速渲染引擎，結合 rudo-gc 實現細粒度響應式更新。然而，在渲染層面存在顯著的性能瓶頸。

**當前渲染流程：**

```
Signal 更新 → Effect 運行 → 組件標記 dirty
    ↓
Scene::update() 被調用
    ↓
遍歷組件樹，對每個組件：
    ├─ 如果 dirty：重繪並清除子孫緩存 ⚠️
    └─ 如果不 dirty：跳過
```

**核心問題位於 widget.rs:88-91：**

```rust
// 當前行為：父組件更新時清除所有子孫的緩存
*child.vello_cache.borrow_mut() = None;
child.is_dirty.store(true, std::sync::atomic::Ordering::SeqCst);
render_component(child, scene, ...);
```

### 1.2 性能瓶頸診斷

| 症狀 | 根因 |
|------|------|
| 父組件狀態變化導致整個子樹重繪 | `vello_cache` 被父更新清除 |
| 1000+ 組件場景性能急劇下降 | 遞歸重繪而非增量合成 |
| Vello 優勢無法發揮 | 每次更新都重建大量 Scene 數據 |

**問題本質：**

```
當前：父更新 → 清除子孫緩存 → 強制重繪所有節點
期望：父更新 → 只重繪父 → 子孫直接 append 緩存
```

這違背了細粒度更新的核心價值。

---

## 2. 調研結論

### 2.1 Xilem/Masonry 分析

**項目路徑：** `/learn-projects/xilem/masonry/`

**核心發現：** Xilem 已經實現了 Fragment 緩存機制，採用簡潔的 HashMap 方案。

**關鍵代碼 (render_root.rs:145)：**

```rust
pub(crate) struct RenderRootState {
    // ...
    /// Scene cache for the widget tree.
    pub(crate) scene_cache: HashMap<WidgetId, (Scene, Scene)>,
}
```

**繪製流程 (paint.rs:18-133)：**

```rust
fn paint_widget(
    global_state: &mut RenderRootState,
    complete_scene: &mut Scene,
    scene_cache: &mut HashMap<WidgetId, (Scene, Scene)>,
    node: ArenaMut<'_, WidgetArenaNode>,
) {
    // 1. 獲取或創建緩存
    let (scene, postfix_scene) = scene_cache.entry(id).or_default();
    scene.reset();
    postfix_scene.reset();

    // 2. 只重繪需要更新的 widget
    if ctx.widget_state.request_paint {
        widget.paint(&mut ctx, &props, scene);
    }

    // 3. 合成到最終場景（不清除子孫緩存）
    complete_scene.append(scene, Some(transform));

    // 4. 遞歸處理子孫
    recurse_on_children(..., |mut node| {
        paint_widget(..., node.reborrow_mut(), ...);
    });
}
```

**Xilem 設計特點：**

| 特性 | 實現方式 |
|------|----------|
| 緩存存儲 | `HashMap<WidgetId, (Scene, Scene)>` |
| 髒標記 | `request_paint` + `request_post_paint` |
| Children 處理 | 遞歸但不清除緩存 |
| 後繪製 | 獨立的 `post_paint` Scene |

### 2.2 Leptos/Tachys 分析

**項目路徑：** `/learn-projects/leptos/tachys/`

**核心發現：** Leptos 的 Tachys 渲染層提供了清晰的狀態管理接口。

**Fragment 定義 (fragment.rs:8-11)：**

```rust
pub struct Fragment {
    pub nodes: StaticVec<AnyView>,
}
```

**渲染接口 (view/mod.rs:39-52)：**

```rust
pub trait Render: Sized {
    type State: Mountable;

    /// 首次創建視圖
    fn build(self) -> Self::State;

    /// 更新現有視圖
    fn rebuild(self, state: &mut Self::State);
}
```

**RenderEffect (render_effect.rs:22-44)：**

```rust
pub struct RenderEffect<T> {
    value: Arc<RwLock<Option<T>>>,
    inner: Arc<RwLock<EffectInner>>,
}
```

**RenderEffect 特點：**
1. **立即同步執行**：創建時立即運行，不等待微任務
2. **可取消**：Drop 時取消訂閱
3. **自動追蹤**：依賴信號變化時自動重新運行

### 2.3 設計決策矩陣

| 設計選項 | Xilem 方式 | Leptos 方式 | Rvue 選擇 |
|---------|-----------|-------------|----------|
| 緩存存儲 | `HashMap<Id, Scene>` | `Fragment { nodes }` | `HashMap<ComponentId, Scene>` |
| 狀態管理 | `request_paint` 標記 | `Render::State` trait | `is_dirty + vello_cache` |
| 更新模式 | 條件重繪 + append | `build() + rebuild()` | `build() + rebuild()` |
| Children | 遞歸不清除 | 遞歸 rebuild | 遞歸不清除緩存 |

---

## 3. 技術設計

### 3.1 架構總覽

```
┌──────────────────────────────────────────────────────────────────┐
│  Scene (應用級)                                                    │
│  ├── vello_scene: Scene ← Vello 場景                              │
│  └── scene_cache: HashMap<ComponentId, (Scene, Scene)>            │
├──────────────────────────────────────────────────────────────────┤
│  Component                                                        │
│  ├── id: ComponentId ← 唯一標識                                    │
│  ├── is_dirty: AtomicBool ← 是否需要重繪                           │
│  ├── vello_cache: GcCell<Option<SceneWrapper>> ← 緩存（可選）     │
│  └── children: GcCell<Vec<Gc<Component>>> ← 子組件               │
├──────────────────────────────────────────────────────────────────┤
│  渲染流程                                                          │
│  1. Signal 更新 → Effect 運行 → 標記 dirty                         │
│  2. Scene::update() 收集 dirty 組件                               │
│  3. 遍歷組件樹：                                                    │
│     ├─ dirty：重繪並更新緩存                                       │
│     └─ 不 dirty：直接 append 緩存                                  │
└──────────────────────────────────────────────────────────────────┘
```

### 3.2 核心數據結構

```rust
/// 場景緩存條目
#[derive(Trace)]
pub struct SceneCacheEntry {
    /// 主場景
    scene: Scene,
    /// 後繪製場景（用於 transform、clip 等）
    postfix_scene: Scene,
    /// 邊界框（用於可見性裁剪）
    bounds: kurbo::Rect,
    /// 窗口變換矩陣
    transform: kurbo::Affine,
    /// 是否已初始化
    initialized: bool,
}

/// 組件渲染狀態
#[derive(Trace)]
pub struct ComponentRenderState {
    /// 緩存條目
    cache: Option<SceneCacheEntry>,
    /// 髒標記
    is_dirty: AtomicBool,
    /// 髒標記的傳播版本（用於避免重複標記）
    dirty_version: AtomicU64,
}
```

### 3.3 渲染流程設計

**Scene::update() 流程：**

```rust
impl Scene {
    pub fn update(&mut self, root: &Gc<Component>) {
        self.vello_scene.reset();

        // 收集所有 dirty 的組件
        let dirty_components = Self::collect_dirty_components(root);

        // 遍歷並重繪
        Self::render_component(
            root,
            &mut self.vello_scene,
            &mut self.scene_cache,
            &dirty_components,
        );
    }

    fn collect_dirty_components(component: &Gc<Component>) -> HashSet<ComponentId> {
        let mut dirty = HashSet::new();
        Self::collect_dirty_recursive(component, &mut dirty);
        dirty
    }

    fn collect_dirty_recursive(
        component: &Gc<Component>,
        dirty: &mut HashSet<ComponentId>,
    ) {
        if component.render_state.is_dirty.load(Ordering::Acquire) {
            dirty.insert(component.id);
        }

        for child in component.children.borrow().iter() {
            Self::collect_dirty_recursive(child, dirty);
        }
    }
}
```

**render_component() 實現：**

```rust
impl Scene {
    fn render_component(
        component: &Gc<Component>,
        complete_scene: &mut Scene,
        scene_cache: &mut HashMap<ComponentId, SceneCacheEntry>,
        dirty_set: &HashSet<ComponentId>,
    ) {
        let id = component.id;
        let is_dirty = dirty_set.contains(&id);

        // 獲取或創建緩存條目
        let entry = if is_dirty {
            // Dirty：重繪並更新緩存
            let entry = scene_cache.entry(id).or_default();
            entry.scene.reset();
            entry.postfix_scene.reset();

            // 執行組件繪製
            component.paint(&mut entry.scene);

            // 執行後繪製
            component.post_paint(&mut entry.postfix_scene);

            // 更新邊界框和變換
            entry.bounds = component.layout_bounds();
            entry.transform = component.window_transform();

            entry
        } else {
            // 不 dirty：使用緩存
            match scene_cache.get(&id) {
                Some(entry) => entry,
                None => {
                    // 首次渲染，創建並重繪
                    let entry = scene_cache.entry(id).or_default();
                    component.paint(&mut entry.scene);
                    component.post_paint(&mut entry.postfix_scene);
                    entry.bounds = component.layout_bounds();
                    entry.transform = component.window_transform();
                    entry
                }
            }
        };

        // 合成到最終場景
        complete_scene.append(&entry.scene, Some(entry.transform));
        complete_scene.append(&entry.postfix_scene, Some(entry.transform));

        // 遞歸處理子組件
        for child in component.children.borrow().iter() {
            Self::render_component(child, complete_scene, scene_cache, dirty_set);
        }

        // 清除髒標記
        component.render_state.is_dirty.store(false, Ordering::Release);
    }
}
```

### 3.4 髒標記傳導

```rust
impl Component {
    /// 標記組件為 dirty，並可選地向上傳導
    pub fn mark_dirty(&self, propagate_up: bool) {
        self.render_state.is_dirty.store(true, Ordering::Release);

        if propagate_up {
            if let Some(parent) = self.parent.upgrade() {
                parent.mark_dirty(true);
            }
        }
    }

    /// 標記需要後繪製
    pub fn mark_post_paint(&self) {
        self.render_state.needs_post_paint.store(true, Ordering::Release);
        self.mark_dirty(false);
    }
}
```

### 3.5 與現有代碼整合

**修改 Component 結構：**

```rust
pub struct Component {
    // ... 現有欄位

    /// 渲染狀態
    pub render_state: Gc<ComponentRenderState>,
    /// 父組件引用（用於傳導）
    pub parent: Weak<Component>,
    /// 子組件
    pub children: GcCell<Vec<Gc<Component>>>,
}

impl Component {
    pub fn new() -> Self {
        Self {
            // ...
            render_state: Gc::new(ComponentRenderState {
                cache: None,
                is_dirty: AtomicBool::new(true), // 初始為 dirty
                dirty_version: AtomicU64::new(0),
            }),
            parent: Weak::new(),
            children: GcCell::new(Vec::new()),
        }
    }
}
```

**移除有問題的代碼 (widget.rs:88-91)：**

```rust
// 移除以下代碼：
// *child.vello_cache.borrow_mut() = None;
// child.is_dirty.store(true, std::sync::atomic::Ordering::SeqCst);
// render_component(child, scene, ...);

// 替換為：
if child.render_state.is_dirty.load(Ordering::Acquire) {
    render_component(child, scene, ...);
} else {
    // 直接 append 緩存
    if let Some(cache) = child.render_state.cache {
        scene.append(&cache.scene, Some(cache.transform));
        scene.append(&cache.postfix_scene, Some(cache.transform));
    }
}
```

---

## 4. 實施計劃

### 4.1 階段一：技術債清理（P0）

**目標：** 修復 immediate 問題，移除有問題的代碼

**任務：**

| 任務 | 描述 | 預估時間 |
|------|------|----------|
| 移除 widget.rs:88-91 | 移除清除子孫緩存的邏輯 | 0.5 天 |
| 重構渲染流程 | 實現新的 render_component 流程 | 1 天 |
| 添加單元測試 | 測試父子組件更新的獨立性 | 1 天 |

**驗收標準：**
- 父組件更新不會導致未修改的子孫組件重繪
- 100 個靜態組件場景下更新性能提升 50%+

### 4.2 階段二：緩存架構完善（P1）

**目標：** 實現完整的 Scene 緩存機制

**任務：**

| 任務 | 描述 | 預估時間 |
|------|------|----------|
| 定義 SceneCacheEntry | 實現緩存條目結構 | 0.5 天 |
| 實現 scene_cache | 在 Scene 中添加 HashMap 緩存 | 1 天 |
| 實現後繪製支持 | 添加 post_paint 機制 | 1 天 |
| 實現邊界框追蹤 | 支援可見性裁剪 | 0.5 天 |

**驗收標準：**
- 1000+ 組件場景下更新性能提升 3x+
- 支援 transform 和 clip 效果

### 4.3 階段三：優化與測試（P2）

**目標：** 性能優化和全面測試

**任務：**

| 任務 | 描述 | 預估時間 |
|------|------|----------|
| 基準測試 | 建立性能基準線 | 0.5 天 |
| 內存優化 | 減少 Scene 複製 | 1 天 |
| 壓力測試 | 10000+ 組件場景 | 0.5 天 |
| 文檔更新 | 更新 API 文檔 | 0.5 天 |

**驗收標準：**
- 10000 組件場景下更新延遲 < 16ms
- 內存使用量維持在合理範圍

### 4.4 時間線

```
Week 1          Week 2          Week 3
│               │               │
├───────────────┼───────────────┤
│ P0: 修復      │ P1: 緩存架構  │ P2: 優化
│ ├─ 移除問題代碼│ ├─ SceneCache │ ├─ 基準測試
│ ├─ 重構渲染   │ ├─ 後繪製支持 │ ├─ 內存優化
│ └─ 單元測試   │ └─ 邊界框追蹤 │ └─ 壓力測試
```

---

## 5. 風險評估

### 5.1 技術風險

| 風險 | 可能性 | 影響 | 緩解措施 |
|------|--------|------|----------|
| 緩存內存增長失控 | 中 | 高 | 實現 LRU 淘汰策略 |
| 邊界框計算錯誤 | 低 | 中 | 添加單元測試驗證 |
| 向後兼容性破壞 | 低 | 高 | 提供兼容層 |

### 5.2 性能風險

| 風險 | 描述 | 緩解措施 |
|------|------|----------|
| HashMap 查找開銷 | 大量組件時可能成為瓶頸 | 使用 FxHashMap 優化 |
| Scene 內存佔用 | 緩存多個 Scene 可能佔用過多內存 | 實施緩存大小限制 |

### 5.3 回滾計劃

如果新架構出現嚴重問題，可以：

1. **回滾到當前實現**：保留 `vello_cache` 邏輯
2. **禁用緩存**：設置環境變量跳過緩存
3. **降級模式**：自動檢測並切換到簡化模式

---

## 6. 參考資料

| 來源 | 路徑 | 相關內容 |
|------|------|----------|
| Xilem/Masonry | `/learn-projects/xilem/masonry/` | `scene_cache` 實現 |
| Leptos/Tachys | `/learn-projects/leptos/tachys/` | `Render` trait、`RenderEffect` |
| MVP Review 5 | `/docs/mvp-review-5.md` | 原始設計草案 |

---

**文檔版本：** 1.0  
**創建日期：** 2026-01-26
