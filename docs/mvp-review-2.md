# Rvue MVP Implementation Review
## Alex Crichton, Leptos Team, 尤雨溪, Ryan Carniato 平行世界協作

**Date:** 2026-01-24  
**Context:** 基於 [Easy-Oilpan + Solid Macro 設計文檔](/docs/2026-01-17_17-30-40_Gemini_Google_Gemini.md) 的實現審查

---

## 🎯 Executive Summary

Rvue 已經成功實現了「平行世界」設想的核心架構：**Rust GC + Vello + Taffy + 細粒度響應式**。這是一個大膽而有遠見的實現，在許多方面達到了設計目標，但也暴露出一些需要進一步完善的技術細節。

**綜合評分：B+ → A-**（相比設計階段的 B+，實現已經取得顯著進展）

### 架構達成度
- ✅ **GC 整合**：使用 rudo-gc（hybrid GC），解決了 Rust UI 的雙向引用問題
- ✅ **細粒度響應式**：Signal/Effect 系統完整實現，具備自動依賴追蹤
- ✅ **Vello 渲染**：GPU 加速渲染已整合
- ✅ **Taffy 佈局**：Flexbox 佈局系統已整合
- ⚠️ **編譯時響應式**：部分達成（宏基礎已建立，但尚未實現 Solid.js 級別的編譯時優化）

---

## 👥 專家評審

### 🦀 Alex Crichton - Rust 底層與 GC 實現

#### ✅ 亮點

1. **GC 整合的正確性**
```rust
// signal.rs - SignalData 的 Trace 實現
unsafe impl<T: Trace + Clone + 'static> Trace for SignalData<T> {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.value.trace(visitor);
        self.subscribers.trace(visitor);
        // AtomicU64 is not GC-managed, so we don't trace it ✓
    }
}
```
**評價**：Trace 實現正確。AtomicU64 不需要 trace，value 和 subscribers 正確標記。這展示了對 GC 語義的深刻理解。

2. **借用檢查器的防禦性編程**
```rust
// signal.rs:84-96 - 雙重檢查避免重複訂閱
fn subscribe(&self, effect: Gc<Effect>) {
    let already_subscribed = {
        let subscribers = self.subscribers.borrow();
        subscribers.iter().any(|sub| Gc::ptr_eq(sub, &effect))
    };
    if !already_subscribed {
        let mut subscribers = self.subscribers.borrow_mut();
        if !subscribers.iter().any(|sub| Gc::ptr_eq(sub, &effect)) {
            subscribers.push(effect);
        }
    }
}
```
**評價**：這是教科書級別的防止 borrow checker 死鎖的代碼。先用不可變借用檢查，釋放後再用可變借用修改。避免了在 `GcCell::borrow()` 期間調用可能觸發訂閱的代碼。

3. **遞歸執行的保護**
```rust
// effect.rs:45-47 - 防止無限循環
if gc_effect.is_running.swap(true, Ordering::SeqCst) {
    return; // Already running, skip to prevent infinite loop
}
```
**評價**：非常關鍵。在細粒度響應式系統中，effect 可能在執行過程中被自己觸發的 signal 變更再次調用。這個保護機制防止了堆棧溢出。

#### ⚠️ 需要改進的地方

1. **GC 暫停時間的隱憂**
```rust
// component.rs:130 - 預分配策略不足
children: GcCell::new(Vec::with_capacity(initial_children_capacity))
```
**問題**：雖然有基本的預分配，但在大型應用中，頻繁的 `Gc::new()` 調用可能導致 GC 壓力過大。

**建議**：
- 引入對象池（Object Pool）for `Component` 和 `Effect`
- 實現分代 GC 或增量 GC（如果 rudo-gc 支持）
- 監控 GC 觸發頻率和暫停時間

