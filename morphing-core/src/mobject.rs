use std::fmt::Debug;
use std::sync::Arc;

use super::storable::SharableSlot;
use super::storable::Slot;
use super::storable::SlotKeyGeneratorTypeMap;
use super::storable::Storable;
use super::storable::StorageKey;
use super::storable::StorageTypeMap;
use super::storable::VecSlot;
use super::timer::Clock;
use super::timer::ClockSpan;

pub type ResourceReuseResult = Result<(), ()>;

pub trait Mobject:
    'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
{
    type ResourceInput<'m>;
    type Resource: Send + Sync;

    fn resource_input<'m>(mobject: &'m Self) -> Self::ResourceInput<'m>;
    fn prepare_new(
        resource_input: &Self::ResourceInput<'_>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Resource;
    fn prepare_incremental(
        resource: &mut Self::Resource,
        resource_input: &Self::ResourceInput<'_>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> ResourceReuseResult;
    fn render(resource: &Self::Resource, render_pass: &mut wgpu::RenderPass);
}

pub trait Variant<M>: 'static + Send + Sized + Sync
where
    M: Mobject,
{
    type KeyInput<'o>: serde::Serialize;
    type Slot: Slot;
    type Observe: Send + Sync;
    type Key;

    fn key_input<'o>(observe: &'o Self::Observe) -> Self::KeyInput<'o>;
    fn allocate(
        worldline: &Worldline<M, Self>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key;
    fn prepare(
        worldline: &Worldline<M, Self>,
        key: &Self::Key,
        storage_type_map: &mut StorageTypeMap,
        clock: Clock,
        clock_span: ClockSpan,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> ResourceReuseResult;
}

pub trait Resource<RI> {
    fn prepare_new(
        resource_input: &RI,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self;
    fn prepare_incremental(
        &mut self,
        resource_input: &RI,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> ResourceReuseResult;
    fn render(&self, render_pass: &mut wgpu::RenderPass);
}

pub trait Refresh<M>: 'static + Send + Sync
where
    M: Mobject,
{
    fn refresh(&self, clock: Clock, clock_span: ClockSpan, mobject: M) -> M;
}

pub struct Worldline<M, V>
where
    M: Mobject,
    V: Variant<M>,
{
    observe: V::Observe,
    variant: V,
}

impl<M, V> Storable for Worldline<M, V>
where
    M: Mobject,
    V: Variant<M>,
{
    type KeyInput<'o> = V::KeyInput<'o>;
    type Slot = V::Slot;

    fn key_input<'s>(&'s self) -> Self::KeyInput<'s> {
        V::key_input(&self.observe)
    }
}

pub struct StaticVariant;

impl<M> Variant<M> for StaticVariant
where
    M: Mobject,
{
    type KeyInput<'o> = &'o M;
    type Slot = SharableSlot<M::Resource>;
    type Observe = Arc<M>;
    type Key = StorageKey<Worldline<M, StaticVariant>>;

    fn key_input<'o>(observe: &'o Self::Observe) -> Self::KeyInput<'o> {
        observe
    }

    fn allocate(
        worldline: &Worldline<M, Self>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key {
        slot_key_generator_map.allocate(worldline)
    }

    fn prepare(
        worldline: &Worldline<M, Self>,
        key: &Self::Key,
        storage_type_map: &mut StorageTypeMap,
        _clock: Clock,
        _clock_span: ClockSpan,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> ResourceReuseResult {
        storage_type_map
            .update_or_insert(
                key,
                || {
                    Arc::new(M::prepare_new(
                        &M::resource_input(worldline.observe.as_ref()),
                        device,
                        queue,
                        format,
                    ))
                },
                |_| Ok(()),
            )
            .unwrap_or(Err(()))
    }
}

pub struct DynamicVariant<R>(R);

impl<M, R> Variant<M> for DynamicVariant<R>
where
    M: Mobject,
    R: Refresh<M>,
{
    type KeyInput<'o> = &'o M;
    type Slot = VecSlot<M::Resource>;
    type Observe = M;
    type Key = StorageKey<Worldline<M, DynamicVariant<R>>>;

    fn key_input<'o>(observe: &'o Self::Observe) -> Self::KeyInput<'o> {
        observe
    }

    fn allocate(
        worldline: &Worldline<M, Self>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key {
        slot_key_generator_map.allocate(worldline)
    }

    fn prepare(
        worldline: &Worldline<M, Self>,
        key: &Self::Key,
        storage_type_map: &mut StorageTypeMap,
        clock: Clock,
        clock_span: ClockSpan,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> ResourceReuseResult {
        let mobject = worldline
            .variant
            .0
            .refresh(clock, clock_span, worldline.observe.clone());
        let resource_input = M::resource_input(&mobject);
        storage_type_map
            .update_or_insert(
                key,
                || M::prepare_new(&resource_input, device, queue, format),
                |resource| M::prepare_incremental(resource, &resource_input, device, queue, format),
            )
            .unwrap_or(Err(()))
    }
}
