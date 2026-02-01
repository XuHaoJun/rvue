//! Application runner with winit event loop

use crate::component::{Component, ComponentLifecycle};
use crate::event::context::EventContextOps;
use crate::event::dispatch::{run_pointer_event_pass, run_text_event_pass};
use crate::event::hit_test::hit_test;
use crate::event::types::{
    map_scroll_delta, KeyboardEvent, PointerButtonEvent, PointerEvent, PointerMoveEvent,
};
use crate::event::update::{run_update_focus_pass, run_update_pointer_pass};
use crate::event::winit_translator::{get_pointer_event_position, WinitTranslator};
use crate::render::Scene as RvueScene;
use crate::style::Stylesheet;
use crate::vello_util::{CreateSurfaceError, RenderContext, RenderSurface};
use crate::view::ViewStruct;
use rudo_gc::{Gc, GcCell};
use std::cell::RefMut;
use std::sync::Arc;
use vello::kurbo::Affine;
use vello::kurbo::{Point, Vec2};
use vello::peniko::Color;
use vello::{AaConfig, AaSupport, Renderer, RendererOptions};
use wgpu::Color as WgpuColor;
use wgpu::SurfaceTexture;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::ModifiersState;
use winit::window::{Window, WindowId};

pub trait AppStateLike {
    fn root_component(&self) -> Gc<Component>;
    fn pointer_capture(&self) -> Option<Gc<Component>>;
    fn pointer_capture_mut(&mut self) -> RefMut<'_, Option<Gc<Component>>>;
    fn last_pointer_pos(&self) -> Option<Point>;
    fn hovered_component(&self) -> Option<Gc<Component>>;
    fn focused(&self) -> Option<Gc<Component>>;
    fn focused_mut(&mut self) -> &mut Option<Gc<Component>>;
    fn fallback(&self) -> Option<Gc<Component>>;
    fn pending_focus(&mut self) -> &mut Option<Gc<Component>>;
    fn active_path(&mut self) -> &mut Vec<Gc<Component>>;
    fn hovered_path(&mut self) -> &mut Vec<Gc<Component>>;
    fn focused_path(&mut self) -> &mut Vec<Gc<Component>>;
    fn set_active_path(&mut self, path: Vec<Gc<Component>>);
    fn set_hovered_path(&mut self, path: Vec<Gc<Component>>);
    fn set_focused_path(&mut self, path: Vec<Gc<Component>>);
    fn set_needs_pointer_pass_update(&mut self, _value: bool);
    fn needs_pointer_pass_update(&self) -> bool;
    fn set_focused(&mut self, focused: Option<Gc<Component>>);
    fn clear_pointer_capture(&mut self);
}

pub struct FocusState {
    pub focused: Option<Gc<Component>>,
    pub fallback: Option<Gc<Component>>,
    pub pending_focus: Option<Gc<Component>>,
    pub focus_anchor: Option<Gc<Component>>,
}

/// Application state
/// Fields are ordered for correct drop order: GC resources first, window last
pub struct AppState<'a> {
    view: Option<ViewStruct>,
    scene: RvueScene,
    pub stylesheet: Option<Stylesheet>,
    pub focus_state: FocusState,
    pub pointer_capture: GcCell<Option<Gc<Component>>>,
    pub last_pointer_pos: Option<Point>,
    pub hovered_component: GcCell<Option<Gc<Component>>>,
    pub active_path: Vec<Gc<Component>>,
    pub hovered_path: Vec<Gc<Component>>,
    pub focused_path: Vec<Gc<Component>>,
    pub needs_pointer_pass_update: bool,
    pub last_gc_count: usize,
    renderer: Option<Renderer>,
    surface: Option<RenderSurface<'a>>,
    render_cx: Option<RenderContext>,
    window: Option<Arc<Window>>,
    event_translator: WinitTranslator,
}

impl<'a> AppStateLike for AppState<'a> {
    fn root_component(&self) -> Gc<Component> {
        self.view
            .as_ref()
            .map(|v| v.root_component.clone())
            .unwrap_or_else(|| panic!("No root component"))
    }

    fn pointer_capture(&self) -> Option<Gc<Component>> {
        self.pointer_capture.borrow().clone()
    }