2. **Trace 實現的盲點**
```rust
// widget.rs:50-54 - Derived 閉包無法 trace
ReactiveValue::Derived(_) => {
    // Closures may capture GC pointers, but we can't trace them
    // This is a limitation - derived values should be used carefully
}
```
**問題**：這是一個內存安全漏洞。如果 `Derived` 閉包捕獲了 `Gc` 指針，但沒有被 trace，GC 可能會回收仍在使用的對象。

**建議**：
- 完全移除 `ReactiveValue::Derived`，或
- 使用 `Gc<dyn Fn() -> T>` 代替 `Box<dyn Fn() -> T>`，或
- 在文檔中明確警告：Derived 閉包不可捕獲 GC 對象

3. **線程安全的缺失**
```rust
// effect.rs:8-10 - 使用 thread_local!
thread_local! {
    static CURRENT_EFFECT: RefCell<Option<Gc<Effect>>> = const { RefCell::new(None) };
}
```
**問題**：整個響應式系統是單線程的。在多窗口或背景計算場景下會受限。

**建議**：
- 為跨線程場景設計 `Send + Sync` 版本的 Signal
- 或明確文檔標註：Rvue 是單線程 UI 框架（類似 JavaScript 主線程）

---

### ⚛️ Ryan Carniato - 細粒度響應式系統

#### ✅ 亮點

1. **自動依賴追蹤的正確實現**
```rust
// signal.rs:54-60
impl<T: Trace + Clone + 'static> SignalRead<T> for ReadSignal<T> {
    fn get(&self) -> T {
        if let Some(effect) = current_effect() {
            self.data.subscribe(effect);
        }
        self.data.value.borrow().clone()
    }
}
```
**評價**：這正是 SolidJS 的核心機制！signal.get() 時自動註冊當前 effect 為訂閱者。這讓開發者無需手動管理依賴關係。

2. **細粒度更新的證據**
```rust
// component.rs:260-271 - 屬性級別的更新
pub fn set_text_content(&self, content: String) {
    let (font_size, color) = {
        if let ComponentProps::Text { font_size, color, .. } = &*self.props.borrow() {
            (*font_size, *color)
        } else {
            return;
        }
    };
    *self.props.borrow_mut() = ComponentProps::Text { content, font_size, color };
    self.mark_dirty();
}
```
**評價**：這是對的方向！不是重建整個組件，而是只更新變更的屬性。但實現還不夠深入（見後續問題）。

3. **Effect 的批量更新**
```rust
// signal.rs:100-126 - 先標記 dirty，再批量執行
fn notify_subscribers(&self) {
    let effects_to_update: Vec<Gc<Effect>> = { /* collect */ };
    for effect in effects_to_update.iter() {
        effect.mark_dirty();
    }
    for effect in effects_to_update.iter() {
        if effect.is_dirty() {
            Effect::update_if_dirty(effect);
        }
    }
}
```
**評價**：很好！先標記所有 dirty effect，再統一執行。這避免了同一個 effect 因為多個 signal 變更而被執行多次（類似 React 的 batching）。

#### ⚠️ 需要改進的地方

1. **組件不是「只執行一次的 Setup Function」**
```rust
// counter/main.rs:26-35
let view = view! {
    <Flex direction="column" gap=20.0>
        <Text content={format!("Count: {}", count.get())} />
        <Button label="+" on_click={move || set_count_inc.update(|x| *x += 1)} />
    </Flex>
};
```
**問題**：這段代碼看起來像 JSX，但它實際上在每次 `create_counter_view()` 時重建整個樹。真正的 Solid.js 組件只執行一次。

**現狀 vs. 目標：**
```javascript
// 當前 Rvue (類似 React)
fn create_counter_view() -> ViewStruct {
    let (count, set_count) = create_signal(0);
    view! { <Text content={count.get()} /> } // 每次調用都重建
}

// 理想的 Solid.js 模式
fn Counter() -> ViewStruct {
    let (count, set_count) = create_signal(0);
    // 只執行一次，建立靜態結構和響應式綁定
    let text_node = create_text_component();
    create_effect(move || text_node.set_content(count.get())); // 細粒度更新
    return text_node.into_view();
}
```

