# rvue 架構草案：基於 rudo-gc 的可行性分析

## 1. 背景與目標

本文件旨在評估 `rudo-gc`（即 "easy-oilpan" 實作）作為新一代 Rust GUI 框架 `rvue` 核心內存管理基石的技術可行性。該框架概念源於「平行世界協作」構想，試圖結合：
- **SolidJS** 的細粒度響應式（Fine-grained Reactivity）與編譯時優化（Macros）。
- **Taffy** 的 Flexbox/Grid 佈局能力。
- **Vello** 的高性能 GPU 計算渲染。
- **Easy-Oilpan (`rudo-gc`)** 的開發者體驗（無強制的 Lifetime/Borrow/Rc<RefCell> 地獄）。

## 2. `rudo-gc` (Easy-Oilpan) 技術現狀分析

經代碼審查 (`crates/rudo-gc`)，`rudo-gc` 目前已具備支持該架構的關鍵特性：

### 核心優勢 (Pros)
1.  **混合式掃瞄策略 (Hybrid Scanning)**:
    -   **保守式棧掃瞄 (Conservative Stack Scanning)**: 透過 `stack.rs` 中的 `spill_registers_and_scan` 實現，支持 x86_64 與 aarch64。這極大降低了 Rust UI 開發的門檻，開發者無需像在其它 Rust GC 中那樣手動 `Root` 每一個棧變量。
    -   **精確式堆掃瞄 (Precise Heap Scanning)**: 提供 `#[derive(Trace)]`，確保 Heap 上的複雜物件圖（如 UI 樹、Signal 依賴圖）能被精確回收，無內存洩漏風險。

2.  **內存佈局與性能**:
    -   **BiBOP (Big Bag of Pages)**: 採用固定大小 Page (4KB) 與 Size-Class 分配。這不僅實現了 O(1) 分配，更關鍵的是**支持 Interior Pointers**（內部指針）。這意味著即使編譯器優化後只保留了指向物件內部的指針（而非物件首地址），GC 仍能透過 Page Header 正確定位並保留物件。這緩解了保守式掃瞄在 Rust 激進優化下的一部分風險。
    -   **分代回收 (Generational GC)**: 區分 Minor/Major GC。UI 應用產生大量臨時物件（事件閉包、臨時樣式計算），分代 GC 能極高效地處理這些 "Die-Young" 的物件，減少 Stop-the-World 時間。

3.  **解決 Rust UI 的痛點**:
    -   透過 `Gc<T>` + `GcCell<T>`，徹底解決了 Rust GUI 中常見的 `Rc<RefCell<T>>` 泛濫與循環引用問題。這對於實作「雙向綁定」的 Widget 樹（父指子，子指父）至關重要。

### 潛在風險與限制 (Risks & Limitations)
1.  **線程模型 (Threading Model)**:
    -   目前 `Gc<T>` 是 `!Send` 且 `!Sync`。這意味著**整個 UI 邏輯、狀態管理與 Diff 計算必須運行在單一線程（主線程）**。
    -   **對於 MVP 的影響**: **幾乎沒有負面影響**。
        -   **行業標準**: 幾乎所有主流 UI 框架（瀏覽器 DOM/JS, Flutter, Android View, iOS UIKit）都強制要求 UI 操作在單一主線程 (Main Thread) 執行，以避免複雜的鎖競爭和 UI 不一致。
        -   **Vello 整合**: Vello 本身設計即支持透過 `Scene` 結構（普通的 Rust 數據，非 GC）進行通訊。UI 線程負責生成 `Scene`，渲染線程負責消費它，這天然符合 `!Send` 的邊界。
    -   **未來優化路徑**: 
        -   **Worker 線程模式**: 類似 Web Workers。將繁重的非 UI 計算任務（如圖像處理、大數據過濾）剝離到後台線程，計算結果透過 `mpsc::channel` 傳回主線程更新 UI。這是最安全且高效的模式，無需讓 `Gc` 變為 Thread-safe。
        -   **Parallel Layout**: 未來若 Taffy 佈局成為瓶頸，可考量針對性優化，但對於 MVP 規模的應用（數千個節點），單線程 Taffy 性能綽綽有餘。

2.  **延遲 (Latency / Jank)**:
    -   雖然有分代回收，但 Major GC 似乎仍採用 Stop-the-World 的 Mark-Sweep 算法。如果堆內存較大，全堆掃瞄可能導致超過 16ms 的停頓，造成掉幀。
    -   **改進需求**: 生產環境可能需要引入 **Incremental Marking (增量標記)** 來攤銷 GC 暫停時間。

3.  **安全性 (Safety - Theoretical)**:
    -   儘管有 Register Spilling，Rust/LLVM 的優化（如將指針隱藏在 Vector Registers 或進行極致的指針運算）在理論上仍可能導致保守式掃瞄失效（漏判 Root）。但在 BiBOP 架構下，此風險已較低。

## 3. 架構整合可行性評估

### Reactivity System (Solid-like Macros)
*   **可行性**: **高**
*   **分析**: SolidJS 的核心依賴於創建大量的 Signal 和 Effect 閉包。在 Rust 中，這些閉包如果捕獲環境變量，生命週期管理極其複雜。使用 `rudo-gc`，所有 Signal 都分配在 GC Heap 上，閉包捕獲 `Gc<Signal>`，形成一個自動管理的依賴圖。`Gc::new_cyclic_weak` 機制也支持了複雜的循環依賴構建。

### Layout & Rendering (Taffy + Vello)
*   **可行性**: **中**
*   **分析**:
    *   **Taffy**: Taffy 的 Style 結構體可以被包裹在 `Gc` 中。佈局計算後的結果回寫到 Layout Tree，這完全是內存操作，適配度高。
    *   **Vello**: Vello 的 Scene Encoding 通常生成一組繪圖指令（Display List）。這些指令是 plain info，可以輕鬆傳送給 Vello 的渲染線程/其上下文。關鍵在於**不要試圖將 GPU 資源 handle 直接放在 GC 中並期望跨線程自動回收**（因為 `!Send`）。需要一個明確的 Resource Bridge。

## 4. 結論

使用 `learn-projects/rudo` (easy-oilpan) 作為 `rvue` 的基礎是**技術上可行的**，且是實現 "Rust UI 聖杯"（同時擁有 Rust 的性能與 TS/JS 的開發體驗）的最有希望的路徑之一。

**建議實施路徑**:
1.  **Phase 1 (Prototype)**: 直接使用當前 `rudo-gc`。實作一個最小化的 `view!` 宏，驗證是否能寫出類似 SolidJS 的代碼並正確運行。忽略 GC 暫停問題。
2.  **Phase 2 (Optimization)**: 針對 GUI 場景優化 `rudo-gc`，主要是引入 Incremental Marking 防止掉幀。
3.  **Phase 3 (Ecosystem)**: 構建 Taffy 和 Vello 的 GC 綁定層。

**總結評分**:
*   DX (開發體驗) 潛力: ⭐⭐⭐⭐⭐
*   性能潛力: ⭐⭐⭐⭐ (取決於 GC 優化)
*   工程難度: ⭐⭐⭐⭐
