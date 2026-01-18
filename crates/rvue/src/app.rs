//! Application runner with winit event loop

use crate::vello_util::{CreateSurfaceError, RenderContext, RenderSurface};
use crate::view::ViewStruct;
use std::sync::Arc;
use vello::peniko::Color;
use vello::{AaConfig, AaSupport, Renderer, RendererOptions};
use wgpu::SurfaceTexture;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

/// Application state
pub struct AppState<'a> {
    window: Option<Arc<Window>>,
    view: Option<ViewStruct>,
    render_cx: Option<RenderContext>,
    surface: Option<RenderSurface<'a>>,
    renderer: Option<Renderer>,
    scene: vello::Scene,
}

impl<'a> AppState<'a> {
    fn new() -> Self {
        Self {
            window: None,
            view: None,
            render_cx: None,
            surface: None,
            renderer: None,
            scene: vello::Scene::new(),
        }
    }
}

/// Application handler for winit event loop
impl ApplicationHandler for AppState<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = winit::window::Window::default_attributes()
                .with_title("Rvue Application")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

            let window = event_loop.create_window(window_attributes).unwrap();
            self.window = Some(Arc::new(window));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.handle_resize(size);
            }
            WindowEvent::RedrawRequested => {
                self.render_frame();
            }
            _ => {}
        }
    }
}

impl<'a> AppState<'a> {
    fn handle_resize(&mut self, size: PhysicalSize<u32>) {
        if let (Some(ref mut render_cx), Some(ref mut surface)) =
            (&mut self.render_cx, &mut self.surface)
        {
            render_cx.resize_surface(surface, size.width, size.height);
        }
    }

    fn render_frame(&mut self) {
        let size = match self.window.as_ref().map(|w| w.inner_size()) {
            Some(s) if s.width != 0 && s.height != 0 => s,
            _ => return,
        };

        let surface_texture = match self.get_or_create_surface(size) {
            Ok(Some(st)) => st,
            Ok(None) => return,
            Err(e) => {
                eprintln!("Rendering initialization failed: {}", e);
                return;
            }
        };

        let (render_cx, surface) = match (self.render_cx.as_mut(), self.surface.as_mut()) {
            (Some(cx), Some(s)) => (cx, s),
            _ => return,
        };

        let dev_id = surface.dev_id;
        let device = &render_cx.devices[dev_id].device;
        let queue = &render_cx.devices[dev_id].queue;
        let surface_format = surface.format;

        let render_params = vello::RenderParams {
            base_color: Color::WHITE,
            width: size.width,
            height: size.height,
            antialiasing_method: AaConfig::Area,
        };

        let renderer = self.renderer.get_or_insert_with(|| {
            let options = RendererOptions {
                surface_format: Some(surface_format),
                use_cpu: false,
                antialiasing_support: AaSupport::area_only(),
                num_init_threads: None,
            };
            Renderer::new(device, options).expect("Failed to create Vello renderer")
        });

        let scene = &self.scene;
        if let Err(e) =
            renderer.render_to_surface(device, queue, scene, &surface_texture, &render_params)
        {
            eprintln!("Vello render failed: {}", e);
            return;
        }

        if let Some(window) = &self.window {
            window.pre_present_notify();
        }
        surface_texture.present();
    }

    fn get_or_create_surface(
        &mut self,
        size: PhysicalSize<u32>,
    ) -> Result<Option<SurfaceTexture>, CreateSurfaceError> {
        if self.render_cx.is_none() {
            self.render_cx = Some(RenderContext::new());
        }

        let render_cx = self.render_cx.as_mut().unwrap();

        if self.surface.is_none() {
            let window = match &self.window {
                Some(w) => w.clone(),
                None => return Ok(None),
            };
            let new_surface = pollster::block_on(render_cx.create_surface(
                window,
                size.width,
                size.height,
                wgpu::PresentMode::AutoVsync,
            ))?;
            self.surface = Some(new_surface);
        } else {
            let surface = self.surface.as_mut().unwrap();
            if surface.config.width != size.width || surface.config.height != size.height {
                render_cx.resize_surface(surface, size.width, size.height);
            }
        }

        let surface = self.surface.as_mut().unwrap();
        match surface.surface.get_current_texture() {
            Ok(texture) => Ok(Some(texture)),
            Err(wgpu::SurfaceError::Outdated) => {
                let new_size = self.window.as_ref().map(|w| w.inner_size()).unwrap_or(size);
                render_cx.resize_surface(surface, new_size.width, new_size.height);
                match surface.surface.get_current_texture() {
                    Ok(texture) => Ok(Some(texture)),
                    Err(e) => {
                        eprintln!("Failed to get surface texture after resize: {}", e);
                        Ok(None)
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to get surface texture: {}", e);
                Ok(None)
            }
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
    let view = view_fn();

    let event_loop = EventLoop::new().map_err(|e| AppError::WindowCreationFailed(e.to_string()))?;

    let mut app_state = AppState::new();
    app_state.view = Some(view);

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
