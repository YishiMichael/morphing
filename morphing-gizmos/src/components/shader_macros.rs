macro_rules! wgpu_struct_field_type {
    (vec2<$type:ty>) => {
        nalgebra::Vector2<$type>
    };
    (vec3<$type:ty>) => {
        nalgebra::Vector3<$type>
    };
    (vec4<$type:ty>) => {
        nalgebra::Vector4<$type>
    };
    (mat2x2<$type:ty>) => {
        nalgebra::Matrix2<$type>
    };
    (mat2x3<$type:ty>) => {
        nalgebra::Matrix3x2<$type>
    };
    (mat2x4<$type:ty>) => {
        nalgebra::Matrix4x2<$type>
    };
    (mat3x2<$type:ty>) => {
        nalgebra::Matrix2x3<$type>
    };
    (mat3x3<$type:ty>) => {
        nalgebra::Matrix3<$type>
    };
    (mat3x4<$type:ty>) => {
        nalgebra::Matrix4x3<$type>
    };
    (mat4x2<$type:ty>) => {
        nalgebra::Matrix2x4<$type>
    };
    (mat4x3<$type:ty>) => {
        nalgebra::Matrix3x4<$type>
    };
    (mat4x4<$type:ty>) => {
        nalgebra::Matrix4<$type>
    };
    ($type:ty) => {
        $type
    };
} // TODO: transpose?

#[macro_export]
macro_rules! wgpu_struct {
    ($(
        $vis:vis struct $struct_name:ident {
            $($name:ident: $type:ident$(<$type_param:ty>)?,)*
        }
    )*) => {$(
        #[derive(encase::ShaderType)]
        $vis struct $struct_name {
            $($name: wgpu_struct_field_type!($type$(<$type_param>)?),)*
        }
    )*};
}

macro_rules! wgpu_shader_types_field_type {
    (array<$type:ty>) => {
        Vec<$type>
    };
    ($type:ty) => {
        $type
    };
}

macro_rules! wgpu_shader_types_buffer_binding_type {
    (var<uniform>) => {
        wgpu::BufferBindingType::Uniform
    };
    (var<storage>) => {
        wgpu::BufferBindingType::Storage { read_only: true }
    };
}

macro_rules! wgpu_shader_types_buffer_usages {
    (var<uniform>) => {
        wgpu::BufferUsages::UNIFORM
    };
    (var<storage>) => {
        wgpu::BufferUsages::STORAGE
    };
}

#[macro_export]
macro_rules! wgpu_shader_types {
    ($(
        $vis:vis struct $shader_types_name:ident {
            // pub(FRAGMENT) @binding(0) var<uniform> u_paint: ColorUniform,
            $(pub($($shader_vis:ident)|+) @binding($binding_index:literal) var<$($kind:ident),*> $name:ident: $type:ident$(<$type_param:ty>)?,)*
        }
    )*) => {paste::paste! {$(
        $vis struct $shader_types_name {
            $($name: wgpu_shader_types_field_type!($type$(<$type_param>)?),)*
        }

        $vis struct [<$shader_types_name Buffers>] {
            $($name: wgpu::Buffer,)*
        }

        static [<$shader_types_name:snake:upper _BIND_GROUP_LAYOUT>]: ::std::sync::OnceLock<wgpu::BindGroupLayout> = ::std::sync::OnceLock::new();

        impl $shader_types_name {
            $vis fn bind_group_layout(device: &wgpu::Device) -> &'static wgpu::BindGroupLayout {
                [<$shader_types_name:snake:upper _BIND_GROUP_LAYOUT>].get_or_init(|| {
                    device.create_bind_group_layout(
                        &wgpu::BindGroupLayoutDescriptor {
                            label: None,
                            entries: &[
                                $(wgpu::BindGroupLayoutEntry {
                                    binding: $binding_index,
                                    visibility: $(wgpu::ShaderStages::$shader_vis)|+,
                                    ty: wgpu::BindingType::Buffer {
                                        ty: wgpu_shader_types_buffer_binding_type!(var<$($kind),*>),
                                        has_dynamic_offset: false,
                                        min_binding_size: Some(
                                            <wgpu_shader_types_field_type!($type$(<$type_param>)?) as encase::ShaderType>::min_size(),
                                        ),
                                    },
                                    count: None,
                                },)*
                            ],
                        },
                    )
                })
            }

            $vis fn bind_group_from_buffers(device: &wgpu::Device, buffers: &[<$shader_types_name Buffers>]) -> wgpu::BindGroup {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: PaintShaderTypes::bind_group_layout(device),
                    entries: &[
                        $(wgpu::BindGroupEntry {
                            binding: $binding_index,
                            resource: buffers.$name.as_entire_binding(),
                        },)*
                    ],
                })
            }

            $vis fn new_buffers(&self, device: &wgpu::Device) -> [<$shader_types_name Buffers>] {
                [<$shader_types_name Buffers>] {
                    $($name: device.create_buffer(&wgpu::BufferDescriptor {
                        label: None,
                        size: encase::ShaderType::size(&self.$name).get(),
                        usage: wgpu_shader_types_buffer_usages!(var<$($kind),*>) | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }),)*
                }
            }

            $vis fn new_buffers_initialized(&self, device: &wgpu::Device) -> [<$shader_types_name Buffers>] {
                use wgpu::util::DeviceExt;
                [<$shader_types_name Buffers>] {
                    $($name: {
                        let mut buffer = encase::UniformBuffer::new(Vec::<u8>::new());
                        buffer.write(&self.$name).unwrap();
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents: buffer.as_ref(),
                            usage: wgpu_shader_types_buffer_usages!(var<$($kind),*>),
                        })
                    },)*
                }
            }

            $vis fn write_buffers(&self, queue: &wgpu::Queue, buffers: &mut [<$shader_types_name Buffers>]) {
                $(encase::internal::WriteInto::write_into(
                    &self.$name,
                    &mut encase::internal::Writer::new(
                        &self.$name,
                        &mut *queue
                            .write_buffer_with(
                                &buffers.$name,
                                0,
                                encase::ShaderType::size(&self.$name),
                            )
                            .unwrap(),
                        0,
                    )
                    .unwrap(),
                );)*
            }
        }
    )*}};
}
