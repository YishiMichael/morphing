use wgpu::util::DeviceExt;

use super::storable::ResourceReuseResult;

// TODO: move to builtins

// pub trait Resource<I> {
//     fn prepare_new(
//         input: &I,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> Self;
//     fn prepare_incremental(
//         &mut self,
//         input: &I,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> ResourceReuseResult;
//     // fn render(&self, render_pass: &mut wgpu::RenderPass);
// }

pub trait ResourceType<I> {
    type Resource;

    fn prepare_new(
        &self,
        input: &I,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> Self::Resource;
    fn prepare_incremental(
        &self,
        resource: &mut Self::Resource,
        input: &I,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> ResourceReuseResult;
}

pub struct BufferResource {
    buffer: wgpu::Buffer,
    size: wgpu::BufferAddress,
}

impl BufferResource {
    pub fn as_entire_binding(&self) -> wgpu::BindingResource<'_> {
        self.buffer.as_entire_binding()
    }

    pub fn buffer_slice(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(0..self.size)
    }
}

pub struct SizedBufferType(wgpu::BufferUsages);

impl<I> ResourceType<I> for SizedBufferType
where
    I: encase::ShaderType + encase::internal::WriteInto,
{
    type Resource = BufferResource;

    fn prepare_new(
        &self,
        input: &I,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> Self::Resource {
        let size = input.size().get();
        let mut buffer = encase::UniformBuffer::new(vec![0u8; size as usize]);
        buffer.write(input).unwrap();
        BufferResource {
            buffer: device.create_buffer_init(&::wgpu::util::BufferInitDescriptor {
                label: None,
                contents: buffer.as_ref(),
                usage: self.0,
            }),
            size,
        }
    }

    fn prepare_incremental(
        &self,
        resource: &mut Self::Resource,
        input: &I,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> ResourceReuseResult {
        input.write_into(
            &mut encase::internal::Writer::new(
                input,
                &mut *queue
                    .write_buffer_with(&resource.buffer, 0, input.size())
                    .unwrap(),
                0,
            )
            .unwrap(),
        );
        Ok(())
    }
}

pub struct UnsizedBufferType(wgpu::BufferUsages);

impl<I> ResourceType<I> for UnsizedBufferType
where
    I: encase::ShaderType + encase::internal::WriteInto,
{
    type Resource = BufferResource;

    fn prepare_new(
        &self,
        input: &I,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> Self::Resource {
        let size = input.size().get();
        let mut buffer = encase::UniformBuffer::new(vec![0u8; size.next_power_of_two() as usize]);
        buffer.write(input).unwrap();
        BufferResource {
            buffer: device.create_buffer_init(&::wgpu::util::BufferInitDescriptor {
                label: None,
                contents: buffer.as_ref(),
                usage: self.0,
            }),
            size,
        }
    }

    fn prepare_incremental(
        &self,
        resource: &mut Self::Resource,
        input: &I,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> ResourceReuseResult {
        queue
            .write_buffer_with(&resource.buffer, 0, input.size())
            .map(|mut queue_write_buffer_view| {
                input.write_into(
                    &mut encase::internal::Writer::new(input, &mut *queue_write_buffer_view, 0)
                        .unwrap(),
                )
            })
            .ok_or(())
    }
}

pub const UNIFORM_BUFFER_TYPE: SizedBufferType = SizedBufferType(wgpu::BufferUsages::UNIFORM);
pub const STORAGE_BUFFER_TYPE: SizedBufferType = SizedBufferType(wgpu::BufferUsages::STORAGE);
pub const VERTEX_BUFFER_TYPE: UnsizedBufferType = UnsizedBufferType(wgpu::BufferUsages::VERTEX);
pub const INDEX_BUFFER_TYPE: UnsizedBufferType = UnsizedBufferType(wgpu::BufferUsages::INDEX);
