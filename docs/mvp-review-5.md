# Rvue MVP Phase 5 Review
## Alex Crichton, Greg Johnston (Leptos), 尤雨溪 (Evan You), Ryan Carniato 平行世界協作

**Date:** 2026-01-26  
**Context:** 基於 [Easy-Oilpan + Solid Macro 設計文檔](/docs/2026-01-17_17-30-40_Gemini_Google_Gemini.md)、[MVP-Review-4](/docs/mvp-review-4.md) 與 [Stylo Survey](/docs/stylo-survey-1.md) 的深度審查與下一步戰略規劃

---

## 🎯 Executive Summary

Rvue 在 Phase 4 中達成了響應式系統的完整成熟度。當前框架已具備：
- ✅ 保守式閉包掃描的 GC 追蹤
- ✅ 成熟的 Keyed Diff 算法
- ✅ 完整的事件系統（hit-test、焦點、捕獲）
- ✅ Slot 機制（Children/ChildrenFn）
- ✅ 40+ 通過的單元測試

**綜合評分：A+ (Production Ready Foundation)**

本次審查的核心議題是：**如何跨出下一個大功能，讓 Rvue 更接近 1.0？**

---

## 📊 Phase 5 戰略選項分析

經過深度評估，我們識別出 **四個主要戰略方向**，每個都能顯著推進 Rvue 向 1.0 邁進：

| 選項 | 描述 | 複雜度 | 影響力 | 優先建議 |
|------|------|--------|--------|----------|
| **A: Stylo 整合** | CSS 選擇器 + 類型安全屬性混合方案 | 高 | 高 | ⭐⭐⭐ |
| **B: SSR/Hydration** | 服務端渲染 + 水合支持 | 極高 | 極高 | ⭐⭐ |
| **C: Vello Fragment 緩存** | 局部渲染優化 | 中 | 高 | ⭐⭐⭐⭐ |
| **D: 開發者工具生態** | DevTools + Hot Reload | 中 | 中 | ⭐ |

---

## 🔬 專家深度評審

### 🦀 Alex Crichton - 系統架構與 GC 邊界

#### ✅ Phase 4 成就確認

**1. Effect Trace 的穩定性**

Phase 4 中實現的保守式閉包掃描已經過充分測試。`Effect.trace()` 正確使用 `visitor.visit_region()` 掃描閉包的捕獲環境：

```rust
// effect.rs - 審查確認：實現正確
unsafe impl Trace for Effect {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.owner.trace(visitor);
        // 閉包的 layout 在創建時記錄
        if layout.size() > 0 && layout.align() >= std::mem::align_of::<usize>() {
            unsafe { visitor.visit_region(data_ptr, layout.size()); }
        }
    }
}
```

**2. Signal 訂閱者的弱引用改進**

```rust
// signal.rs - 使用 Weak<Effect> 正確避免循環引用
subscribers: GcCell<Vec<Weak<Effect>>>,
```

之前 Review 4 提出的 P0 問題已解決。

#### 🎯 戰略建議：優先 Vello Fragment 緩存 (Option C)

**理由：**

1. **渲染是當前明確的性能瓶頸**
   ```rust
   // scene.rs - 現狀：每次更新都全量 reset
   if let Some(ref mut scene) = self.vello_scene {
       scene.reset();  // ⚠️ 這違背了細粒度更新的核心價值
   }
   ```

2. **GC 系統已穩定，不需要再投入大量精力**

3. **Vello Fragment 緩存不需要引入新的外部依賴**

4. **這是讓 Rvue 真正「跑起來像 C++」的關鍵步驟**

**技術路徑：**

```rust
// 提議的 Fragment 緩存架構
pub struct ComponentScene {
    /// 該組件的 Vello Scene 片段
    fragment: Gc<GcCell<Scene>>,
    /// 是否需要重繪
    dirty: AtomicBool,
    /// 子組件的片段引用
    child_fragments: GcCell<Vec<Gc<ComponentScene>>>,
}

impl Component {
    fn update_fragment(&self, ctx: &mut RenderContext) {
        if self.scene.dirty.load(Ordering::Acquire) {
            // 只重繪該組件本身
            self.repaint_self(ctx);
            self.scene.dirty.store(false, Ordering::Release);
        }
        // 遞歸處理子組件
        for child in self.scene.child_fragments.borrow().iter() {
            child.update_fragment(ctx);
        }
    }
}
```

