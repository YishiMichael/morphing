use encase::ShaderType;
use geometric_algebra::ppga3d as pga;
use geometric_algebra::One;
use wgpu::util::DeviceExt;

use super::paint::QueueWriteBufferMutWrapper;

#[derive(Clone)]
pub struct Transform {
    motor: pga::Motor,
    scale: f32,
}

pub(crate) struct TransformShaderTypes {
    transform_uniform: TransformUniform,
}

pub(crate) struct TransformBuffers {
    pub(crate) transform_uniform: wgpu::Buffer, // TODO: remove pub(crate) vis
}

#[derive(ShaderType)]
pub(crate) struct TransformUniform {
    motor: nalgebra::Matrix4x2<f32>,
    scale: f32,
}

impl Transform {
    pub(crate) fn to_shader_types(&self) -> TransformShaderTypes {
        TransformShaderTypes {
            transform_uniform: TransformUniform {
                motor: nalgebra::Matrix4x2::from_column_slice(&Into::<[f32; 8]>::into(self.motor)), // transpose?
                scale: self.scale,
            },
        }
    }
}

impl TransformShaderTypes {
    pub(crate) fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
                        min_binding_size: Some(TransformUniform::min_size()),
                    },
                    count: None,
                },
            ],
        })
    }

    pub(crate) fn create_buffers_init(&self, device: &wgpu::Device) -> TransformBuffers {
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

    pub(crate) fn create_buffers(&self, device: &wgpu::Device) -> TransformBuffers {
        TransformBuffers {
            transform_uniform: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: self.transform_uniform.size().get(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }

    pub(crate) fn write_buffers(&self, queue: &wgpu::Queue, buffers: &mut TransformBuffers) {
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
            motor: pga::Motor::one(),
            scale: 1.0,
        }
    }
}
