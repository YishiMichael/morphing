use std::sync::Arc;

use itertools::Itertools;

pub struct Renderer {
    pub(crate) window: Arc<winit::window::Window>,
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    // TODO: remove pub(crate) vis
}

impl Renderer {
    pub(crate) fn new(window: winit::window::Window) -> anyhow::Result<Self> {
        let window = Arc::new(window);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone())?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .ok_or(wgpu::core::instance::RequestAdapterError::NotFound)?;

        let (device, queue) = pollster::block_on(
            adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: [
                        wgpu::Features::BUFFER_BINDING_ARRAY,
                        wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY,
                        wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    ]
                    .into_iter()
                    .fold(wgpu::Features::empty(), std::ops::BitOr::bitor),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            ),
        )?;

        let config = {
            let window_size = window.inner_size();
            let wgpu::SurfaceCapabilities {
                formats,
                present_modes,
                alpha_modes,
                ..
            } = surface.get_capabilities(&adapter);
            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: formats
                    .into_iter()
                    .find_or_first(|format| format.is_srgb())
                    .unwrap(),
                width: window_size.width,
                height: window_size.height,
                present_mode: present_modes.into_iter().next().unwrap(),
                alpha_mode: alpha_modes.into_iter().next().unwrap(),
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            }
        };
        surface.configure(&device, &config);

        Ok(Self {
            window,
            surface,
            device,
            queue,
        })
    }

    pub(crate) fn request_redraw(&self) {
        self.window.request_redraw();
    }
}
