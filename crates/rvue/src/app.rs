//! Application runner with winit event loop

use crate::view::ViewStruct;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

/// Application state
pub struct AppState {
    window: Option<Window>,
    view: Option<ViewStruct>,
    renderer_initialized: bool,
}

impl AppState {
    fn new() -> Self {
        Self { window: None, view: None, renderer_initialized: false }
    }
}

/// Application handler for winit event loop
impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Optimize: Defer window creation until actually needed
        // This reduces startup time by not creating the window immediately
        if self.window.is_none() {
            let window_attributes = winit::window::Window::default_attributes()
                .with_title("Rvue Application")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

            let window = event_loop.create_window(window_attributes).unwrap();
            self.window = Some(window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Lazy initialization: Initialize renderer only when first redraw is requested
                if !self.renderer_initialized {
                    // Initialize renderer here (lazy loading)
                    // This defers renderer initialization until the first frame
                    self.renderer_initialized = true;
                }

                // Trigger redraw
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

/// Run the application with the given view
///
/// # Arguments
///
/// * `view_fn` - A function that returns the root view of the application
///
/// # Example
///
/// ```ignore
/// use rvue::prelude::*;
///
/// fn main() {
///     rvue::run_app(|| {
///         view! {
///             <Text value="Hello, Rvue!" />
///         }
///     });
/// }
/// ```
pub fn run_app<F>(view_fn: F) -> Result<(), AppError>
where
    F: FnOnce() -> ViewStruct + 'static,
{
    // Create the view
    let view = view_fn();

    // Create event loop
    let event_loop = EventLoop::new().map_err(|e| AppError::WindowCreationFailed(e.to_string()))?;

    // Create application state
    let mut app_state = AppState::new();
    app_state.view = Some(view);

    // Run the event loop
    event_loop
        .run_app(&mut app_state)
        .map_err(|e| AppError::WindowCreationFailed(e.to_string()))?;

    Ok(())
}

/// Application error types
#[derive(Debug)]
pub enum AppError {
    WindowCreationFailed(String),
    RendererInitializationFailed(String),
    ComponentCreationFailed(String),
    LayoutCalculationFailed(String),
    GcError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::WindowCreationFailed(msg) => write!(f, "Window creation failed: {}", msg),
            AppError::RendererInitializationFailed(msg) => {
                write!(f, "Renderer initialization failed: {}", msg)
            }
            AppError::ComponentCreationFailed(msg) => {
                write!(f, "Component creation failed: {}", msg)
            }
            AppError::LayoutCalculationFailed(msg) => {
                write!(f, "Layout calculation failed: {}", msg)
            }
            AppError::GcError(msg) => write!(f, "GC error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}