    fn pointer_capture_mut(&mut self) -> RefMut<'_, Option<Gc<Component>>> {
        self.pointer_capture.borrow_mut()
    }

    fn last_pointer_pos(&self) -> Option<Point> {
        self.last_pointer_pos
    }

    fn hovered_component(&self) -> Option<Gc<Component>> {
        self.hovered_component.borrow().clone()
    }

    fn focused(&self) -> Option<Gc<Component>> {
        self.focus_state.focused.clone()
    }

    fn focused_mut(&mut self) -> &mut Option<Gc<Component>> {
        &mut self.focus_state.focused
    }

    fn fallback(&self) -> Option<Gc<Component>> {
        self.focus_state.fallback.clone()
    }

    fn pending_focus(&mut self) -> &mut Option<Gc<Component>> {
        &mut self.focus_state.pending_focus
    }

    fn active_path(&mut self) -> &mut Vec<Gc<Component>> {
        &mut self.active_path
    }

    fn hovered_path(&mut self) -> &mut Vec<Gc<Component>> {
        &mut self.hovered_path
    }

    fn focused_path(&mut self) -> &mut Vec<Gc<Component>> {
        &mut self.focused_path
    }

    fn set_active_path(&mut self, path: Vec<Gc<Component>>) {
        self.active_path = path;
    }

    fn set_hovered_path(&mut self, path: Vec<Gc<Component>>) {
        self.hovered_path = path;
    }

    fn set_focused_path(&mut self, path: Vec<Gc<Component>>) {
        self.focused_path = path;
    }

    fn set_needs_pointer_pass_update(&mut self, value: bool) {
        self.needs_pointer_pass_update = value;
    }

    fn needs_pointer_pass_update(&self) -> bool {
        self.needs_pointer_pass_update
    }

    fn set_focused(&mut self, focused: Option<Gc<Component>>) {
        self.focus_state.focused = focused;
    }

    fn clear_pointer_capture(&mut self) {
        *self.pointer_capture.borrow_mut() = None;
    }
}

impl EventContextOps for AppState<'_> {
    fn request_paint(&mut self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn request_layout(&mut self) {
        if let Some(view) = &self.view {
            view.root_component.mark_dirty();
        }
        self.request_paint();
    }

    fn capture_pointer(&mut self, component: Gc<Component>) {
        *self.pointer_capture.borrow_mut() = Some(component);
    }

    fn release_pointer(&mut self) {
        *self.pointer_capture.borrow_mut() = None;
    }

    fn request_focus(&mut self) {
        // Focus will be applied in the next update pass
    }

    fn resign_focus(&mut self) {
        self.focus_state.focused = None;
    }

    fn set_handled(&mut self) {}

    fn is_handled(&self) -> bool {
        false
    }

    fn target(&self) -> Gc<Component> {
        self.root_component()
    }

    fn local_position(&self, window_pos: Point) -> Point {
        window_pos
    }

    fn has_pointer_capture(&self) -> bool {
        false
    }
}

impl<'a> AppState<'a> {
    fn new() -> Self {
        Self {
            renderer: None,
            surface: None,
            render_cx: None,
            window: None,
            view: None,
            scene: RvueScene::new(),
            stylesheet: None,
            focus_state: FocusState {
                focused: None,
                fallback: None,
                pending_focus: None,
                focus_anchor: None,
            },
            pointer_capture: GcCell::new(None),
            last_pointer_pos: None,
            hovered_component: GcCell::new(None),
            active_path: Vec::new(),
            hovered_path: Vec::new(),
            focused_path: Vec::new(),
            needs_pointer_pass_update: false,
            last_gc_count: 0,
            event_translator: WinitTranslator::new(),
        }
    }

