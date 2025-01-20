pub trait Component {
    type ShaderTypes: ComponentShaderTypes;

    fn to_shader_types(&self) -> Self::ShaderTypes;
}

pub trait ComponentShaderTypes {
    type Buffers;

    fn bind_group_layout(device: &wgpu::Device) -> &'static wgpu::BindGroupLayout;
    fn bind_group_from_buffers(device: &wgpu::Device, buffers: &Self::Buffers) -> wgpu::BindGroup;
    fn new_buffers(&self, device: &wgpu::Device) -> Self::Buffers;
    fn initialize_buffers(&self, device: &wgpu::Device) -> Self::Buffers;
    fn write_buffers(&self, queue: &wgpu::Queue, buffers: &mut Self::Buffers);
}
