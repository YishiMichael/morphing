use geometric_algebra::ppga3d as pga;
use geometric_algebra::GeometricProduct;
use geometric_algebra::One;

use super::component::Component;
use super::component::ComponentShaderTypes;
use super::motor::Motor;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Camera {
    view_motor: Motor,
    projection_matrix: nalgebra::Matrix4<f32>,
}

pub struct CameraShaderTypes {
    camera_uniform: CameraUniform,
}

pub struct CameraBuffers {
    camera_uniform: iced::widget::shader::wgpu::Buffer,
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

static CAMERA_BIND_GROUP_LAYOUT: ::std::sync::OnceLock<
    iced::widget::shader::wgpu::BindGroupLayout,
> = ::std::sync::OnceLock::new();

impl ComponentShaderTypes for CameraShaderTypes {
    type Buffers = CameraBuffers;

    fn bind_group_layout(
        device: &iced::widget::shader::wgpu::Device,
    ) -> &'static iced::widget::shader::wgpu::BindGroupLayout {
        CAMERA_BIND_GROUP_LAYOUT.get_or_init(|| {
            device.create_bind_group_layout(
                &iced::widget::shader::wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        // pub(VERTEX) @binding(0) var<uniform> u_camera: CameraUniform;
                        iced::widget::shader::wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: iced::widget::shader::wgpu::ShaderStages::VERTEX,
                            ty: iced::widget::shader::wgpu::BindingType::Buffer {
                                ty: iced::widget::shader::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: Some(
                                    <CameraUniform as encase::ShaderType>::min_size(),
                                ),
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
            layout: CameraShaderTypes::bind_group_layout(device),
            entries: &[iced::widget::shader::wgpu::BindGroupEntry {
                binding: 0,
                resource: buffers.camera_uniform.as_entire_binding(),
            }],
        })
    }

    fn new_buffers(&self, device: &iced::widget::shader::wgpu::Device) -> Self::Buffers {
        CameraBuffers {
            camera_uniform: device.create_buffer(&iced::widget::shader::wgpu::BufferDescriptor {
                label: None,
                size: encase::ShaderType::size(&self.camera_uniform).get(),
                usage: iced::widget::shader::wgpu::BufferUsages::UNIFORM
                    | iced::widget::shader::wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }

    fn initialize_buffers(&self, device: &iced::widget::shader::wgpu::Device) -> Self::Buffers {
        use iced::widget::shader::wgpu::util::DeviceExt;
        CameraBuffers {
            camera_uniform: {
                let mut buffer = encase::UniformBuffer::new(Vec::<u8>::new());
                buffer.write(&self.camera_uniform).unwrap();
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
            view_motor: Motor(
                pga::Motor::one().geometric_product(pga::Translator::new(1.0, 0.0, 0.0, 5.0)),
            ),
            projection_matrix: nalgebra::Matrix4::new_perspective(
                16.0 / 9.0,
                40.0_f32.to_radians(),
                0.1,
                100.0,
            ),
        }
    }
}