**建議**：
- 修改 `view!` 宏的展開邏輯，讓它生成「初始化 + effect 綁定」而非「每次重建」
- 參考設計文檔中的偽代碼（line 175-201）

2. **缺少 Memo（計算屬性）**
```rust
// 當前沒有類似 SolidJS 的 createMemo
let derived = move || count.get() * 2; // 每次調用都重新計算
```
**問題**：Solid.js 有 `createMemo` 來緩存計算結果。當前 Rvue 只能用閉包，沒有緩存。

**建議**：
- 添加 `create_memo<T, F>(f: F) -> ReadSignal<T>` 
- Memo 本身是一個 Signal，但只在依賴變更時重新計算

3. **Effect 清理的缺失**
```rust
// effect.rs - 沒有 cleanup 機制
pub fn create_effect<F>(closure: F) -> Gc<Effect> {
    let effect = Effect::new(closure);
    Effect::run(&effect);
    effect
}
```
**問題**：SolidJS 的 effect 可以返回清理函數：
```javascript
createEffect(() => {
    const timer = setInterval(doSomething, 1000);
    return () => clearInterval(timer); // cleanup
});
```

**建議**：
- 添加 `on_cleanup` 機制
- 在 effect 重新執行前調用上一次的 cleanup

---

### 🎨 尤雨溪 (Evan You) - 開發體驗 (DX)

#### ✅ 亮點

1. **類 Vue 的 API 設計**
```rust
let (count, set_count) = create_signal(0);
view! { <Text content={count.get()} /> }
```
**評價**：非常直觀！類似 Vue 3 的 `ref()`。開發者能立即理解這是響應式狀態。

2. **HTML-like 語法的親和力**
```rust
<Flex direction="column" gap=20.0>
    <Text content="Hello" />
</Flex>
```
**評價**：比純 Rust 代碼友好得多。對前端開發者來說學習曲線平緩。

#### ⚠️ 需要改進的地方

1. **宏展開的不透明性**
```rust
// view! 宏展開後是什麼？開發者無法看到
view! { <Text value="hello" /> }
// 展開為何種 Rust 代碼？IDE 能提供補全嗎？
```
**問題**：
- Rust Analyzer 可能無法在 `view!` 內部提供自動補全
- 錯誤消息可能指向宏內部而非用戶代碼
- 調試困難（無法在宏內部設置斷點）

**建議**：
- 提供 `cargo expand` 示例文檔
- 實現自定義 rust-analyzer 插件（難度極高）
- 或者，提供「非宏」版本的 API 作為備選：
```rust
let text = Text::new().content("hello");
let flex = Flex::new().direction(FlexDirection::Column).child(text);
```

2. **錯誤消息的質量**
```rust
// 當前如果屬性拼寫錯誤：
<Text conte="hello" /> // typo: conte instead of content
// 可能產生的錯誤：
// error: no field `conte` in struct `TextProps`
```
**建議**：
- 在宏中添加拼寫建議（類似 "did you mean `content`?"）
- 限制可用屬性，提供明確的允許列表

3. **狀態管理的模式不清晰**
```rust
// 當前沒有明確的「全局狀態」或「上下文」機制
// 開發者只能通過函數參數傳遞 signal
fn parent() {
    let (count, set_count) = create_signal(0);
    child(count, set_count); // 手動傳遞
}
```
**建議**：
- 添加 Vue 的 `provide/inject` 或 React 的 `Context` 機制
- 示例：
```rust
provide("theme", dark_theme);
// 在子組件中
let theme = inject::<Theme>("theme");
```

---

### 🦎 Leptos Team (Greg) - 宏工藝與人體工學

#### ✅ 亮點

1. **基礎宏架構已建立**
```rust
// rvue-macro/src/lib.rs
#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    let nodes = parse_view(tokens)?;
    let rvue_nodes: Vec<_> = nodes.iter()
        .filter_map(|n| convert_rstml_to_rvue(n, None))
        .collect();
    let output = generate_view_code(rvue_nodes);
    // ...
}
```
**評價**：rstml 解析 → AST 轉換 → 代碼生成的管道已經建立。這是正確的架構。

