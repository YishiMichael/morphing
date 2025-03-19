pub struct PlanarTrimeshPresentation {
    pub transform_bind_group: wgpu::BindGroup,
    pub paint_bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

pub struct PlanarTrimeshVecPresentation(pub Vec<PlanarTrimeshPresentation>);
// TODO: align with components