---

### ⚛️ Ryan Carniato - 響應式系統與細粒度更新

#### ✅ 響應式系統評估

**當前狀態：已達到 Solid.js 水準**

```rust
// signal.rs - 自動依賴追蹤
impl<T: Trace + Clone + 'static> SignalRead<T> for ReadSignal<T> {
    fn get(&self) -> T {
        if let Some(effect) = current_effect() {
            self.data.subscribe(effect);  // ✅ 完美的隱式訂閱
        }
        self.data.value.borrow().clone()
    }
}
```

**Keyed Diff 算法品質：優秀**

```rust
// keyed_state.rs - 與 Solid/Leptos 同等水平
pub fn diff_keys<K: Eq + Hash + Clone>(
    old_keys: &IndexSet<K, BuildHasherDefault<FxHasher>>,
    new_keys: &IndexSet<K, BuildHasherDefault<FxHasher>>,
) -> KeyedDiff<K>
```

- ✅ 使用 `FxHasher` 優化
- ✅ 使用 `IndexSet` 保持順序
- ✅ 實現 `group_adjacent_moves()`
- ✅ 降序排序 removals

#### 🎯 戰略建議：Stylo 選擇器整合後接 Fragment 緩存

**理由：**

1. **CSS 狀態（:hover, :focus, :active）需要與響應式系統深度整合**

2. **Signal 驅動的樣式更新是 Solid 模式的自然延伸**

3. **但渲染優化（Fragment）應該先於 CSS，因為 CSS 會增加更多渲染負擔**

**提議的整合方案：**

```rust
// Signal-driven styling
#[component]
fn Button(children: Children) -> impl Widget {
    let (hovered, set_hovered) = create_signal(false);
    
    // 響應式樣式：Signal 變化 -> 標記 Fragment dirty
    let style = create_memo(move || {
        Style {
            background: if hovered.get() { Color::LightBlue } else { Color::Blue },
            ..Default::default()
        }
    });
    
    view! {
        <Box
            on:pointer_enter={move |_| set_hovered(true)}
            on:pointer_leave={move |_| set_hovered(false)}
            style={style}  // 當 memo 更新時，只標記該 Button 的 fragment dirty
        >
            {children}
        </Box>
    }
}
```

---

### 🎨 尤雨溪 (Evan You) - 開發體驗 (DX)

#### ✅ 當前 DX 評估

**1. view! 宏表達力：優秀**

```rust
view! {
    <Flex direction="column" gap=20.0 align_items="center">
        <Text content={format!("Count: {}", count.get())} />
        <Show when={show_message.get()}>
            <Text content="Counter is active!" />
        </Show>
        <Button label="+" on_click={move || set_count.update(|x| *x += 1)} />
    </Flex>
}
```

**2. Slot 機制：完整實現**

```rust
// slot.rs - Children 和 ChildrenFn 都已實現
pub struct Children(pub Box<dyn FnOnce() -> ViewStruct>);
pub struct ChildrenFn(pub(crate) Gc<LazyView>);
pub struct MaybeChildren(pub Option<ChildrenFn>);
```

**3. 樣式系統：基礎但功能有限**

```rust
// style.rs - 當前只支持內聯樣式
pub struct Style {
    pub color: Option<Color>,
    pub background_color: Option<Color>,
    pub font_size: Option<f32>,
    // ... 17 個屬性，全部 Option<T>
}
```

#### ⚠️ DX 痛點

**1. 無 CSS 類名/選擇器支持**

```rust
// 當前：每個組件都要內聯樣式
<Button style={Style { background_color: Some(Color::Blue), .. }} />

// 期望：CSS 類名
<Button class="primary-button" />
```

**2. 無偽類狀態支持**