2. **屬性解析的靈活性**
```rust
// 支持靜態和動態屬性
<Text content="static" />
<Text content={dynamic_value} />
<Button on_click={move || {}} />
```
**評價**：語法直觀，類似 Leptos。

#### ⚠️ 需要改進的地方

1. **未實現編譯時依賴分析**
```rust
// 當前的 view! 宏生成的是「運行時構建」代碼
view! { <Text content={count.get()} /> }
// 生成類似：
// let text = Text::new(count.get());

// 理想的編譯時優化應該生成：
// let text_component = Component::new_text(...);
// create_effect(move || {
//     text_component.set_text_content(count.get()); // 僅更新內容
// });
```
**問題**：這是設計文檔中最關鍵的「Compile-time Reactivity」（line 59-67）。當前實現還沒有達到這個目標。

**建議**：
- 在宏展開時分析哪些屬性是 `Signal` / `ReadSignal`
- 為每個響應式屬性生成獨立的 `create_effect`
- 生成直接調用 `Component::set_*` 方法的代碼

2. **組件的嵌套展開問題**
```rust
// 如果定義了自定義組件：
#[component]
fn MyButton() -> impl View {
    view! { <Button label="Click" /> }
}

// 在另一個組件中使用：
view! { <MyButton /> }
// 宏如何知道 MyButton 是自定義組件？如何展開？
```
**問題**：當前沒有看到自定義組件的處理邏輯。

**建議**：
- 在 `view!` 宏中區分內建 widget 和自定義組件
- 自定義組件直接調用對應函數：`MyButton()`
- 內建 widget 生成構建代碼

3. **代碼生成的性能優化**
```rust
// 當前每次宏展開都解析和生成完整代碼
// 對於大型應用，編譯時間可能很長
```
**建議**：
- 使用 `proc_macro2::Span` 保留原始代碼位置信息
- 考慮增量編譯友好的設計（例如避免全局狀態）

---

## 🏗️ 架構層面的評估

### ✅ 成功達成的設計目標

1. **GC 解決雙向引用**
```rust
// component.rs:59,178
pub parent: GcCell<Option<Gc<Component>>>,
pub fn add_child(&self, child: Gc<Component>) {
    self.children.borrow_mut().push(Gc::clone(&child));
}
```
**評價**：父子雙向指針無需 `Weak<T>` 或 `Arc<RefCell<T>>`。這就是設計文檔中說的「解決 Rust UI 最頭痛的問題」（line 223）。

2. **Retained Mode 而非 Rebuild**
```rust
// component.rs:149-155
pub fn mark_dirty(&self) {
    self.is_dirty.store(true, Ordering::SeqCst);
    if let Some(parent) = self.parent.borrow().as_ref() {
        parent.mark_dirty();
    }
}
```
**評價**：組件不是每次都重建，而是標記 dirty 後局部更新。這是正確的方向（接近 Flutter 的 RenderObject，但更輕量）。

3. **Vello + Taffy 的整合**
```rust
// component.rs:520-547 - 遞歸構建 Taffy 樹
pub fn build_layout_tree(
    component: &Gc<Component>,
    taffy: &mut TaffyTree<()>,
    text_context: &mut TextContext,
) -> LayoutNode {
    let child_layouts: Vec<LayoutNode> = component.children.borrow()
        .iter()
        .map(|child| build_layout_tree(child, taffy, text_context))
        .collect();
    // ...
}
```
**評價**：組件樹 → Taffy 樹 → 佈局結果 → Vello 渲染的管道已打通。

### ⚠️ 尚未達成的設計目標

1. **編譯時響應式（設計文檔 line 59-67）**

**設計目標：**
> 宏會檢測哪個屬性是 Signal。如果是 Signal，宏會生成一個細粒度的 Listener，當 Signal 變更時，直接修改 Vello Scene 的特定 buffer，而不觸發 Layout 重算（除非必要）。

