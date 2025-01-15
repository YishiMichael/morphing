use std::sync::Arc;

pub(crate) struct Renderer {
    window: Arc<winit::window::Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Renderer {
    pub(crate) async fn new(window: winit::window::Window) -> Self {
        let window = Arc::new(window);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: [
                        wgpu::Features::BUFFER_BINDING_ARRAY,
                        wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY,
                        wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    ]
                    .iter()
                    .copied()
                    .reduce(std::ops::BitOr::bitor)
                    .unwrap(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .unwrap();

        let window_size = window.inner_size();
        let config = {
            let surface_capabilities = surface.get_capabilities(&adapter);
            // Shader code in this tutorial assumes an Srgb surface texture. Using a different
            // one will result all the colors comming out darker. If you want to support non
            // Srgb surfaces, you'll need to account for that when drawing to the frame.
            let surface_format = surface_capabilities
                .formats
                .iter()
                .copied()
                .find(|format| format.is_srgb())
                .unwrap_or(surface_capabilities.formats[0]);
            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: window_size.width,
                height: window_size.height,
                present_mode: surface_capabilities.present_modes[0],
                alpha_mode: surface_capabilities.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            }
        };
        surface.configure(&device, &config);

        Self {
            window,
            surface,
            device,
            queue,
        }
    }

    pub(crate) fn request_redraw(&self) {
        self.window.request_redraw();
    }
}

// pub(crate) struct Renderer {
//     wgpu_context: Option<WgpuContext>,
//     world: World,
// }

// impl Renderer {
//     pub(crate) fn new(world: World) -> Self {
//         Self {
//             wgpu_context: None,
//             world,
//         }
//     }

//     pub(crate) fn is_wgpu_context_initialized(&self) -> bool {
//         self.wgpu_context.is_none()
//     }

//     pub(crate) fn init_wgpu_context(&mut self, window: winit::window::Window) {
//         self.wgpu_context = Some(pollster::block_on(WgpuContext::new(window)));
//     }

//     pub(crate) fn request_redraw(&self) {
//         self.wgpu_context.as_ref().unwrap().window.request_redraw();
//     }
// }
