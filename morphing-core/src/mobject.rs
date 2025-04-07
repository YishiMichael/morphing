use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use super::storable::SharableSlot;
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
    // type KeyInput<'s>: serde::Serialize;
    // type Slot;
    type Observe: Send + Sync;
    type Key;

    // fn key_input<'o>(observe: &'o Self::Observe) -> Self::KeyInput<'s>;
    fn allocate(
        variant: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key;
    fn prepare(
        variant: &Self,
        observe: &Self::Observe,
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

// pub struct Worldline<M, V>
// where
//     M: Mobject,
//     V: Variant<M>,
// {
//     observe: V::Observe,
//     variant: V,
// }

pub struct StaticVariant;

impl<M> Storable for (M, StaticVariant)
where
    M: Mobject,
{
    type KeyInput = M;
    type Slot = SharableSlot<M::Resource>;

    // fn key_input<'s>(&'s self) -> Self::KeyInput<'s> {
    //     V::key_input(&self.observe)
    // }
}

impl<M> Variant<M> for StaticVariant
where
    M: Mobject,
{
    // type KeyInput<'s> = &'o M;
    // type Slot = SharableSlot<M::Resource>;
    type Observe = Arc<M>;
    type Key = StorageKey<(M, StaticVariant)>;

    // fn key_input<'o>(observe: &'o Self::Observe) -> Self::KeyInput<'s> {
    //     observe
    // }

    fn allocate(
        _variant: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key {
        slot_key_generator_map.allocate::<(M, StaticVariant)>(observe.as_ref())
    }

    fn prepare(
        _variant: &Self,
        observe: &Self::Observe,
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
                        &M::resource_input(observe.as_ref()),
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

pub struct DynamicVariant<R> {
    refresh: R,
}

impl<M, R> Storable for (M, DynamicVariant<R>)
where
    M: Mobject,
    R: Refresh<M>,
{
    type KeyInput = M;
    type Slot = VecSlot<M::Resource>;

    // fn key_input<'s>(&'s self) -> Self::KeyInput<'s> {
    //     V::key_input(&self.observe)
    // }
}

impl<M, R> Variant<M> for DynamicVariant<R>
where
    M: Mobject,
    R: Refresh<M>,
{
    type Observe = M;
    type Key = StorageKey<(M, DynamicVariant<R>)>;

    // fn key_input<'o>(observe: &'o Self::Observe) -> Self::KeyInput<'s> {
    //     observe
    // }

    fn allocate(
        _variant: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key {
        slot_key_generator_map.allocate(observe)
    }

    fn prepare(
        variant: &Self,
        observe: &Self::Observe,
        key: &Self::Key,
        storage_type_map: &mut StorageTypeMap,
        clock: Clock,
        clock_span: ClockSpan,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> ResourceReuseResult {
        let mobject = variant.refresh.refresh(clock, clock_span, observe.clone());
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

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Data<T>(T);

impl<T> From<T> for Data<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for Data<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Mobject for Data<T>
where
    T: 'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    type ResourceInput<'m> = ();
    type Resource = ();

    fn resource_input<'m>(_mobject: &'m Self) -> Self::ResourceInput<'m> {
        ()
    }

    fn prepare_new(
        _resource_input: &Self::ResourceInput<'_>,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> Self::Resource {
        ()
    }

    fn prepare_incremental(
        _resource: &mut Self::Resource,
        _resource_input: &Self::ResourceInput<'_>,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> ResourceReuseResult {
        Ok(())
    }

    fn render(_resource: &Self::Resource, _render_pass: &mut wgpu::RenderPass) {}
}

// demo

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject0<MA, MB> {
    ma: MA,
    mb: MB,
}

impl Mobject for MyMobject0<Data<f32>, Data<f32>> {
    type ResourceInput<'m> = MyMobject0<
        <Data<f32> as Mobject>::ResourceInput<'m>,
        <Data<f32> as Mobject>::ResourceInput<'m>,
    >;
    type Resource = MyMobject0<<Data<f32> as Mobject>::Resource, <Data<f32> as Mobject>::Resource>;

    fn resource_input<'m>(mobject: &'m Self) -> Self::ResourceInput<'m> {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::resource_input(&mobject.ma),
            mb: <Data<f32> as Mobject>::resource_input(&mobject.mb),
        }
    }

    fn prepare_new(
        resource_input: &Self::ResourceInput<'_>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Resource {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::prepare_new(&resource_input.ma, device, queue, format),
            mb: <Data<f32> as Mobject>::prepare_new(&resource_input.mb, device, queue, format),
        }
    }

    fn prepare_incremental(
        resource: &mut Self::Resource,
        resource_input: &Self::ResourceInput<'_>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> ResourceReuseResult {
        [
            <Data<f32> as Mobject>::prepare_incremental(
                &mut resource.ma,
                &resource_input.ma,
                device,
                queue,
                format,
            ),
            <Data<f32> as Mobject>::prepare_incremental(
                &mut resource.mb,
                &resource_input.mb,
                device,
                queue,
                format,
            ),
        ]
        .into_iter()
        .collect()
    }

    fn render(resource: &Self::Resource, render_pass: &mut wgpu::RenderPass) {
        <Data<f32> as Mobject>::render(&resource.ma, render_pass);
        <Data<f32> as Mobject>::render(&resource.mb, render_pass);
    }
}

impl<MA, MB> Variant<MyMobject0<Data<f32>, Data<f32>>> for MyMobject0<MA, MB>
where
    MA: Variant<Data<f32>>,
    MB: Variant<Data<f32>>,
{
    // type KeyInput<'s> = MyMobject0<MA::KeyInput<'s>, MB::KeyInput<'s>>;
    // type Slot = MyMobject0<MA::Slot, MB::Slot>;
    type Observe = MyMobject0<MA::Observe, MB::Observe>;
    type Key = MyMobject0<MA::Key, MB::Key>;

    // fn key_input<'o>(observe: &'o Self::Observe) -> Self::KeyInput<'s> {
    //     MyMobject0 {
    //         ma: MA::key_input(&observe),
    //         mb: MB::key_input(&observe),
    //     }
    // }

    fn allocate(
        variant: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key {
        MyMobject0 {
            ma: MA::allocate(&variant.ma, &observe.ma, slot_key_generator_map),
            mb: MB::allocate(&variant.mb, &observe.mb, slot_key_generator_map),
        }
    }

    fn prepare(
        variant: &Self,
        observe: &Self::Observe,
        key: &Self::Key,
        storage_type_map: &mut StorageTypeMap,
        clock: Clock,
        clock_span: ClockSpan,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> ResourceReuseResult {
        [
            MA::prepare(
                &variant.ma,
                &observe.ma,
                &key.ma,
                storage_type_map,
                clock,
                clock_span,
                device,
                queue,
                format,
            ),
            MB::prepare(
                &variant.mb,
                &observe.mb,
                &key.mb,
                storage_type_map,
                clock,
                clock_span,
                device,
                queue,
                format,
            ),
        ]
        .into_iter()
        .collect()
    }
}
