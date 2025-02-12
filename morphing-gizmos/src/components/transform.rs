use std::sync::OnceLock;

use encase::ShaderType;
use geometric_algebra::ppga3d as pga;
use geometric_algebra::One;
use iced::widget::shader::wgpu::util::DeviceExt;

use super::component::Component;
use super::component::ComponentShaderTypes;
use super::motor::Motor;
use super::paint::QueueWriteBufferMutWrapper;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Transform {
    motor: Motor,
    scale: f32,
}

pub struct TransformShaderTypes {
    transform_uniform: TransformUniform,
}

pub struct TransformBuffers {
    transform_uniform: iced::widget::shader::wgpu::Buffer,
}

#[derive(ShaderType)]
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

static TRANSFORM_BIND_GROUP_LAYOUT: OnceLock<iced::widget::shader::wgpu::BindGroupLayout> =
    OnceLock::new();

impl ComponentShaderTypes for TransformShaderTypes {
    type Buffers = TransformBuffers;

    fn bind_group_layout(
        device: &iced::widget::shader::wgpu::Device,
    ) -> &'static iced::widget::shader::wgpu::BindGroupLayout {
        TRANSFORM_BIND_GROUP_LAYOUT.get_or_init(|| {
            device.create_bind_group_layout(
                &iced::widget::shader::wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        // pub(VERTEX) @binding(0) var<uniform> u_transform: TransformUniform;
                        iced::widget::shader::wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: iced::widget::shader::wgpu::ShaderStages::VERTEX,
                            ty: iced::widget::shader::wgpu::BindingType::Buffer {
                                ty: iced::widget::shader::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: Some(TransformUniform::min_size()),
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
            layout: TransformShaderTypes::bind_group_layout(device),
            entries: &[iced::widget::shader::wgpu::BindGroupEntry {
                binding: 0,
                resource: buffers.transform_uniform.as_entire_binding(),
            }],
        })
    }

    fn new_buffers(&self, device: &iced::widget::shader::wgpu::Device) -> Self::Buffers {
        TransformBuffers {
            transform_uniform: device.create_buffer(
                &iced::widget::shader::wgpu::BufferDescriptor {
                    label: None,
                    size: self.transform_uniform.size().get(),
                    usage: iced::widget::shader::wgpu::BufferUsages::UNIFORM
                        | iced::widget::shader::wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            ),
        }
    }

    fn initialize_buffers(&self, device: &iced::widget::shader::wgpu::Device) -> Self::Buffers {
        TransformBuffers {
            transform_uniform: {
                let mut buffer = encase::UniformBuffer::new(Vec::<u8>::new());
                buffer.write(&self.transform_uniform).unwrap();
                device.create_buffer_init(&iced::widget::shader::wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: buffer.as_ref(),
                    usage: iced::widget::shader::wgpu::BufferUsages::UNIFORM,
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
                    .write_buffer_with(&buffers.transform_uniform, 0, self.transform_uniform.size())
                    .unwrap(),
            ));
            buffer.write(&self.transform_uniform).unwrap();
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            motor: Motor(pga::Motor::one()),
            scale: 1.0,
        }
    }
}