**當前狀態：**
- ✅ 宏可以解析屬性
- ❌ 宏沒有區分靜態 vs. 響應式屬性
- ❌ 沒有為響應式屬性生成細粒度 effect
- ❌ 沒有直接操作 Vello Scene 的 buffer 修改

**差距示例：**
```rust
// 當前代碼（運行時綁定）
view! { <Text content={count.get()} /> }
// 實際行為：每次重新讀取 count.get()，重建 Text widget

// 設計目標（編譯時生成）
// 宏應該生成：
let text_component = Component::new(...);
let count_clone = count.clone();
create_effect(move || {
    text_component.set_text_content(count_clone.get());
    // 直接修改 Vello Scene，無需重建組件
});
```

2. **Vello Scene 的細粒度更新（設計文檔 line 199）**

**設計目標：**
> 當 Signal 改變時，我們可以精確地計算出 Vello 畫布中受影響的區域，並只更新該部分的 GPU buffer。

**當前狀態：**
```rust
// render/scene.rs - 每次都重新生成整個 Scene
pub fn update(&mut self) {
    if self.is_dirty {
        self.vello_scene.reset();
        for fragment in &self.fragments {
            // 重新渲染所有 fragment
        }
    }
}
```
**問題**：這不是細粒度更新。整個 Scene 都被重置和重建。

**建議**：
- 記錄每個 Component 對應的 Vello Scene 區域
- 當 Component dirty 時，只更新對應區域
- 使用 Vello 的增量更新 API（如果存在）

---

## 📊 與設計文檔的對比

| 特性 | 設計目標 | 當前實現 | 差距 |
|-----|---------|---------|-----|
| GC 管理 | easy-oilpan (hybrid GC) | rudo-gc ✓ | ✅ 已達成 |
| Stack Scanning | 保守式 + 精確式 | rudo-gc 提供 | ✅ 已達成 |
| Signal/Effect | 細粒度響應式 | 完整實現 ✓ | ✅ 已達成 |
| 自動依賴追蹤 | signal.get() 自動訂閱 | 已實現 ✓ | ✅ 已達成 |
| 編譯時響應式 | 宏生成細粒度更新 | ❌ 尚未實現 | 🔴 **關鍵差距** |
| Vello 細粒度更新 | 只更新變更區域 | ❌ 全 Scene 重建 | 🔴 **關鍵差距** |
| Taffy 佈局緩存 | 按需觸發佈局 | ✓ dirty flag | ✅ 已達成 |
| 組件只執行一次 | Setup Function | ❌ 類似 React | 🔴 **關鍵差距** |

---

## 🔬 技術風險評估

### 1. GC 暫停時間（設計文檔 line 87）

**設計文檔警告：**
> UI 需要 60/120 FPS。如果 easy-oilpan 觸發全堆掃描導致掉幀，體驗會很差。

**當前狀態：**
- 沒有 GC 性能監控代碼
- 不知道實際的 GC 暫停時間
- 沒有 GC 調優參數

**建議：**
- 添加 GC 性能監控（暫停時間、觸發頻率）
- 壓力測試：1000+ 組件的 signal 更新
- 如果 rudo-gc 支持，配置增量/分代 GC

### 2. 異步與 Stack Scanning（設計文檔 line 89）

**設計文檔警告：**
> Rust 的 async fn 會生成狀態機，變量會被捕獲進結構體並存放在 Heap 上。保守式 Stack 掃描很難正確找到這些跨 await 點的指針。

**當前狀態：**
- 整個框架是同步的（沒有 async/await）
- 但未來可能需要異步操作（如網絡請求、動畫）

**建議：**
- 在文檔中明確：Rvue 是同步 UI 框架
- 如果需要異步，使用獨立的 async runtime，通過 channel 與 UI 線程通信
- 所有 GC 對象必須在主線程訪問

### 3. 宏的 IDE 支持（設計文檔 line 88）

