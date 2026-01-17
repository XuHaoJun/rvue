# Rvue MVP Review - Parallel World Collaboration

**Reviewers:** Alex Crichton, Leptos Team, 尤雨溪 (Evan You), Ryan Carniato
**Date:** 2026-01-17

---

## 🎙️ 專家 Review 意見總結

### 1. 🏗️ Alex Crichton (Runtime & Infrastructure)
> "The move to a GC-managed UI tree is bold but necessary for Rust GUI sanity."

*   **優點：** `rudo-gc` (easy-oilpan) 的引入極大簡化了 UI 樹的構建，避免了 `Rc<RefCell<T>>` 的地獄。`Trace` trait 的實作符合預期，確保了內存安全。
*   **風險：** 目前 `Component` 結構中的 `Vec<Gc<Component>>` 雖然在單線程環境運行良好，但隨著 GPU 渲染 (Vello) 的加入，需要考慮 GC 的暫停時間 (STW) 是否會導致掉幀。
*   **建議：** 接下來實作 Vello GPU 渲染時，應考慮如何將 GC 對象與 GPU Buffer 生命週期同步。

### 2. ⚡ Ryan Carniato (Reactivity)
> "Reactivity is not just about signals; it's about the precision of the update path."

*   **優點：** `signal.rs` 成功實作了細粒度的依賴追蹤。`create_effect` 正確地在執行時捕獲了 `current_effect`。
*   **問題：** 目前 UI 的更新仍然依賴 `ComponentLifecycle::update` 的遞迴遍歷。這不是真正的 "Solid-like" 細粒度更新。
*   **建議：** `view!` 宏應該產出直接綁定到 Vello 屬性的 `Effect`。例如，修改 `Text` 的內容不應該觸發 `Component::update`，而應該直接觸發 `VelloFragment::update_vello_text`。

### 3. 🎨 尤雨溪 (DX & API Design)
> "Developer experience is the interface between the framework and the human mind."

*   **優點：** 擬定的 API 風格與 Vue/Solid 非常接近，對於從 Web 轉過來的開發者非常親切。
*   **批評：** `view!` 宏目前還只是個 Placeholder。在 Rust 中實作 JSX-like DSL 且要兼顧 Ownership 是非常大的挑戰。
*   **建議：** 優先實作 `view!` 宏的基礎解析，讓 `Counter` 範例能真正跑起來。目前的 `ComponentProps` 使用字串存儲數據稍嫌笨重，應考慮更強型別的屬性傳遞。

### 4. 🦀 Leptos Team (Rust GUI Implementation)
> "The balance between Rust's safety and UI flexibility is found in the macro layer."

*   **優點：** 選擇 `Taffy` 作為佈局引擎是非常正確的決定。
*   **比較：** 與 Leptos 使用的 Arena/Lifetimes 模式不同，Rvue 的 GC 模式允許更自由的組件引用，但開發者需要學習 `Gc<T>` 的心智模型。
*   **技術細節：** `render/widget.rs` 目前充滿了矩形佔位符。在實作 Vello 渲染時，需要建立一套完整的 `Widget -> Scene Graph` 映射機制，並處理字體渲染。

---

## 🚀 下一步實作建議 (Roadmap to GPU Rendering)

1.  **Vello Renderer 整合：** 在 `app.rs` 中實作真正的 WGPU + Vello 渲染循環，取代目前的空實作。
2.  **細粒度更新路徑：** 將 `Signal` 變更直接映射到 `Vello Scene` 的修改，而非重建整個 Fragment。
3.  **Layout & Render 分離：** 確保 Taffy 的佈局計算不會在每次繪圖時執行，僅在 `is_dirty` 為真時更新。
4.  **宏的深度整合：** 開始實作 `view!` 宏，將閉包轉換為 `Effect` 並與組件屬性綁定。

---

**結論：** 架構方向正確，已具備細粒度響應式的基礎設施。下一步實作 GPU 渲染時，重點在於**「如何讓 Signal 變更直接反映在 GPU 指令流中」**。
