use encase::ShaderType;
use geometric_algebra::ppga3d as pga;
use geometric_algebra::GeometricProduct;
use geometric_algebra::One;
use wgpu::util::DeviceExt;

use super::paint::QueueWriteBufferMutWrapper;

#[derive(Clone)]
pub struct Camera {
    view_motor: pga::Motor,
    projection_matrix: nalgebra::Matrix4<f32>,
}

pub(crate) struct CameraShaderTypes {
    camera_uniform: CameraUniform,
}

pub(crate) struct CameraBuffers {
    pub(crate) camera_uniform: wgpu::Buffer,
}

#[derive(ShaderType)]
pub(crate) struct CameraUniform {
    view_motor: nalgebra::Matrix2x4<f32>,
    projection_matrix: nalgebra::Matrix4<f32>,
}

impl Camera {
    pub(crate) fn to_shader_types(&self) -> CameraShaderTypes {
        CameraShaderTypes {
            camera_uniform: CameraUniform {
                view_motor: nalgebra::Matrix2x4::from_column_slice(&Into::<[f32; 8]>::into(
                    self.view_motor,
                )),
                projection_matrix: self.projection_matrix,
            },
        }
    }
}

impl CameraShaderTypes {
    pub(crate) fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // pub(VERTEX) @binding(0) var<uniform> u_camera: CameraUniform;
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(CameraUniform::min_size()),
                    },
                    count: None,
                },
            ],
        })
    }

    pub(crate) fn create_buffers_init(&self, device: &wgpu::Device) -> CameraBuffers {
        CameraBuffers {
            camera_uniform: {
                let mut buffer = encase::UniformBuffer::new(Vec::<u8>::new());
                buffer.write(&self.camera_uniform).unwrap();
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: buffer.as_ref(),
                    usage: wgpu::BufferUsages::UNIFORM,
                })
            },
        }
    }

    pub(crate) fn create_buffers(&self, device: &wgpu::Device) -> CameraBuffers {
        CameraBuffers {
            camera_uniform: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: self.camera_uniform.size().get(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }

    pub(crate) fn write_buffers(&self, queue: &wgpu::Queue, buffers: &mut CameraBuffers) {
        {
            let mut buffer = encase::UniformBuffer::new(QueueWriteBufferMutWrapper(
                queue
                    .write_buffer_with(&buffers.camera_uniform, 0, self.camera_uniform.size())
                    .unwrap(),
            ));
            buffer.write(&self.camera_uniform).unwrap();
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            view_motor: pga::Motor::one()
                .geometric_product(pga::Translator::new(1.0, 0.0, 0.0, 5.0)),
            projection_matrix: nalgebra::Matrix4::new_perspective(
                16.0 / 9.0,
                40.0_f32.to_radians(),
                0.1,
                100.0,
            ),
        }
    }
}
