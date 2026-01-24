// Copyright 2025 the Rvue Authors
// SPDX-License-Identifier: Apache-2.0

//! Simple helpers for managing wgpu state and surfaces.

use wgpu::{
    Device, Instance, PresentMode, Surface, SurfaceConfiguration, Texture, TextureFormat,
    TextureUsages, TextureView,
};

use std::fmt;

/// Simple render context that maintains wgpu state for rendering the pipeline.
pub struct RenderContext {
    pub instance: Instance,
    pub devices: Vec<DeviceHandle>,
}

pub struct DeviceHandle {
    adapter: wgpu::Adapter,
    pub device: Device,
    pub queue: wgpu::Queue,
}

impl Default for RenderContext {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderContext {
    pub fn new() -> Self {
        let backends = wgpu::Backends::all();
        let flags = wgpu::InstanceFlags::from_build_config().with_env();
        let backend_options = wgpu::BackendOptions::from_env_or_default();
        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends,
            flags,
            backend_options,
            ..Default::default()
        });
        Self { instance, devices: Vec::new() }
    }

    /// Creates a new surface for the specified window and dimensions.
    pub async fn create_surface<'w>(
        &mut self,
        window: std::sync::Arc<winit::window::Window>,
        width: u32,
        height: u32,
        present_mode: PresentMode,
    ) -> Result<RenderSurface<'w>, CreateSurfaceError> {
        let surface = self.instance.create_surface(window)?;
        self.create_render_surface(surface, width, height, present_mode).await
    }

    /// Creates a new render surface for the specified window and dimensions.
    pub async fn create_render_surface<'w>(
        &mut self,
        surface: Surface<'w>,
        width: u32,
        height: u32,
        present_mode: PresentMode,
    ) -> Result<RenderSurface<'w>, CreateSurfaceError> {
        let dev_id =
            self.device(Some(&surface)).await.ok_or(CreateSurfaceError::NoCompatibleDevice)?;

        let device_handle = &self.devices[dev_id];
        let capabilities = surface.get_capabilities(&device_handle.adapter);
        // Prefer Rgba8Unorm as it's the primary target for Vello's compute shaders
        let format = capabilities
            .formats
            .iter()
            .find(|f| **f == TextureFormat::Rgba8Unorm)
            .copied()
            .or_else(|| {
                capabilities
                    .formats
                    .iter()
                    .find(|f| **f == TextureFormat::Bgra8Unorm)
                    .copied()
            })
            .ok_or(CreateSurfaceError::UnsupportedSurfaceFormat)?;

        let alpha_mode =
            if capabilities.alpha_modes.contains(&wgpu::CompositeAlphaMode::PostMultiplied) {
                wgpu::CompositeAlphaMode::PostMultiplied
            } else if capabilities.alpha_modes.contains(&wgpu::CompositeAlphaMode::PreMultiplied) {
                wgpu::CompositeAlphaMode::PreMultiplied
            } else {
                wgpu::CompositeAlphaMode::Auto
            };

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            format,
            width,
            height,
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
        };

        let (target_texture, target_view) =
            create_targets(width, height, format, &device_handle.device);

        let surface =
            RenderSurface { surface, config, dev_id, format, target_texture, target_view };
        self.configure_surface(&surface);
        Ok(surface)
    }

    /// Resizes the surface to the new dimensions.
    pub fn resize_surface(&self, surface: &mut RenderSurface<'_>, width: u32, height: u32) {
        let (texture, view) =
            create_targets(width, height, surface.format, &self.devices[surface.dev_id].device);
        surface.target_texture = texture;
        surface.target_view = view;
        surface.config.width = width;
        surface.config.height = height;
        self.configure_surface(surface);
    }

    pub fn set_present_mode(&self, surface: &mut RenderSurface<'_>, present_mode: PresentMode) {
        surface.config.present_mode = present_mode;
        self.configure_surface(surface);
    }

    fn configure_surface(&self, surface: &RenderSurface<'_>) {
        let device = &self.devices[surface.dev_id].device;
        surface.surface.configure(device, &surface.config);
    }

    /// Finds or creates a compatible device handle id.
    pub async fn device(&mut self, compatible_surface: Option<&Surface<'_>>) -> Option<usize> {
        let compatible = match compatible_surface {
            Some(s) => self
                .devices
                .iter()
                .enumerate()
                .find(|(_, d)| d.adapter.is_surface_supported(s))
                .map(|(i, _)| i),
            None => (!self.devices.is_empty()).then_some(0),
        };
        if compatible.is_none() {
            return self.new_device(compatible_surface).await;
        }
        compatible
    }

    /// Creates a compatible device handle id.
    async fn new_device(&mut self, compatible_surface: Option<&Surface<'_>>) -> Option<usize> {
        let adapter =
            wgpu::util::initialize_adapter_from_env_or_default(&self.instance, compatible_surface)
                .await
                .ok()?;
        let features = adapter.features();
        let limits = wgpu::Limits::default();
        let maybe_features = wgpu::Features::CLEAR_TEXTURE | wgpu::Features::BGRA8UNORM_STORAGE;

        let device_request = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: features & maybe_features,
            required_limits: limits,
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        });

        let (device, queue) = match device_request.await {
            Ok((d, q)) => (d, q),
            Err(_) => return None,
        };

        let device_handle = DeviceHandle { adapter, device, queue };
        self.devices.push(device_handle);
        Some(self.devices.len() - 1)
    }
}

fn create_targets(
    width: u32,
    height: u32,
    format: TextureFormat,
    device: &Device,
) -> (Texture, TextureView) {
    let target_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Intermediate Target Texture"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        usage: TextureUsages::STORAGE_BINDING
            | TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_SRC,
        format,
        view_formats: &[],
    });
    let target_view = target_texture.create_view(&wgpu::TextureViewDescriptor::default());
    (target_texture, target_view)
}

/// Errors that can occur when creating a surface.
#[derive(Debug, thiserror::Error)]
pub enum CreateSurfaceError {
    #[error("No compatible device found")]
    NoCompatibleDevice,
    #[error("Unsupported surface format")]
    UnsupportedSurfaceFormat,
    #[error("Surface creation failed: {0}")]
    SurfaceCreationError(#[from] wgpu::CreateSurfaceError),
}

/// Combination of surface and its configuration.
pub struct RenderSurface<'s> {
    pub surface: Surface<'s>,
    pub config: SurfaceConfiguration,
    pub dev_id: usize,
    pub format: TextureFormat,
    pub target_texture: Texture,
    pub target_view: TextureView,
}

impl fmt::Debug for RenderSurface<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderSurface")
            .field("surface", &self.surface)
            .field("config", &self.config)
            .field("dev_id", &self.dev_id)
            .field("format", &self.format)
            .field("target_texture", &self.target_texture)
            .field("target_view", &self.target_view)
            .finish()
    }
}
