use morphing_core::traits::Component;
use morphing_core::traits::ComponentShaderTypes;

use super::motor::Motor2D;
use super::motor::Motor3D;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CameraTransform2D {
    pub view_motor: Motor2D,
    pub projection_matrix: nalgebra::Matrix3<f32>,
}

pub struct CameraTransform2DShaderTypes {
    camera_transform_2d_uniform: CameraTransform2DUniform,
}

pub struct CameraTransform2DBuffers {
    camera_transform_2d_uniform: wgpu::Buffer,
}

#[derive(encase::ShaderType)]
struct CameraTransform2DUniform {
    view_motor: nalgebra::Vector3<f32>,
    projection_matrix: nalgebra::Matrix3<f32>,
}

impl Component for CameraTransform2D {
    type ShaderTypes = CameraTransform2DShaderTypes;

    fn to_shader_types(&self) -> Self::ShaderTypes {
        CameraTransform2DShaderTypes {
            camera_transform_2d_uniform: CameraTransform2DUniform {
                view_motor: self.view_motor.clone().into(),
                projection_matrix: self.projection_matrix,
            },
        }
    }
}

static CAMERA_TRANSFORM_2D_BIND_GROUP_LAYOUT: ::std::sync::OnceLock<wgpu::BindGroupLayout> =
    ::std::sync::OnceLock::new();

impl ComponentShaderTypes for CameraTransform2DShaderTypes {
    type Buffers = CameraTransform2DBuffers;

    fn bind_group_layout(device: &wgpu::Device) -> &'static wgpu::BindGroupLayout {
        CAMERA_TRANSFORM_2D_BIND_GROUP_LAYOUT.get_or_init(|| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // pub(VERTEX) @binding(0) var<uniform> u_camera_transform_2d: CameraTransform2DUniform;
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(
                                <CameraTransform2DUniform as encase::ShaderType>::min_size(),
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
            layout: CameraTransform2DShaderTypes::bind_group_layout(device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffers.camera_transform_2d_uniform.as_entire_binding(),
            }],
        })
    }

    fn new_buffers(&self, device: &wgpu::Device) -> Self::Buffers {
        CameraTransform2DBuffers {
            camera_transform_2d_uniform: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: encase::ShaderType::size(&self.camera_transform_2d_uniform).get(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }

    fn new_buffers_initialized(&self, device: &wgpu::Device) -> Self::Buffers {
        use wgpu::util::DeviceExt;
        CameraTransform2DBuffers {
            camera_transform_2d_uniform: {
                let mut buffer = encase::UniformBuffer::new(Vec::<u8>::new());
                buffer.write(&self.camera_transform_2d_uniform).unwrap();
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
            &self.camera_transform_2d_uniform,
            &mut encase::internal::Writer::new(
                &self.camera_transform_2d_uniform,
                &mut *queue
                    .write_buffer_with(
                        &buffers.camera_transform_2d_uniform,
                        0,
                        encase::ShaderType::size(&self.camera_transform_2d_uniform),
                    )
                    .unwrap(),
                0,
            )
            .unwrap(),
        );
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CameraTransform3D {
    pub view_motor: Motor3D,
    pub projection_matrix: nalgebra::Matrix4<f32>,
}

pub struct CameraTransform3DShaderTypes {
    camera_transform_3d_uniform: CameraTransform3DUniform,
}

pub struct CameraTransform3DBuffers {
    camera_transform_3d_uniform: wgpu::Buffer,
}

#[derive(encase::ShaderType)]
struct CameraTransform3DUniform {
    view_motor: nalgebra::Matrix4x2<f32>,
    projection_matrix: nalgebra::Matrix4<f32>,
}

impl Component for CameraTransform3D {
    type ShaderTypes = CameraTransform3DShaderTypes;

    fn to_shader_types(&self) -> Self::ShaderTypes {
        CameraTransform3DShaderTypes {
            camera_transform_3d_uniform: CameraTransform3DUniform {
                view_motor: self.view_motor.clone().into(),
                projection_matrix: self.projection_matrix,
            },
        }
    }
}

static CAMERA_TRANSFORM_3D_BIND_GROUP_LAYOUT: ::std::sync::OnceLock<wgpu::BindGroupLayout> =
    ::std::sync::OnceLock::new();

impl ComponentShaderTypes for CameraTransform3DShaderTypes {
    type Buffers = CameraTransform3DBuffers;

    fn bind_group_layout(device: &wgpu::Device) -> &'static wgpu::BindGroupLayout {
        CAMERA_TRANSFORM_3D_BIND_GROUP_LAYOUT.get_or_init(|| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // pub(VERTEX) @binding(0) var<uniform> u_camera_transform_3d: CameraTransform3DUniform;
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(
                                <CameraTransform3DUniform as encase::ShaderType>::min_size(),
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
            layout: CameraTransform3DShaderTypes::bind_group_layout(device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffers.camera_transform_3d_uniform.as_entire_binding(),
            }],
        })
    }

    fn new_buffers(&self, device: &wgpu::Device) -> Self::Buffers {
        CameraTransform3DBuffers {
            camera_transform_3d_uniform: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: encase::ShaderType::size(&self.camera_transform_3d_uniform).get(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }

    fn new_buffers_initialized(&self, device: &wgpu::Device) -> Self::Buffers {
        use wgpu::util::DeviceExt;
        CameraTransform3DBuffers {
            camera_transform_3d_uniform: {
                let mut buffer = encase::UniformBuffer::new(Vec::<u8>::new());
                buffer.write(&self.camera_transform_3d_uniform).unwrap();
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
            &self.camera_transform_3d_uniform,
            &mut encase::internal::Writer::new(
                &self.camera_transform_3d_uniform,
                &mut *queue
                    .write_buffer_with(
                        &buffers.camera_transform_3d_uniform,
                        0,
                        encase::ShaderType::size(&self.camera_transform_3d_uniform),
                    )
                    .unwrap(),
                0,
            )
            .unwrap(),
        );
    }
}
