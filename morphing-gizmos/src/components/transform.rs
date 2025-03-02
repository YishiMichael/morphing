use geometric_algebra::One;

use super::component::Component;
use super::component::ComponentShaderTypes;
use super::motor::Motor;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Transform {
    motor: Motor,
    scale: f32,
}

pub struct TransformShaderTypes {
    transform_uniform: TransformUniform,
}

pub struct TransformBuffers {
    transform_uniform: wgpu::Buffer,
}

#[derive(encase::ShaderType)]
struct TransformUniform {
    motor: nalgebra::Matrix4x2<f32>,
    scale: f32,
}

impl Component for Transform {
    type ShaderTypes = TransformShaderTypes;

    fn to_shader_types(&self) -> Self::ShaderTypes {
        TransformShaderTypes {
            transform_uniform: TransformUniform {
                motor: self.motor.clone().into(),
                scale: self.scale,
            },
        }
    }
}

static TRANSFORM_BIND_GROUP_LAYOUT: ::std::sync::OnceLock<wgpu::BindGroupLayout> =
    ::std::sync::OnceLock::new();

impl ComponentShaderTypes for TransformShaderTypes {
    type Buffers = TransformBuffers;

    fn bind_group_layout(device: &wgpu::Device) -> &'static wgpu::BindGroupLayout {
        TRANSFORM_BIND_GROUP_LAYOUT.get_or_init(|| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // pub(VERTEX) @binding(0) var<uniform> u_transform: TransformUniform;
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(
                                <TransformUniform as encase::ShaderType>::min_size(),
                            ),
                        },
                        count: None,
                    },
                ],
            })
        })
    }

    fn bind_group_from_buffers(device: &wgpu::Device, buffers: &Self::Buffers) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: TransformShaderTypes::bind_group_layout(device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffers.transform_uniform.as_entire_binding(),
            }],
        })
    }

    fn new_buffers(&self, device: &wgpu::Device) -> Self::Buffers {
        TransformBuffers {
            transform_uniform: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: encase::ShaderType::size(&self.transform_uniform).get(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }

    fn initialize_buffers(&self, device: &wgpu::Device) -> Self::Buffers {
        use wgpu::util::DeviceExt;
        TransformBuffers {
            transform_uniform: {
                let mut buffer = encase::UniformBuffer::new(Vec::<u8>::new());
                buffer.write(&self.transform_uniform).unwrap();
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: buffer.as_ref(),
                    usage: wgpu::BufferUsages::UNIFORM,
                })
            },
        }
    }

    fn write_buffers(&self, queue: &wgpu::Queue, buffers: &mut Self::Buffers) {
        encase::internal::WriteInto::write_into(
            &self.transform_uniform,
            &mut encase::internal::Writer::new(
                &self.transform_uniform,
                &mut *queue
                    .write_buffer_with(
                        &buffers.transform_uniform,
                        0,
                        encase::ShaderType::size(&self.transform_uniform),
                    )
                    .unwrap(),
                0,
            )
            .unwrap(),
        );
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            motor: Motor(geometric_algebra::ppga3d::Motor::one()),
            scale: 1.0,
        }
    }
}