```rust
// 當前：手動管理 hover 狀態
let (hovered, set_hovered) = create_signal(false);
<Box
    on:pointer_enter={move |_| set_hovered(true)}
    on:pointer_leave={move |_| set_hovered(false)}
    style={if hovered.get() { hover_style } else { base_style }}
/>

// 期望：CSS-like 偽類
<Box class="my-box" />
// .my-box:hover { background: lightblue; }
```

#### 🎯 戰略建議：Stylo Hybrid 方案 (Option A)

**理由：**

1. **熟悉的 CSS 語法降低學習曲線**
2. **:hover/:focus/:active 是桌面應用的基本需求**
3. **類型安全的屬性系統保留 Rust 的編譯時檢查**

**DX 願景：**

```rust
// 未來的樣式 API
stylesheet! {
    ".button" {
        padding: 8.0;
        border_radius: 4.0;
        background: blue;
    }
    ".button:hover" {
        background: lightblue;
    }
    ".button:active" {
        transform: scale(0.98);
    }
}

#[component]
fn Button(label: &str) -> impl Widget {
    view! {
        <Box class="button">
            <Text content={label} />
        </Box>
    }
}
```

---

### 🦎 Greg Johnston (Leptos Team) - 宏工藝與架構

#### ✅ 宏實現評估

**codegen.rs 結構清晰**

```rust
// 職責分離良好
generate_view_code(nodes: Vec<RvueNode>) -> TokenStream
generate_element_code(el: &RvueElement, ctx_ident: &Ident) -> TokenStream
generate_widget_builder_code(...) -> TokenStream
generate_reactive_effects(...) -> TokenStream
```

**analysis.rs 響應式檢測**

```rust
// 自動檢測響應式表達式
classify_expression(expr: &Expr) -> ExpressionKind
```

#### ⚠️ 架構挑戰

**1. 缺少 Props 派生宏**

```rust
// 當前：手動實現 builder pattern
impl Button {
    pub fn label(mut self, label: &str) -> Self { ... }
    pub fn on_click<F>(mut self, f: F) -> Self { ... }
}

// 期望：類似 Leptos 的 #[derive(Props)]
#[derive(Props)]
pub struct ButtonProps {
    #[prop(into)]
    label: String,
    #[prop(optional)]
    on_click: Option<Box<dyn Fn() + 'static>>,
}
```

**2. 缺少 Context 的完整追蹤**

```rust
// component.rs:125-128 - 仍是佔位符
for _entry in self.contexts.borrow().iter() {
    // Manual trace of context values - placeholder
}
```

#### 🎯 戰略建議：完善宏基礎設施後再擴展功能

**優先順序：**

1. **修復 Context Trace（P0，1-2 天）**
2. **實現 #[derive(Props)]（P1，3-5 天）**
3. **然後才考慮 Stylo 整合**

---

## 🏗️ 戰略決策矩陣

### Option A: Stylo 整合

| 維度 | 評估 |
|------|------|
| **價值主張** | 讓開發者使用熟悉的 CSS 語法，支持 :hover/:focus/:active |
| **技術可行性** | 中等。`selectors` crate 可獨立使用，但需實現 ~25 個 trait 方法 |
| **依賴影響** | 增加 `selectors`, `cssparser`, `smallvec` 等 ~8 個 crates |
| **時間估計** | 3-4 週 |
| **風險** | 中。可能與 GC 系統有整合複雜度 |

**詳細計劃（來自 [Stylo Survey](/docs/stylo-survey-1.md)）：**

```
Phase 1: selectors Integration (Week 1-2)
  └─ 實現 RvueSelectorImpl + Element trait

Phase 2: Property System (Week 2-3)
  └─ Port Masonry 的 Property trait

Phase 3: Stylesheet Support (Week 3-4)
  └─ stylesheet! macro

Phase 4: Signal Integration (Week 4)
  └─ 響應式樣式綁定
```

### Option B: SSR/Hydration