**設計文檔警告：**
> 過於依賴過程宏會導致 Rust Analyzer 自動補全失效，且編譯時間變長。

**當前狀態：**
- 宏已大量使用（`view!`, `#[component]`）
- 沒有 IDE 插件

**建議：**
- 提供「宏展開示例」文檔
- 考慮提供非宏 API 作為備選（builder pattern）
- 監控編譯時間（在 CI 中）

---

## 🚀 優先級改進建議

### P0 - 必須修復（安全性 / 正確性）

1. **修復 `ReactiveValue::Derived` 的 Trace 問題**
   - 風險：GC 可能回收正在使用的對象
   - 方案：改用 `Gc<dyn Fn()>` 或移除該變體

2. **添加 GC 性能監控**
   - 風險：未知的 GC 暫停可能導致掉幀
   - 方案：記錄 GC 觸發時間和暫停時長

### P1 - 應該實現（核心功能）

1. **實現編譯時響應式**
   - 目標：達成設計文檔的「SolidJS Compile-time」目標
   - 方案：
     ```rust
     // 宏展開前
     view! { <Text content={count.get()} /> }
     
     // 宏展開後
     let text_component = Component::new_text(...);
     let count_ref = count.clone();
     create_effect(move || {
         text_component.set_text_content(count_ref.get());
     });
     ```

2. **Vello Scene 的細粒度更新**
   - 目標：只更新變更的組件對應的渲染區域
   - 方案：為每個 Component 記錄 Vello Scene 的 layer/path ID

3. **添加 `create_memo`**
   - 目標：緩存計算結果，避免重複計算
   - 方案：
     ```rust
     let doubled = create_memo(move || count.get() * 2);
     ```

### P2 - 可以優化（DX / 性能）

1. **Context / Provide-Inject 機制**
   - 目標：避免 prop drilling
   - 方案：線程局部的 context stack

2. **Effect cleanup 機制**
   - 目標：清理定時器、事件監聽器等
   - 方案：
     ```rust
     create_effect(move || {
         let timer = ...;
         on_cleanup(move || drop(timer));
     });
     ```

3. **更好的錯誤消息**
   - 目標：宏錯誤時提供友好的提示
   - 方案：在宏中添加拼寫檢查和屬性驗證

---

## 💡 與設計文檔的對話

### 尤雨溪：「DX 是第一優先級」

**設計文檔（line 30）：**
> Evan You (Vue/Vite): 關注開發體驗 (DX)、API 的易用性以及構建工具鏈的整合。

**當前實現：**
```rust
let (count, set_count) = create_signal(0);
view! { <Text content={count.get()} /> }
```

**Evan 的評價：**
✅ API 很直觀，類似 Vue 3 的 `ref()`。但是：
- ❌ 錯誤消息質量還不夠好
- ❌ IDE 支持不足（宏內部無補全）
- ⚠️ 需要更多文檔和示例

**建議優先級調整：**
- 在實現 P1（編譯時響應式）之前，先完善 P2（文檔、錯誤消息）
- 因為即使功能不完整，但如果 DX 好，開發者會更願意使用和反饋

### Ryan Carniato：「細粒度更新是核心優勢」

**設計文檔（line 67）：**
> UI Component 應該是一個只執行一次的 Setup Function。有了 GC，我們不需要像 Flutter 那樣重建 Widget Tree。

**當前實現的問題：**
```rust
fn create_counter_view() -> ViewStruct {
    let (count, set_count) = create_signal(0);
    view! { <Text content={count.get()} /> } // ❌ 每次調用都重建
}
```

**Ryan 的評價：**
這還不是真正的 Solid.js 模式。需要讓組件「只執行一次」，然後用 effect 綁定響應式更新。

**實現路徑：**
1. 修改 `view!` 宏，生成「組件構建 + effect 綁定」的代碼
2. 確保組件函數只在掛載時調用一次，後續更新通過 effect
3. 這將大幅減少 GC 壓力（不再頻繁創建組件）

### Alex Crichton：「GC 性能是未知風險」

