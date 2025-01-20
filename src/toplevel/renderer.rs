use std::sync::Arc;

use pollster::FutureExt;

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
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .block_on()
            .ok_or(wgpu::core::instance::RequestAdapterError::NotFound)?;

        let (device, queue) = (adapter
            .request_device(
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
            )
            .block_on())?;

        let config = {
            let window_size = window.inner_size();
            let wgpu::SurfaceCapabilities {
                present_modes,
                alpha_modes,
                ..
            } = surface.get_capabilities(&adapter);
            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
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

    pub(crate) fn create_texture(&self) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: self.window.inner_size().width,
                height: self.window.inner_size().height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[wgpu::TextureFormat::Bgra8UnormSrgb],
        })
    }
}
