use std::marker::PhantomData;

use wgpu::util::DeviceExt;

use super::mobject::Resource;
use super::storable::ResourceReuseResult;

// TODO: move to builtins

pub trait BufferUsages: 'static {
    const BUFFER_USAGE: wgpu::BufferUsages;
}

pub struct IndexBufferUsages;

impl BufferUsages for IndexBufferUsages {
    const BUFFER_USAGE: wgpu::BufferUsages = wgpu::BufferUsages::INDEX;
}

pub struct VertexBufferUsages;

impl BufferUsages for VertexBufferUsages {
    const BUFFER_USAGE: wgpu::BufferUsages = wgpu::BufferUsages::VERTEX;
}

pub struct UniformBufferUsages;

impl BufferUsages for UniformBufferUsages {
    const BUFFER_USAGE: wgpu::BufferUsages = wgpu::BufferUsages::UNIFORM;
}

pub struct StorageBufferUsages;

impl BufferUsages for StorageBufferUsages {
    const BUFFER_USAGE: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE;
}

pub trait BufferReservePolicy: 'static {
    fn reserve(size: wgpu::BufferAddress) -> wgpu::BufferAddress;
}

pub struct BufferReserveExactPolicy;

impl BufferReservePolicy for BufferReserveExactPolicy {
    fn reserve(size: wgpu::BufferAddress) -> wgpu::BufferAddress {
        size
    }
}

pub struct BufferReserveExtendedPolicy;

impl BufferReservePolicy for BufferReserveExtendedPolicy {
    fn reserve(size: wgpu::BufferAddress) -> wgpu::BufferAddress {
        size.next_power_of_two()
    }
}

pub struct BufferResource<BU, BRP> {
    buffer: wgpu::Buffer,
    size: wgpu::BufferAddress,
    phantom: PhantomData<fn() -> (BU, BRP)>,
}

impl<BU, BRP> BufferResource<BU, BRP> {
    pub fn as_entire_binding(&self) -> wgpu::BindingResource<'_> {
        self.buffer.as_entire_binding()
    }

    pub fn buffer_slice(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(0..self.size)
    }
}

impl<RR, BU, BRP> Resource<RR> for BufferResource<BU, BRP>
where
    RR: encase::ShaderType + encase::internal::WriteInto,
    BRP: BufferReservePolicy,
    BU: BufferUsages,
{
    fn new(
        resource_repr: RR,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> Self {
        let size = resource_repr.size().get();
        let mut buffer = encase::UniformBuffer::new(vec![0u8; BRP::reserve(size) as usize]);
        buffer.write(&resource_repr).unwrap();
        BufferResource {
            buffer: device.create_buffer_init(&::wgpu::util::BufferInitDescriptor {
                label: None,
                contents: buffer.as_ref(),
                usage: BU::BUFFER_USAGE,
            }),
            size,
            phantom: PhantomData,
        }
    }

    fn update(
        resource: &mut Self,
        resource_repr: RR,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> ResourceReuseResult {
        queue
            .write_buffer_with(&resource.buffer, 0, resource_repr.size())
            .map(|mut queue_write_buffer_view| {
                resource_repr.write_into(
                    &mut encase::internal::Writer::new(
                        &resource_repr,
                        &mut *queue_write_buffer_view,
                        0,
                    )
                    .unwrap(),
                )
            })
            .ok_or(())
    }
}

pub type IndexBuffer<BRP = BufferReserveExtendedPolicy> = BufferResource<IndexBufferUsages, BRP>;
pub type VertexBuffer<BRP = BufferReserveExtendedPolicy> = BufferResource<VertexBufferUsages, BRP>;
pub type UniformBuffer<BRP = BufferReserveExactPolicy> = BufferResource<UniformBufferUsages, BRP>;
pub type StorageBuffer<BRP = BufferReserveExactPolicy> = BufferResource<StorageBufferUsages, BRP>;