**設計文檔（line 87）：**
> 如果 easy-oilpan 觸發全堆掃描導致掉幀，體驗會很差。需要實現增量式 GC 或分代 GC。

**當前實現：**
- 依賴 rudo-gc，但沒有性能監控
- 不知道實際 GC 暫停時間

**Alex 的評價：**
這是一個「黑盒」。必須添加監控才能知道是否有問題。

**立即行動：**
```rust
// 在 app.rs 中添加
struct GcMetrics {
    last_gc_time: Instant,
    gc_pause_ms: Vec<u64>,
}

// 在每次 GC 後記錄
if gc_triggered {
    metrics.gc_pause_ms.push(pause_time);
    if pause_time > 16 { // 超過一幀（60fps）
        eprintln!("Warning: GC pause {}ms exceeded frame budget", pause_time);
    }
}
```

---

## 📚 總結與展望

### 當前成就

Rvue 已經成功實現了：
1. ✅ **核心響應式系統**：Signal/Effect 自動依賴追蹤
2. ✅ **GC 整合**：解決 Rust UI 的雙向引用問題
3. ✅ **渲染管道**：Vello + Taffy 整合完成
4. ✅ **基礎宏系統**：`view!` 和 `#[component]` 可用

這已經是一個可工作的 MVP，證明了「平行世界」設想的可行性。

### 與設計文檔的差距

核心差距：**編譯時響應式** 尚未實現。

這是設計文檔的核心創新（line 59-67），也是區別於現有 Rust UI 框架的關鍵特性。當前實現更接近「運行時響應式 + GC」（類似 Leptos + GC），而非「編譯時優化」（Solid.js 模式）。

### 下一步行動

**立即（本週）：**
1. 修復 `ReactiveValue::Derived` 的 Trace 問題
2. 添加 GC 性能監控
3. 撰寫「宏展開示例」文檔

**短期（1-2 週）：**
1. 實現編譯時響應式（修改 `view!` 宏的代碼生成）
2. 添加 `create_memo`
3. Vello Scene 的細粒度更新

**中期（1-2 月）：**
1. 添加 Context/Provide-Inject
2. Effect cleanup 機制
3. 更好的錯誤消息和文檔

### 最終評價

Rvue 是一個大膽而有遠見的項目。當前實現已經證明了基礎架構的可行性，但距離設計文檔中的「終極目標」還有一段距離。最關鍵的下一步是**實現編譯時響應式**，這將讓 Rvue 真正成為「寫起來像 TypeScript/Solid，跑起來像 C++」的 Rust UI 框架。

**平行世界協作的結論：**
- Alex: "架構基礎紮實，但需要 GC 性能數據"
- Ryan: "Signal/Effect 正確，但需要實現 Solid 的 'setup once' 模式"
- Evan: "DX 不錯，但錯誤消息和文檔需要加強"
- Leptos Team: "宏基礎已建立，下一步是編譯時優化"

**綜合評分：B+ → A-**
- 如果實現編譯時響應式 → **A**
- 如果再加上 GC 性能優化 → **A+**

---

## 附錄：設計文檔關鍵引用

1. **核心理念（line 73-82）：**
   > 我們不採用 Flutter 的 "Rebuild entire widget tree"，而是採用 "Retained Mode Widget Graph with Fine-Grained Updates"。

2. **編譯時響應式（line 62-67）：**
   > Rust 是編譯語言，不像 JS 可以動態生成代碼。我們必須在宏展開階段就知道哪些屬性是動態的。

3. **GC 的價值（line 43-44）：**
   > 這將徹底解決 Rust UI 中 Rc<RefCell<T>> 的地獄。UI 樹本質上是圖結構（父指子，子指父），GC 是最自然的解法。

4. **性能目標（line 98-101）：**
   > 寫起來像 TypeScript/Solid：無生命週期煩惱。跑起來像 C++：無 VDOM，直接操作 GPU 數據。

這些目標在當前實現中**部分達成**。繼續努力！🚀
