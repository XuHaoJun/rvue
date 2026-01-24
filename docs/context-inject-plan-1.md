# Rvue Context (Provide/Inject) 設計方案

本文檔探討了 Rvue 中 Context 機制的實現方案，並根據 Alex Crichton、Leptos Team、尤雨溪及 Ryan Carniato 的專家評審意見進行了優化。

## 1. 方案調研

### 方案 A：基於組件樹的向上遍歷 (Leptos / Flutter 模式)
- **機制**：數據存儲在組件節點上。`inject` 時沿著 `parent` 指針向上遍歷。
- **優點**：與組件生命週期綁定，支持動態查找。

### 方案 B：寫入時繼承的 Context Map (Svelte 模式)
- **機制**：組件初始化時，獲取父組件的 Context Map 並進行 Shallow Copy。
- **優點**：`inject` 查找速度極快 (O(1))。

### 方案 C：宏驅動的「隱式參數」注入 (Compile-time DI)
- **機制**：在 `#[component]` 宏展開時自動插入 `inject` 調用。

### 方案 D：響應式 Dependency Context (Solid.js 模式)
- **機制**：Context 傳遞 Signal Handle，變更時自動觸發下游細粒度更新。

---

## 2. Rvue 最終建議方案：安全遍歷(方案A) + 響應式優先(方案D)

### 核心架構與 GC 安全性
1.  **GC 安全容器**：
    為了解決 `Box<dyn Any>` 無法被 GC 追蹤的問題，我們定義一個具備 Trace 能力的特徵：
    ```rust
    pub trait ContextValue: Trace + Any {
        fn as_any(&self) -> &dyn Any;
    }
    impl<T: Trace + Any> ContextValue for T {
        fn as_any(&self) -> &dyn Any { self }
    }
    ```
    組件存儲改為：
    ```rust
    pub struct Component {
        // ...
        pub contexts: GcCell<HashMap<TypeId, Gc<dyn ContextValue>>>,
    }
    ```
    *註：這樣 `Component::trace` 就能正確追蹤 Context 中的所有 GC 指針。*

2.  **線程局部追蹤**：
    使用 `thread_local!` 追蹤當前正在執行的「Setup/構建中」組件。這僅在組件函數執行期間有效。

### API 設計 (DX 優化)
- **`provide<T: Trace + Any>(value: T)`**：
  - 僅限在組件 Setup 階段調用。
  - 將數據封裝成 `Gc<T>` 並存入當前組件的 `contexts`。
- **`inject<T: Any>() -> Option<T>`**：
  - 向上遍歷 `parent` 鏈條，返回最近的匹配項。
- **`expect_context<T: Any>() -> T`**：
  - `inject` 的嚴格版本，若缺失則報錯並提供友好的錯誤消息（包含組件層次上下文）。
- **`use_context<T: Any>() -> T`**：
  - 同 `expect_context`，符合 Leptos/Hooks 習慣。

### 響應式建議
- **強烈建議** Context 存儲 `ReadSignal<T>` 而非純數值 `T`。
- 這樣當 Provider 的數據變化時，所有注入了該 Context 的組件 Effect 會自動重新執行。

---

## 3. 方案對比總結

| 特性 | 組件樹遍歷 (建議) | 寫入時繼承 | 備註 |
| :--- | :--- | :--- | :--- |
| **GC 安全性** | ✅ 高 (通過 TraceAny) | ✅ 高 | 必須解決 dyn Any 的追蹤問題 |
| **查找性能** | O(depth) | O(1) | 大多數 UI 樹深度 < 50，遍歷開銷可忽略 |
| **DX (開發體驗)** | ✅ 極佳 | ✅ 極佳 | 符合 Vue/Solid 開發者直覺 |
| **動態性** | 支持全生命週期查找 | 僅限 Setup 繼承 | 遍歷方案更具彈性 |

---

## 4. 下一步行動 (優先級)

1.  **P0 - 基礎設施**：
    - 在 `rvue` 中定義 `ContextValue` trait。
    - 在 `Component` 結構中添加 `contexts` 字段並更新 `Trace` 實現。
2.  **P1 - 核心 API**：
    - 實現 `provide` / `inject` / `expect_context`。
    - 在 `view!` 宏與 `#[component]` 展開過程中正確設置/清除 `CURRENT_COMPONENT`。
3.  **P2 - 錯誤處理與文檔**：
    - 完善 `expect_context` 的錯誤提示（顯示組件 ID 或名稱）。
    - 撰寫響應式 Context 最佳實踐文檔。
