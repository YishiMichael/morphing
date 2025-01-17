use encase::ShaderType;
use wgpu::util::DeviceExt;

#[derive(Clone)]
pub struct Paint {
    pub color: palette::Srgba<f32>,
    pub gradients: Vec<Gradient>,
}

#[derive(Clone)]
pub struct Gradient {
    pub from: nalgebra::Vector2<f32>,
    pub to: nalgebra::Vector2<f32>,
    pub radius_slope: f32,
    pub radius_quotient: f32,
    pub radial_stops: Vec<(f32, palette::Srgba<f32>)>,
    pub angular_stops: Vec<(f32, palette::Srgba<f32>)>,
}

pub(crate) struct PaintShaderTypes {
    paint_uniform: PaintUniform,
    gradients_storage: Vec<GradientStorage>,
    radial_stops_storage: Vec<GradientStopStorage>,
    angular_stops_storage: Vec<GradientStopStorage>,
}

pub(crate) struct PaintBuffers {
    pub(crate) paint_uniform: wgpu::Buffer,
    pub(crate) gradients_storage: wgpu::Buffer,
    pub(crate) radial_stops_storage: wgpu::Buffer,
    pub(crate) angular_stops_storage: wgpu::Buffer,
}

#[derive(ShaderType)]
pub(crate) struct PaintUniform {
    color: nalgebra::Vector4<f32>,
}

#[derive(ShaderType)]
struct GradientStorage {
    from: nalgebra::Vector2<f32>,
    to: nalgebra::Vector2<f32>,
    radius_slope: f32,
    radius_quotient: f32,
    radial_stops_range: nalgebra::Vector2<u32>,
    angular_stops_range: nalgebra::Vector2<u32>,
}

#[derive(ShaderType)]
struct GradientStopStorage {
    alpha: f32,
    color: nalgebra::Vector4<f32>,
}

pub struct QueueWriteBufferMutWrapper<'a>(pub wgpu::QueueWriteBufferView<'a>); // TODO: move to another place

impl encase::internal::BufferMut for QueueWriteBufferMutWrapper<'_> {
    fn capacity(&self) -> usize {
        self.0.capacity()
    }

    fn write<const N: usize>(&mut self, offset: usize, val: &[u8; N]) {
        self.0.write(offset, val);
    }

    fn write_slice(&mut self, offset: usize, val: &[u8]) {
        self.0.write_slice(offset, val);
    }
}

impl Paint {
    pub(crate) fn to_shader_types(&self) -> PaintShaderTypes {
        #[inline]
        fn convert_color(color: palette::Srgba) -> nalgebra::Vector4<f32> {
            let (r, g, b, a) = color.into_components();
            nalgebra::Vector4::new(r, g, b, a)
        }

        //let mut gradients = Vec::new();
        let mut radial_stops_storage = Vec::new();
        let mut angular_stops_storage = Vec::new();
        let gradients_storage = self
            .gradients
            .iter()
            .scan(
                (0, 0),
                |(radial_stops_len, angular_stops_len),
                 &Gradient {
                     from,
                     to,
                     radius_slope,
                     radius_quotient,
                     ref radial_stops,
                     ref angular_stops,
                 }| {
                    let radial_stops_range =
                        *radial_stops_len..*radial_stops_len + radial_stops.len() as u32;
                    let angular_stops_range =
                        *angular_stops_len..*angular_stops_len + angular_stops.len() as u32;
                    *radial_stops_len = radial_stops_range.end;
                    *angular_stops_len = angular_stops_range.end;
                    radial_stops_storage.extend(radial_stops.iter().map(|&(alpha, color)| {
                        GradientStopStorage {
                            alpha: alpha,
                            color: convert_color(color),
                        }
                    }));
                    angular_stops_storage.extend(angular_stops.iter().map(|&(alpha, color)| {
                        GradientStopStorage {
                            alpha: alpha,
                            color: convert_color(color),
                        }
                    }));
                    Some(GradientStorage {
                        from,
                        to,
                        radius_slope,
                        radius_quotient,
                        radial_stops_range: nalgebra::Vector2::new(
                            radial_stops_range.start,
                            radial_stops_range.end,
                        ),
                        angular_stops_range: nalgebra::Vector2::new(
                            angular_stops_range.start,
                            angular_stops_range.end,
                        ),
                    })
                },
            )
            .collect();
        PaintShaderTypes {
            paint_uniform: PaintUniform {
                color: convert_color(self.color),
            },
            gradients_storage,
            radial_stops_storage,
            angular_stops_storage,
        }
    }
}

