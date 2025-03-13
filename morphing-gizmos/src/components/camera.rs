use geometric_algebra::GeometricProduct;
use geometric_algebra::One;

use super::component::Component;
use super::component::ComponentShaderTypes;
use super::motor::Motor3D;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Camera {
    view_motor: Motor3D,
    projection_matrix: nalgebra::Matrix4<f32>,
}

pub struct CameraShaderTypes {
    camera_uniform: CameraUniform,
}

pub struct CameraBuffers {
    camera_uniform: wgpu::Buffer,
}

#[derive(encase::ShaderType)]
struct CameraUniform {
    view_motor: nalgebra::Matrix4x2<f32>,
    projection_matrix: nalgebra::Matrix4<f32>,
}

impl Component for Camera {
    type ShaderTypes = CameraShaderTypes;

    fn to_shader_types(&self) -> Self::ShaderTypes {
        CameraShaderTypes {
            camera_uniform: CameraUniform {
                view_motor: self.view_motor.clone().into(),
                projection_matrix: self.projection_matrix,
            },
        }
    }
}

static CAMERA_BIND_GROUP_LAYOUT: ::std::sync::OnceLock<wgpu::BindGroupLayout> =
    ::std::sync::OnceLock::new();

impl ComponentShaderTypes for CameraShaderTypes {
    type Buffers = CameraBuffers;

    fn bind_group_layout(device: &wgpu::Device) -> &'static wgpu::BindGroupLayout {
        CAMERA_BIND_GROUP_LAYOUT.get_or_init(|| {
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
                            min_binding_size: Some(
                                <CameraUniform as encase::ShaderType>::min_size(),
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
            layout: CameraShaderTypes::bind_group_layout(device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffers.camera_uniform.as_entire_binding(),
            }],
        })
    }

    fn new_buffers(&self, device: &wgpu::Device) -> Self::Buffers {
        CameraBuffers {
            camera_uniform: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: encase::ShaderType::size(&self.camera_uniform).get(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }

    fn new_buffers_initialized(&self, device: &wgpu::Device) -> Self::Buffers {
        use wgpu::util::DeviceExt;
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

    fn write_buffers(&self, queue: &wgpu::Queue, buffers: &mut Self::Buffers) {
        encase::internal::WriteInto::write_into(
            &self.camera_uniform,
            &mut encase::internal::Writer::new(
                &self.camera_uniform,
                &mut *queue
                    .write_buffer_with(
                        &buffers.camera_uniform,
                        0,
                        encase::ShaderType::size(&self.camera_uniform),
                    )
                    .unwrap(),
                0,
            )
            .unwrap(),
        );
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            view_motor: Motor3D(geometric_algebra::ppga3d::Motor::one().geometric_product(
                geometric_algebra::ppga3d::Translator::new(1.0, 0.0, 0.0, 5.0),
            )),
            projection_matrix: nalgebra::Matrix4::new_perspective(
                16.0 / 9.0,
                40.0_f32.to_radians(),
                0.1,
                100.0,
            ),
        }
    }
}
