use std::sync::OnceLock;

use encase::ShaderType;
use iced::widget::shader::wgpu::util::DeviceExt;

use super::component::Component;
use super::component::ComponentShaderTypes;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Paint {
    pub color: palette::Srgba<f32>,
    pub gradients: Vec<Gradient>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Gradient {
    pub from_position: nalgebra::Vector2<f32>,
    pub to_position: nalgebra::Vector2<f32>,
    pub radius_slope: f32,
    pub radius_quotient: f32,
    pub radial_stops: Vec<(f32, palette::Srgba<f32>)>,
    pub angular_stops: Vec<(f32, palette::Srgba<f32>)>,
}

pub struct PaintBuffers {
    paint_uniform: iced::widget::shader::wgpu::Buffer,
    gradients_storage: iced::widget::shader::wgpu::Buffer,
    radial_stops_storage: iced::widget::shader::wgpu::Buffer,
    angular_stops_storage: iced::widget::shader::wgpu::Buffer,
}

pub struct PaintShaderTypes {
    paint_uniform: PaintUniform,
    gradients_storage: Vec<GradientStorage>,
    radial_stops_storage: Vec<GradientStopStorage>,
    angular_stops_storage: Vec<GradientStopStorage>,
}

#[derive(ShaderType)]
struct PaintUniform {
    color: nalgebra::Vector4<f32>,
}

#[derive(ShaderType)]
struct GradientStorage {
    from_position: nalgebra::Vector2<f32>,
    to_position: nalgebra::Vector2<f32>,
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

pub struct QueueWriteBufferMutWrapper<'a>(pub iced::widget::shader::wgpu::QueueWriteBufferView<'a>); // TODO: move to another place

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

impl Component for Paint {
    type ShaderTypes = PaintShaderTypes;

    fn to_shader_types(&self) -> Self::ShaderTypes {
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
                     from_position,
                     to_position,
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
                        from_position,
                        to_position,
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

static PAINT_BIND_GROUP_LAYOUT: OnceLock<iced::widget::shader::wgpu::BindGroupLayout> =
    OnceLock::new();

impl ComponentShaderTypes for PaintShaderTypes {
    type Buffers = PaintBuffers;

    fn bind_group_layout(
        device: &iced::widget::shader::wgpu::Device,
    ) -> &'static iced::widget::shader::wgpu::BindGroupLayout {
        PAINT_BIND_GROUP_LAYOUT.get_or_init(|| {
            device.create_bind_group_layout(
                &iced::widget::shader::wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        // pub(FRAGMENT) @binding(0) var<uniform> u_paint: PaintUniform;
                        iced::widget::shader::wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: iced::widget::shader::wgpu::ShaderStages::FRAGMENT,
                            ty: iced::widget::shader::wgpu::BindingType::Buffer {
                                ty: iced::widget::shader::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: Some(PaintUniform::min_size()),
                            },
                            count: None,
                        },
                        // pub(FRAGMENT) @binding(1) var<storage> s_gradients: array<GradientStorage>;
                        iced::widget::shader::wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: iced::widget::shader::wgpu::ShaderStages::FRAGMENT,
                            ty: iced::widget::shader::wgpu::BindingType::Buffer {
                                ty: iced::widget::shader::wgpu::BufferBindingType::Storage {
                                    read_only: true,
                                },
                                has_dynamic_offset: false,
                                min_binding_size: Some(GradientStorage::min_size()),
                            },
                            count: None,
                        },
                        // pub(FRAGMENT) @binding(2) var<storage> s_radial_stops: array<GradientStopStorage>;
                        iced::widget::shader::wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: iced::widget::shader::wgpu::ShaderStages::FRAGMENT,
                            ty: iced::widget::shader::wgpu::BindingType::Buffer {
                                ty: iced::widget::shader::wgpu::BufferBindingType::Storage {
                                    read_only: true,
                                },
                                has_dynamic_offset: false,
                                min_binding_size: Some(GradientStopStorage::min_size()),
                            },
                            count: None,
                        },
                        // pub(FRAGMENT) @binding(3) var<storage> s_angular_stops: array<GradientStopStorage>;
                        iced::widget::shader::wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: iced::widget::shader::wgpu::ShaderStages::FRAGMENT,
                            ty: iced::widget::shader::wgpu::BindingType::Buffer {
                                ty: iced::widget::shader::wgpu::BufferBindingType::Storage {
                                    read_only: true,
                                },
                                has_dynamic_offset: false,
                                min_binding_size: Some(GradientStopStorage::min_size()),
                            },
                            count: None,
                        },
                    ],
                },
            )
        })
    }

    fn bind_group_from_buffers(
        device: &iced::widget::shader::wgpu::Device,
        buffers: &Self::Buffers,
    ) -> iced::widget::shader::wgpu::BindGroup {
        device.create_bind_group(&iced::widget::shader::wgpu::BindGroupDescriptor {
            label: None,
            layout: PaintShaderTypes::bind_group_layout(device),
            entries: &[
                iced::widget::shader::wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.paint_uniform.as_entire_binding(),
                },
                iced::widget::shader::wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.gradients_storage.as_entire_binding(),
                },
                iced::widget::shader::wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffers.radial_stops_storage.as_entire_binding(),
                },
                iced::widget::shader::wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffers.angular_stops_storage.as_entire_binding(),
                },
            ],
        })
    }

    fn new_buffers(&self, device: &iced::widget::shader::wgpu::Device) -> Self::Buffers {
        PaintBuffers {
            paint_uniform: device.create_buffer(&iced::widget::shader::wgpu::BufferDescriptor {
                label: None,
                size: self.paint_uniform.size().get(),
                usage: iced::widget::shader::wgpu::BufferUsages::UNIFORM
                    | iced::widget::shader::wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            gradients_storage: device.create_buffer(
                &iced::widget::shader::wgpu::BufferDescriptor {
                    label: None,
                    size: self.gradients_storage.size().get(),
                    usage: iced::widget::shader::wgpu::BufferUsages::STORAGE
                        | iced::widget::shader::wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            ),
            radial_stops_storage: device.create_buffer(
                &iced::widget::shader::wgpu::BufferDescriptor {
                    label: None,
                    size: self.radial_stops_storage.size().get(),
                    usage: iced::widget::shader::wgpu::BufferUsages::STORAGE
                        | iced::widget::shader::wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            ),
            angular_stops_storage: device.create_buffer(
                &iced::widget::shader::wgpu::BufferDescriptor {
                    label: None,
                    size: self.angular_stops_storage.size().get(),
                    usage: iced::widget::shader::wgpu::BufferUsages::STORAGE
                        | iced::widget::shader::wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            ),
        }
    }

    fn initialize_buffers(&self, device: &iced::widget::shader::wgpu::Device) -> Self::Buffers {
        PaintBuffers {
            paint_uniform: {
                let mut buffer = encase::UniformBuffer::new(Vec::<u8>::new());
                buffer.write(&self.paint_uniform).unwrap();
                device.create_buffer_init(&iced::widget::shader::wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: buffer.as_ref(),
                    usage: iced::widget::shader::wgpu::BufferUsages::UNIFORM,
                })
            },
            gradients_storage: {
                let mut buffer = encase::DynamicStorageBuffer::new(Vec::<u8>::new());
                buffer.write(&self.gradients_storage).unwrap();
                device.create_buffer_init(&iced::widget::shader::wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: buffer.as_ref(),
                    usage: iced::widget::shader::wgpu::BufferUsages::STORAGE,
                })
            },
            radial_stops_storage: {
                let mut buffer = encase::DynamicStorageBuffer::new(Vec::<u8>::new());
                buffer.write(&self.radial_stops_storage).unwrap();
                device.create_buffer_init(&iced::widget::shader::wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: buffer.as_ref(),
                    usage: iced::widget::shader::wgpu::BufferUsages::STORAGE,
                })
            },
            angular_stops_storage: {
                let mut buffer = encase::DynamicStorageBuffer::new(Vec::<u8>::new());
                buffer.write(&self.angular_stops_storage).unwrap();
                device.create_buffer_init(&iced::widget::shader::wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: buffer.as_ref(),
                    usage: iced::widget::shader::wgpu::BufferUsages::STORAGE,
                })
            },
        }
    }

    fn write_buffers(
        &self,
        queue: &iced::widget::shader::wgpu::Queue,
        buffers: &mut Self::Buffers,
    ) {
        {
            let mut buffer = encase::UniformBuffer::new(QueueWriteBufferMutWrapper(
                queue
                    .write_buffer_with(&buffers.paint_uniform, 0, self.paint_uniform.size())
                    .unwrap(),
            ));
            buffer.write(&self.paint_uniform).unwrap();
        }
        {
            let mut buffer = encase::DynamicStorageBuffer::new(QueueWriteBufferMutWrapper(
                queue
                    .write_buffer_with(&buffers.gradients_storage, 0, self.gradients_storage.size())
                    .unwrap(),
            ));
            buffer.write(&self.gradients_storage).unwrap();
        }
        {
            let mut buffer = encase::DynamicStorageBuffer::new(QueueWriteBufferMutWrapper(
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
            let mut buffer = encase::DynamicStorageBuffer::new(QueueWriteBufferMutWrapper(
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
