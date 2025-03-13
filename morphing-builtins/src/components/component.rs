pub trait Component: serde::de::DeserializeOwned + serde::Serialize {
    type ShaderTypes: ComponentShaderTypes;

    fn to_shader_types(&self) -> Self::ShaderTypes;
}

pub trait ComponentShaderTypes {
    type Buffers;

    fn bind_group_layout(device: &wgpu::Device) -> &'static wgpu::BindGroupLayout;
    fn bind_group_from_buffers(device: &wgpu::Device, buffers: &Self::Buffers) -> wgpu::BindGroup;
    fn new_buffers(&self, device: &wgpu::Device) -> Self::Buffers;
    fn new_buffers_initialized(&self, device: &wgpu::Device) -> Self::Buffers;
    fn write_buffers(&self, queue: &wgpu::Queue, buffers: &mut Self::Buffers);
}
