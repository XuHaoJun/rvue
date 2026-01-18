//! Application runner with winit event loop

use crate::render::Scene as RvueScene;
use crate::vello_util::{CreateSurfaceError, RenderContext, RenderSurface};
use crate::view::ViewStruct;
use std::sync::Arc;
use vello::kurbo::Affine;
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
    scene: RvueScene,
}

impl<'a> AppState<'a> {
    fn new() -> Self {
        Self {
            window: None,
            view: None,
            render_cx: None,
            surface: None,
            renderer: None,
            scene: RvueScene::new(),
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
        let (scale_factor, size) =
            match self.window.as_ref().map(|w| (w.scale_factor(), w.inner_size())) {
                Some((sf, s)) if s.width != 0 && s.height != 0 => (sf, s),
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
        let _surface_format = surface.format;

        let render_params = vello::RenderParams {
            base_color: Color::WHITE,
            width: size.width,
            height: size.height,
            antialiasing_method: AaConfig::Area,
        };

        let renderer = self.renderer.get_or_insert_with(|| {
            let options = RendererOptions {
                use_cpu: false,
                antialiasing_support: AaSupport::area_only(),
                num_init_threads: None,
                pipeline_cache: None,
            };
            Renderer::new(device, options).expect("Failed to create Vello renderer")
        });

        // Populate scene from view if not already done
        if self.scene.fragments.is_empty() {
            if let Some(view) = &self.view {
                self.scene.add_fragment(view.root_component.clone());
            }
        }

        // Update scene (regenerates the underlying vello::Scene if dirty)
        self.scene.update();

        let scene = self.scene.vello_scene();

        let transformed_scene = if scale_factor == 1.0 {
            None
        } else {
            let mut new_scene = vello::Scene::new();
            new_scene.append(scene, Some(Affine::scale(scale_factor)));
            Some(new_scene)
        };
        let scene_ref = transformed_scene.as_ref().unwrap_or(scene);

        // Render to intermediate texture
        if let Err(e) = renderer.render_to_texture(
            device,
            queue,
            scene_ref,
            &surface.target_view,
            &render_params,
        ) {
            eprintln!("Vello render to texture failed: {}", e);
            return;
        }

        // Blit to surface
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Surface Blit"),
        });

        if surface.format == wgpu::TextureFormat::Rgba8Unorm {
            // Simple copy if formats match
            encoder.copy_texture_to_texture(
                surface.target_texture.as_image_copy(),
                surface_texture.texture.as_image_copy(),
                wgpu::Extent3d { width: size.width, height: size.height, depth_or_array_layers: 1 },
            );
        } else {
            // Fallback for non-matching formats: for now we just warn.
            // In a full implementation, we'd use a render pipeline or TextureBlitter for format conversion.
            eprintln!("Warning: Surface format {:?} doesn't match intermediate target format Rgba8Unorm. Blit might fail or be incorrect.", surface.format);
            encoder.copy_texture_to_texture(
                surface.target_texture.as_image_copy(),
                surface_texture.texture.as_image_copy(),
                wgpu::Extent3d { width: size.width, height: size.height, depth_or_array_layers: 1 },
            );
        }

        queue.submit([encoder.finish()]);

        if let Some(window) = &self.window {
            window.pre_present_notify();
        }
        surface_texture.present();

        // GPU synchronization
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
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
