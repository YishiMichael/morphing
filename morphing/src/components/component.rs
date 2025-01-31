pub trait Component: serde::de::DeserializeOwned + serde::Serialize {
    type ShaderTypes: ComponentShaderTypes;

    fn to_shader_types(&self) -> Self::ShaderTypes;
}

pub trait ComponentShaderTypes {
    type Buffers;

    fn bind_group_layout(
        device: &iced::widget::shader::wgpu::Device,
    ) -> &'static iced::widget::shader::wgpu::BindGroupLayout;
    fn bind_group_from_buffers(
        device: &iced::widget::shader::wgpu::Device,
        buffers: &Self::Buffers,
    ) -> iced::widget::shader::wgpu::BindGroup;
    fn new_buffers(&self, device: &iced::widget::shader::wgpu::Device) -> Self::Buffers;
    fn initialize_buffers(&self, device: &iced::widget::shader::wgpu::Device) -> Self::Buffers;
    fn write_buffers(&self, queue: &iced::widget::shader::wgpu::Queue, buffers: &mut Self::Buffers);
}