| 維度 | 評估 |
|------|------|
| **價值主張** | 支持服務端渲染，拓展 Web 應用場景 |
| **技術可行性** | 極高難度。需要 WASM 支持、序列化、重新水合 |
| **依賴影響** | 大量工作，可能需要分離運行時 |
| **時間估計** | 8-12 週 |
| **風險** | 極高。可能需要重構核心架構 |

**結論：不建議作為下一步。應在 1.0 後考慮。**

### Option C: Vello Fragment 緩存 ⭐ **推薦**

| 維度 | 評估 |
|------|------|
| **價值主張** | 實現真正的局部渲染，大幅提升性能，符合「跑起來像 C++」的願景 |
| **技術可行性** | 高。Vello API 支持 Scene::append，無需新依賴 |
| **依賴影響** | 零新依賴 |
| **時間估計** | 2-3 週 |
| **風險** | 低。失敗可回滾到全量渲染 |

**實現計劃：**

```
Week 1: Fragment 架構
  ├─ ComponentScene struct with dirty flag
  ├─ 修改 Component 持有 fragment 引用
  └─ 實現 mark_dirty 傳導

Week 2: 增量合成
  ├─ 修改 Scene::update() 使用 Scene::append()
  ├─ 實現 Z-order 管理
  └─ 測試性能改進

Week 3: 優化 + 整合
  ├─ 與 Signal 系統整合
  ├─ 基準測試（1000+ 組件場景）
  └─ 文檔
```

### Option D: 開發者工具

| 維度 | 評估 |
|------|------|
| **價值主張** | 提升開發者體驗，類似 Vue DevTools |
| **技術可行性** | 中等。需要獨立 UI 或瀏覽器擴展 |
| **依賴影響** | 可能需要 WebSocket 或類似通信 |
| **時間估計** | 4-6 週 |
| **風險** | 中。工作量大但技術風險低 |

**結論：價值高但優先級較低。應在核心功能穩定後實現。**

---

## 🚀 Phase 5 建議執行計劃

### 推薦順序：C → A（先 Fragment，後 Stylo）

**理由：**

1. **Fragment 緩存是性能基礎**
   - Stylo 整合會增加樣式計算開銷
   - 沒有 Fragment 緩存，CSS 動態更新會更慢
   - 先有局部渲染，再添加局部樣式

2. **Fragment 風險低、收益確定**
   - 2-3 週即可完成
   - 失敗可回滾
   - 成功將大幅提升複雜 UI 性能

3. **Stylo 需要更多準備**
   - Context Trace 需先修復
   - 可能需要 Props derive 支持
   - 依賴較多需謹慎評估

### 階段劃分

```
︱
╰─ Phase 5.0: 技術債清理（1 週）
    ├─ P0: 修復 Context Trace
    ├─ P1: create_memo 雙重計算問題
    └─ P2: 事件處理器類型推斷
︱
╰─ Phase 5.1: Vello Fragment 緩存（2-3 週）
    ├─ ComponentScene 架構
    ├─ 增量合成
    └─ 性能測試
︱
╰─ Phase 5.2: Stylo 整合（3-4 週）
    ├─ selectors crate 整合
    ├─ Property trait system
    ├─ stylesheet! macro
    └─ 響應式樣式
︱
╰─ Phase 5.3: 生態完善（2 週）
    ├─ #[derive(Props)]
    ├─ 文檔更新
    └─ 示例應用
```

---

## 📐 技術設計草案

### Vello Fragment 緩存架構