impl PaintShaderTypes {
    pub(crate) fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // pub(FRAGMENT) @binding(0) var<uniform> u_paint: PaintUniform;
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(PaintUniform::min_size()),
                    },
                    count: None,
                },
                // pub(FRAGMENT) @binding(1) var<storage> s_gradients: array<GradientStorage>;
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: Some(GradientStorage::min_size()),
                    },
                    count: None,
                },
                // pub(FRAGMENT) @binding(2) var<storage> s_radial_stops: array<GradientStopStorage>;
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: Some(GradientStopStorage::min_size()),
                    },
                    count: None,
                },
                // pub(FRAGMENT) @binding(3) var<storage> s_angular_stops: array<GradientStopStorage>;
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: Some(GradientStopStorage::min_size()),
                    },
                    count: None,
                },
            ],
        })
    }

    pub(crate) fn create_buffers_init(&self, device: &wgpu::Device) -> PaintBuffers {
        PaintBuffers {
            paint_uniform: {
                let mut buffer = encase::UniformBuffer::new(Vec::<u8>::new());
                buffer.write(&self.paint_uniform).unwrap();
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: buffer.as_ref(),
                    usage: wgpu::BufferUsages::UNIFORM,
                })
            },
            gradients_storage: {
                let mut buffer = encase::DynamicStorageBuffer::new(Vec::<u8>::new());
                buffer.write(&self.gradients_storage).unwrap();
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: buffer.as_ref(),
                    usage: wgpu::BufferUsages::STORAGE,
                })
            },
            radial_stops_storage: {
                let mut buffer = encase::DynamicStorageBuffer::new(Vec::<u8>::new());
                buffer.write(&self.radial_stops_storage).unwrap();
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: buffer.as_ref(),
                    usage: wgpu::BufferUsages::STORAGE,
                })
            },
            angular_stops_storage: {
                let mut buffer = encase::DynamicStorageBuffer::new(Vec::<u8>::new());
                buffer.write(&self.angular_stops_storage).unwrap();
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: buffer.as_ref(),
                    usage: wgpu::BufferUsages::STORAGE,
                })
            },
        }
    }

    pub(crate) fn create_buffers(&self, device: &wgpu::Device) -> PaintBuffers {
        PaintBuffers {
            paint_uniform: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: self.paint_uniform.size().get(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            gradients_storage: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: self.gradients_storage.size().get(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            radial_stops_storage: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: self.radial_stops_storage.size().get(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            angular_stops_storage: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: self.angular_stops_storage.size().get(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }

    pub(crate) fn write_buffers(&self, queue: &wgpu::Queue, buffers: &mut PaintBuffers) {
        {
            let mut buffer = encase::UniformBuffer::new(QueueWriteBufferMutWrapper(
                queue
                    .write_buffer_with(&buffers.paint_uniform, 0, self.paint_uniform.size())
                    .unwrap(),
            ));
            buffer.write(&self.paint_uniform).unwrap();
        }
        {
            let mut buffer = encase::UniformBuffer::new(QueueWriteBufferMutWrapper(
                queue
                    .write_buffer_with(&buffers.gradients_storage, 0, self.gradients_storage.size())
                    .unwrap(),
            ));
            buffer.write(&self.gradients_storage).unwrap();
        }
        {
            let mut buffer = encase::UniformBuffer::new(QueueWriteBufferMutWrapper(
                queue
                    .write_buffer_with(
                        &buffers.radial_stops_storage,
                        0,
                        self.radial_stops_storage.size(),
                    )
                    .unwrap(),
            ));
            buffer.write(&self.radial_stops_storage).unwrap();
        }
        {
            let mut buffer = encase::UniformBuffer::new(QueueWriteBufferMutWrapper(
                queue
                    .write_buffer_with(
                        &buffers.angular_stops_storage,
                        0,
                        self.angular_stops_storage.size(),
                    )
                    .unwrap(),
            ));
            buffer.write(&self.angular_stops_storage).unwrap();
        }
    }
}