    fn handle_translated_pointer_event(
        &mut self,
        event: &ui_events::pointer::PointerEvent,
        scale_factor: f64,
    ) {
        if let Some(pos) = get_pointer_event_position(event) {
            let logical_x = pos.x / scale_factor;
            let logical_y = pos.y / scale_factor;
            let logical_pos = Point::new(logical_x, logical_y);
            self.last_pointer_pos = Some(logical_pos);

            let new_hovered = hit_test(&self.root_component(), logical_pos);
            *self.hovered_component.borrow_mut() = new_hovered;
        }

        self.scene.update();

        let converted_event =
            crate::event::types::convert_pointer_event_from_ui_events(event, scale_factor);

        run_pointer_event_pass(self, &converted_event);
        self.request_redraw_if_dirty();
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
        let scale_factor = self.window.as_ref().map(|w| w.scale_factor()).unwrap_or(1.0);

        if let Some(translated) = self.event_translator.translate(scale_factor, &event) {
            match translated {
                ui_events_winit::WindowEventTranslation::Pointer(pointer_event) => {
                    self.handle_translated_pointer_event(&pointer_event, scale_factor);
                    return;
                }
                ui_events_winit::WindowEventTranslation::Keyboard(_) => {}
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                // Clear GC resources before exiting to prevent double-free
                self.active_path.clear();
                self.hovered_path.clear();
                self.focused_path.clear();
                *self.pointer_capture.borrow_mut() = None;
                *self.hovered_component.borrow_mut() = None;
                self.scene.root_components.clear();
                self.scene.vello_scene = None;
                self.view = None;
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.handle_resize(size);
            }
            WindowEvent::RedrawRequested => {
                self.run_update_passes();
                self.render_frame();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let scale_factor = self.window.as_ref().map(|w| w.scale_factor()).unwrap_or(1.0);
                let logical_x = position.x / scale_factor;
                let logical_y = position.y / scale_factor;
                let point = Point::new(logical_x, logical_y);
                self.last_pointer_pos = Some(point);

                // Ensure layout is up to date before hit testing
                self.scene.update();

                let new_hovered = hit_test(&self.root_component(), point);
                *self.hovered_component.borrow_mut() = new_hovered;

                let event = PointerEvent::Move(PointerMoveEvent {
                    position: point,
                    delta: Vec2::ZERO,
                    modifiers: self.current_modifiers(),
                });
                run_pointer_event_pass(self, &event);
                self.request_redraw_if_dirty();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let position = self.last_pointer_pos.unwrap_or_default();

                let event = match state {
                    ElementState::Pressed => PointerEvent::Down(PointerButtonEvent {
                        button: button.into(),
                        position,
                        click_count: 1,
                        modifiers: self.current_modifiers(),
                    }),
                    ElementState::Released => PointerEvent::Up(PointerButtonEvent {
                        button: button.into(),
                        position,
                        click_count: 1,
                        modifiers: self.current_modifiers(),
                    }),
                };

                // Ensure layout is up to date before event dispatch (which includes hit testing)
                self.scene.update();

                run_pointer_event_pass(self, &event);
                self.request_redraw_if_dirty();
            }
            WindowEvent::CursorEntered { .. } => {
                run_pointer_event_pass(self, &PointerEvent::Enter(Default::default()));
            }
            WindowEvent::CursorLeft { .. } => {
                *self.hovered_component.borrow_mut() = None;
                run_pointer_event_pass(self, &PointerEvent::Leave(Default::default()));
            }
            WindowEvent::KeyboardInput { event: input, .. } => {
                let key_event = KeyboardEvent {
                    key: input.logical_key,
                    code: input.physical_key,
                    state: input.state.into(),
                    modifiers: self.current_modifiers(),
                    repeat: input.repeat,
                };
                run_text_event_pass(self, &crate::event::types::TextEvent::Keyboard(key_event));
                self.request_redraw_if_dirty();
            }
            WindowEvent::Ime(ime_event) => {
                let ime = match ime_event {
                    winit::event::Ime::Enabled => {
                        crate::event::types::ImeEvent::Enabled(crate::event::types::ImeCause::Other)
                    }
                    winit::event::Ime::Preedit(text, cursor) => {
                        crate::event::types::ImeEvent::Preedit(text, cursor.map_or(0, |c| c.0))
                    }
                    winit::event::Ime::Commit(text) => crate::event::types::ImeEvent::Commit(text),
                    winit::event::Ime::Disabled => crate::event::types::ImeEvent::Disabled,
                };
                run_text_event_pass(self, &crate::event::types::TextEvent::Ime(ime));
                self.request_redraw_if_dirty();
            }
            WindowEvent::Focused(focused) => {
                if !focused {
                    *self.pointer_capture.borrow_mut() = None;
                    *self.hovered_component.borrow_mut() = None;
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let event = PointerEvent::Scroll(crate::event::types::PointerScrollEvent {
                    delta: map_scroll_delta(delta),
                    position: self.last_pointer_pos.unwrap_or_default(),
                    modifiers: self.current_modifiers(),
                });
                run_pointer_event_pass(self, &event);
                self.request_redraw_if_dirty();
            }
            _ => {}
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let (Some(ref render_cx), Some(ref surface)) = (&self.render_cx, &self.surface) {
            let dev_id = surface.dev_id;
            let device = &render_cx.devices[dev_id].device;
            let _ = device.poll(wgpu::PollType::Poll);
        }
    }
}

impl<'a> Drop for AppState<'a> {
    fn drop(&mut self) {
        // Clear all component paths first - these hold Gc references
        self.active_path.clear();
        self.hovered_path.clear();
        self.focused_path.clear();

        // Clear pointer capture and hovered component
        *self.pointer_capture.borrow_mut() = None;
        *self.hovered_component.borrow_mut() = None;

        // Clear the scene's root components to break the reference chain
        self.scene.root_components.clear();
        self.scene.vello_scene = None;

        // Clear the view to break the reference to root component
        self.view = None;

        // GC is disabled during app lifetime, so no need to run cleanup
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

    fn run_update_passes(&mut self) {
        if self.needs_pointer_pass_update {
            run_update_pointer_pass(self);
            self.needs_pointer_pass_update = false;
        }
        run_update_focus_pass(self);

        // Process component lifecycle updates and effects
        self.root_component().update();

        // Monitor GC performance
        self.monitor_gc();
    }

    fn monitor_gc(&mut self) {
        let metrics = rudo_gc::last_gc_metrics();
        if metrics.total_collections > self.last_gc_count {
            self.last_gc_count = metrics.total_collections;

            let duration_ms = metrics.duration.as_millis();
            if duration_ms > 16 {
                eprintln!("WARNING: GC pause of {}ms exceeded frame budget (16ms)!", duration_ms);
            }
        }
    }

    fn request_redraw_if_dirty(&self) {
        let root_dirty = self.view.as_ref().map(|v| v.root_component.is_dirty()).unwrap_or(false);
        if root_dirty {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    fn current_modifiers(&self) -> crate::event::types::Modifiers {
        ModifiersState::default().into()
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
        if self.scene.root_components.is_empty() {
            if let Some(view) = &self.view {
                self.scene.add_fragment(view.root_component.clone());
            }
        }

        // Set stylesheet if not already set
        if self.scene.stylesheet.is_none() {
            if let Some(stylesheet) = &self.stylesheet {
                self.scene.set_stylesheet(stylesheet.clone());
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

        // Clear intermediate texture before rendering
        {
            let mut clear_encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Clear Texture"),
                });
            {
                let _clear_pass = clear_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Clear Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &surface.target_view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(WgpuColor::WHITE),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            }
            queue.submit([clear_encoder.finish()]);
        }

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

        // Intermediate texture format now matches the surface format
        encoder.copy_texture_to_texture(
            surface.target_texture.as_image_copy(),
            surface_texture.texture.as_image_copy(),
            wgpu::Extent3d { width: size.width, height: size.height, depth_or_array_layers: 1 },
        );

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

        if let Some(surface) = self.surface.as_mut() {
            if surface.config.width != size.width || surface.config.height != size.height {
                render_cx.resize_surface(surface, size.width, size.height);
            }
        } else {
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
    // Disable automatic GC collection during app lifetime to prevent
    // race conditions with component drops
    // The final cleanup in AppState::drop will re-enable and run GC
    rudo_gc::set_collect_condition(|_| false);

    let view = view_fn();

    let event_loop = EventLoop::with_user_event()
        .build()
        .map_err(|e| AppError::WindowCreationFailed(e.to_string()))?;

    let mut app_state = AppState::new();
    app_state.view = Some(view);

    // Add default stylesheet for component sizing (buttons, inputs, etc.)
    app_state.stylesheet = Some(Stylesheet::with_defaults());

    // Run the event loop - AppState::drop will handle cleanup
    event_loop
        .run_app(&mut app_state)
        .map_err(|e| AppError::WindowCreationFailed(e.to_string()))?;

    Ok(())
}

/// Run the application with a stylesheet for CSS selector matching.
///
/// # Arguments
///
/// * `view_fn` - A function that returns the root view of the application
/// * `stylesheet` - An optional stylesheet for CSS selector-based styling
///
/// # Example
///
/// ```ignore
/// use rvue::prelude::*;
/// use rvue_style::{Stylesheet, BackgroundColor, Color};
///
/// fn main() {
///     let mut stylesheet = Stylesheet::new();
///     stylesheet.add_rule("button.primary", Properties::with(
///         BackgroundColor(Color::rgb(0, 123, 255))
///     ));
///     stylesheet.add_rule("button:hover", Properties::with(
///         BackgroundColor(Color::rgb(0, 86, 179))
///     ));
///
///     rvue::run_app_with_stylesheet(|| {
///         view! {
///             <Button class="primary">
///                 <Text>Primary</Text>
///             </Button>
///         }
///     }, Some(stylesheet));
/// }
/// ```
pub fn run_app_with_stylesheet<F>(
    view_fn: F,
    stylesheet: Option<Stylesheet>,
) -> Result<(), AppError>
where
    F: FnOnce() -> ViewStruct + 'static,
{
    rudo_gc::set_collect_condition(|_| false);

    let view = view_fn();

    let event_loop = EventLoop::with_user_event()
        .build()
        .map_err(|e| AppError::WindowCreationFailed(e.to_string()))?;

    let mut app_state = AppState::new();
    app_state.view = Some(view);

    let merged_stylesheet = match stylesheet {
        Some(user_sheet) => {
            let mut defaults = Stylesheet::with_defaults();
            defaults.merge(&user_sheet);
            defaults
        }
        None => Stylesheet::with_defaults(),
    };
    app_state.stylesheet = Some(merged_stylesheet);

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