```rust
/// 組件的渲染片段
#[derive(Trace)]
pub struct ComponentFragment {
    /// 該組件的 Vello Scene 片段
    scene: GcCell<Scene>,
    /// 是否需要重繪
    dirty: AtomicBool,
    /// 該片段在父片段中的邊界
    bounds: GcCell<kurbo::Rect>,
}

impl ComponentFragment {
    pub fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Release);
    }
    
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Acquire)
    }
    
    pub fn get_or_repaint<F>(&self, repaint: F) -> &Scene 
    where F: FnOnce(&mut Scene)
    {
        if self.is_dirty() {
            let mut scene = self.scene.borrow_mut();
            scene.reset();
            repaint(&mut scene);
            self.dirty.store(false, Ordering::Release);
        }
        &*self.scene.borrow()
    }
}

// 修改 Component
pub struct Component {
    // ... existing fields
    fragment: Gc<ComponentFragment>,
}

// 修改 Scene::update()
impl Scene {
    pub fn update(&mut self, root: &Gc<Component>) {
        // 不再 reset 整個 scene
        // 而是遞歸收集 dirty fragments
        self.vello_scene.reset();
        self.compose_fragments(root);
    }
    
    fn compose_fragments(&mut self, component: &Gc<Component>) {
        let frag = component.fragment.get_or_repaint(|scene| {
            component.paint(scene);
        });
        
        // 使用 Vello 的 append 而非重繪
        self.vello_scene.append(frag, None);
        
        for child in component.children.borrow().iter() {
            self.compose_fragments(child);
        }
    }
}
```

### Stylo 整合架構

```rust
/// 樣式選擇器實現
pub struct RvueSelectorImpl;

impl selectors::SelectorImpl for RvueSelectorImpl {
    type ExtraMatchingData<'a> = ();
    type AttrValue = String;
    type Identifier = String;
    type LocalName = String;
    type NamespacePrefix = String;
    type NamespaceUrl = String;
    type BorrowedNamespaceUrl = str;
    type BorrowedLocalName = str;
    type NonTSPseudoClass = RvuePseudoClass;
    type PseudoElement = RvuePseudoElement;
}

/// 偽類定義
#[derive(Clone, PartialEq, Eq)]
pub enum RvuePseudoClass {
    Hover,
    Focus,
    Active,
    Disabled,
    Checked,
}

/// 元素包裝器，用於選擇器匹配
pub struct RvueElement<'a> {
    component: &'a Component,
    state: ElementState,
}

impl<'a> selectors::Element for RvueElement<'a> {
    type Impl = RvueSelectorImpl;
    
    fn parent_element(&self) -> Option<Self> { ... }
    fn has_class(&self, name: &str, case_sensitivity: CaseSensitivity) -> bool { ... }
    fn has_id(&self, name: &str, case_sensitivity: CaseSensitivity) -> bool { ... }
    fn is_root(&self) -> bool { ... }
    // ... 其他 ~25 個方法
}

/// 樣式解析器
pub struct StyleResolver {
    rules: Vec<StyleRule>,
    selector_caches: SelectorCaches,
}

impl StyleResolver {
    /// 解析 CSS-like 樣式表
    pub fn parse(css: &str) -> Result<Self, StyleError> { ... }
    
    /// 解析組件樣式
    pub fn resolve(&self, element: &RvueElement) -> Style {
        let mut style = Style::default();
        for rule in &self.rules {
            if rule.selector.matches(element, &mut self.selector_caches) {
                style.merge(&rule.properties);
            }
        }
        style
    }
}
```

---

## 📊 當前代碼質量回顧

```
crates/rvue/src/
├── lib.rs          (54 lines)    - 模組入口 ✅
├── app.rs          (633 lines)   - 應用運行器 ✅
├── component.rs    (766 lines)   - 組件系統 ⚠️ Context Trace 待修復
├── signal.rs       (298 lines)   - 響應式 Signal ✅
├── effect.rs       (256 lines)   - Effect 系統 ✅
├── slot.rs         (272 lines)   - Slot 機制 ✅ NEW
├── context.rs      (30 lines)    - Context API ⚠️
├── style.rs        (200 lines)   - 樣式系統 ⚠️ 待擴展
├── runtime.rs      (31 lines)    - Owner 堆棧 ✅
├── widgets/        (~60KB)       - Widget 實現 ✅
│   ├── for_loop.rs (400+ lines)  - For 組件 + diff ✅
│   ├── keyed_state.rs (458 lines)- 差異算法 ✅
│   └── ...
└── event/          (~40KB)       - 事件系統 ✅

crates/rvue-macro/src/
├── lib.rs          (131 lines)   - 宏入口 ✅
├── codegen.rs      (635 lines)   - 代碼生成 ✅
├── slot.rs         (180 lines)   - Slot 宏 ✅ NEW
├── analysis.rs     (200 lines)   - 響應式檢測 ✅
└── ...

測試統計：44 單元測試全部通過（40 lib + 4 integration）
```

