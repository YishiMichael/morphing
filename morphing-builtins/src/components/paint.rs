use super::{
    color::Color,
    component::{Component, ComponentShaderTypes},
};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Paint {
    pub color: Color,
    pub gradients: Vec<Gradient>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Gradient {
    pub from_position: nalgebra::Vector2<f32>,
    pub to_position: nalgebra::Vector2<f32>,
    pub radius_slope: f32,
    pub radius_quotient: f32,
    pub radial_stops: Vec<(f32, Color)>,
    pub angular_stops: Vec<(f32, Color)>,
}

// pub struct PaintBuffers {
//     color_uniform: wgpu::Buffer,
//     gradients_storage: wgpu::Buffer,
//     radial_stops_storage: wgpu::Buffer,
//     angular_stops_storage: wgpu::Buffer,
// }

// pub struct PaintShaderTypes {
//     color_uniform: ColorUniform,
//     gradients_storage: Vec<GradientStorage>,
//     radial_stops_storage: Vec<GradientStopStorage>,
//     angular_stops_storage: Vec<GradientStopStorage>,
// }

wgpu_struct! {
    struct ColorUniform {
        color: vec4<f32>,
    }

    struct GradientStorage {
        from_position: vec2<f32>,
        to_position: vec2<f32>,
        radius_slope: f32,
        radius_quotient: f32,
        radial_stops_range: vec2<u32>,
        angular_stops_range: vec2<u32>,
    }

    struct GradientStopStorage {
        alpha: f32,
        color: vec4<f32>,
    }
}

// wgpu_shader_types! {
//     pub struct PaintShaderTypes {
//         pub(FRAGMENT) @binding(0) var<uniform> u_paint: ColorUniform,
//         pub(FRAGMENT) @binding(1) var<storage> s_gradients: array<GradientStorage>,
//         pub(FRAGMENT) @binding(2) var<storage> s_radial_stops: array<GradientStopStorage>,
//         pub(FRAGMENT) @binding(3) var<storage> s_angular_stops: array<GradientStopStorage>,
//     }
// }

pub struct PaintBuffers {
    color_uniform: wgpu::Buffer,
    gradients_storage: wgpu::Buffer,
    radial_stops_storage: wgpu::Buffer,
    angular_stops_storage: wgpu::Buffer,
}

pub struct PaintShaderTypes {
    u_paint: ColorUniform,
    s_gradients: Vec<GradientStorage>,
    s_radial_stops: Vec<GradientStopStorage>,
    s_angular_stops: Vec<GradientStopStorage>,
}

impl Component for Paint {
    type ShaderTypes = PaintShaderTypes;

    fn to_shader_types(&self) -> Self::ShaderTypes {
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
                            color: color.into(),
                        }
                    }));
                    angular_stops_storage.extend(angular_stops.iter().map(|&(alpha, color)| {
                        GradientStopStorage {
                            alpha: alpha,
                            color: color.into(),
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
            color_uniform: ColorUniform {
                color: self.color.into(),
            },
            gradients_storage,
            radial_stops_storage,
            angular_stops_storage,
        }
    }
}

static PAINT_BIND_GROUP_LAYOUT: ::std::sync::OnceLock<wgpu::BindGroupLayout> =
    ::std::sync::OnceLock::new();

impl ComponentShaderTypes for PaintShaderTypes {
    type Buffers = PaintBuffers;

    fn bind_group_layout(device: &wgpu::Device) -> &'static wgpu::BindGroupLayout {
        PAINT_BIND_GROUP_LAYOUT.get_or_init(|| {
            device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        // pub(FRAGMENT) @binding(0) var<uniform> u_paint: ColorUniform;
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: Some(
                                    <ColorUniform as encase::ShaderType>::min_size(),
                                ),
                            },
                            count: None,
                        },
                        // pub(FRAGMENT) @binding(1) var<storage> s_gradients: array<GradientStorage>;
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage {
                                    read_only: true,
                                },
                                has_dynamic_offset: false,
                                min_binding_size: Some(
                                    <GradientStorage as encase::ShaderType>::min_size(),
                                ),
                            },
                            count: None,
                        },
                        // pub(FRAGMENT) @binding(2) var<storage> s_radial_stops: array<GradientStopStorage>;
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage {
                                    read_only: true,
                                },
                                has_dynamic_offset: false,
                                min_binding_size: Some(
                                    <Vec<GradientStopStorage> as encase::ShaderType>::min_size(),
                                ),
                            },
                            count: None,
                        },
                        // pub(FRAGMENT) @binding(3) var<storage> s_angular_stops: array<GradientStopStorage>;
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage {
                                    read_only: true,
                                },
                                has_dynamic_offset: false,
                                min_binding_size: Some(
                                    <GradientStopStorage as encase::ShaderType>::min_size(),
                                ),
                            },
                            count: None,
                        },
                    ],
                },
            )
        })
    }

    fn bind_group_from_buffers(device: &wgpu::Device, buffers: &Self::Buffers) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: PaintShaderTypes::bind_group_layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.color_uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.gradients_storage.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffers.radial_stops_storage.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffers.angular_stops_storage.as_entire_binding(),
                },
            ],
        })
    }

    fn new_buffers(&self, device: &wgpu::Device) -> Self::Buffers {
        PaintBuffers {
            color_uniform: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: encase::ShaderType::size(&self.color_uniform).get(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            gradients_storage: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: encase::ShaderType::size(&self.gradients_storage).get(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            radial_stops_storage: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: encase::ShaderType::size(&self.radial_stops_storage).get(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            angular_stops_storage: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: encase::ShaderType::size(&self.angular_stops_storage).get(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }

    fn new_buffers_initialized(&self, device: &wgpu::Device) -> Self::Buffers {
        use wgpu::util::DeviceExt;
        PaintBuffers {
            color_uniform: {
                let mut buffer = encase::UniformBuffer::new(Vec::<u8>::new());
                buffer.write(&self.color_uniform).unwrap();
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

    fn write_buffers(&self, queue: &wgpu::Queue, buffers: &mut Self::Buffers) {
        encase::internal::WriteInto::write_into(
            &self.color_uniform,
            &mut encase::internal::Writer::new(
                &self.color_uniform,
                &mut *queue
                    .write_buffer_with(
                        &buffers.color_uniform,
                        0,
                        encase::ShaderType::size(&self.color_uniform),
                    )
                    .unwrap(),
                0,
            )
            .unwrap(),
        );
        encase::internal::WriteInto::write_into(
            &self.gradients_storage,
            &mut encase::internal::Writer::new(
                &self.gradients_storage,
                &mut *queue
                    .write_buffer_with(
                        &buffers.gradients_storage,
                        0,
                        encase::ShaderType::size(&self.gradients_storage),
                    )
                    .unwrap(),
                0,
            )
            .unwrap(),
        );
        encase::internal::WriteInto::write_into(
            &self.radial_stops_storage,
            &mut encase::internal::Writer::new(
                &self.radial_stops_storage,
                &mut *queue
                    .write_buffer_with(
                        &buffers.radial_stops_storage,
                        0,
                        encase::ShaderType::size(&self.radial_stops_storage),
                    )
                    .unwrap(),
                0,
            )
            .unwrap(),
        );
        encase::internal::WriteInto::write_into(
            &self.angular_stops_storage,
            &mut encase::internal::Writer::new(
                &self.angular_stops_storage,
                &mut *queue
                    .write_buffer_with(
                        &buffers.angular_stops_storage,
                        0,
                        encase::ShaderType::size(&self.angular_stops_storage),
                    )
                    .unwrap(),
                0,
            )
            .unwrap(),
        );
    }
}
