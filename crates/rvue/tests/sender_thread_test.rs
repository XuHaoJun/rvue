//! Tests for UiThreadDispatcher.
//! Verifies that dispatcher can be created and cloned.

#[cfg(feature = "async")]
mod tests {
    use rvue::async_runtime::WriteSignalUiExt;
    use rvue::create_signal;

    #[test]
    fn test_dispatcher_creation() {
        let (_, set_count) = create_signal(0i32);
        let _dispatcher = set_count.ui_dispatcher();
    }

    #[test]
    fn test_dispatcher_clone() {
        let (_, set_count) = create_signal(0i32);
        let dispatcher1 = set_count.ui_dispatcher();
        let _dispatcher2 = dispatcher1.clone();
    }
}