---

## 🎯 P0 行動項（必須在 Phase 5 開始前完成）

### 1. Context Trace 修復

```rust
// 當前 component.rs:125-128 - 佔位符
for _entry in self.contexts.borrow().iter() {
    // TODO: placeholder
}

// 修復方案
struct ContextEntry {
    type_id: TypeId,
    value: Box<dyn Any>,
    tracer: Box<dyn Fn(&mut dyn rudo_gc::Visitor)>,
}

impl Component {
    fn provide_context<T: Trace + Any>(&self, value: T) {
        let gc_value = Gc::new(value);
        let tracer = Box::new(move |visitor: &mut dyn rudo_gc::Visitor| {
            gc_value.trace(visitor);
        });
        self.contexts.borrow_mut().insert(TypeId::of::<T>(), ContextEntry {
            type_id: TypeId::of::<T>(),
            value: Box::new(gc_value),
            tracer,
        });
    }
}

unsafe impl Trace for Component {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        // ... existing trace
        for entry in self.contexts.borrow().iter() {
            (entry.tracer)(visitor);
        }
    }
}
```

### 2. create_memo 雙重計算修復

```rust
// 當前 signal.rs - 初始化時計算兩次
pub fn create_memo<T, F>(f: F) -> ReadSignal<T> {
    let (read, write) = create_signal(untracked(&f));  // 第一次
    create_effect(move || {
        write.set(f_clone());  // 第二次 (Effect 首次運行)
    });
    read
}

// 修復
pub fn create_memo<T, F>(f: F) -> ReadSignal<T> {
    let (read, write) = create_signal(untracked(&f));
    let is_first = std::cell::Cell::new(true);
    create_effect(move || {
        if is_first.replace(false) { 
            return;  // 跳過首次
        }
        write.set(f_clone());
    });
    read
}
```

---

## 💡 總結

Rvue 在 Phase 4 奠定了堅實的響應式系統基礎。現在的關鍵決策是：**如何最有效地向 1.0 邁進？**

**推薦路徑：先 Fragment 後 Stylo**

1. **Vello Fragment 緩存（2-3 週）**
   - 實現局部渲染，解決當前的性能瓶頸
   - 為 Stylo 整合打下性能基礎
   - 風險低，收益確定

2. **Stylo Hybrid 整合（3-4 週）**
   - 使用 `selectors` crate 支持 CSS 選擇器
   - 實現類型安全的屬性系統
   - 支持 :hover/:focus/:active 偽類

3. **生態完善（2 週）**
   - #[derive(Props)]
   - 完善文檔和示例

**預計 Phase 5 結束後，Rvue 將具備：**

> **「寫起來像 TypeScript/Solid（無生命週期煩惱 + CSS 熟悉度），跑起來像 C++（局部渲染 + GPU 加速），佈局像 Flutter/CSS（強大的佈局能力 + CSS 選擇器）。」**

---

**評審長簽名**：

*"The foundation is solid. The reactive system is mature. Now the question is not 'can we build it?' but 'what should we build next?'. My recommendation: Fragment caching first, because you can't have fast CSS transitions without fast rendering. Fix the P0 issues, implement fragment caching, then tackle Stylo. That's the path to 1.0."* 🚀

— Alex Crichton, Greg Johnston, 尤雨溪, Ryan Carniato (模擬)

---

## 附錄：其他考慮的功能

| 功能 | 優先級 | 備註 |
|------|--------|------|
| 動畫系統 | P2 | 需要 Fragment 緩存支持 |
| 自定義繪製 | P2 | 類似 Flutter CustomPaint |
| Accessibility | P1 | 1.0 前需要考慮 |
| 國際化 | P2 | 可獨立 crate 實現 |
| 持久化佈局 | P3 | 複雜度高 |
| Web 目標 | P3 | 需要 WASM 運行時 |
