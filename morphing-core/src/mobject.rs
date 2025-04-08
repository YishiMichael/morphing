use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use super::storable::ArcSlot;
use super::storable::ResourceReuseResult;
use super::storable::SlotKeyGeneratorTypeMap;
use super::storable::StorageKey;
use super::storable::StorageTypeMap;
use super::storable::StoreType;
use super::storable::VecSlot;
use super::timer::Clock;
use super::timer::ClockSpan;

pub trait Mobject:
    'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
{
    type StaticKey;
    type DynamicKey;
    type ResourceRefInput<'s>;
    type ResourceRef<'s>;

    fn static_allocate(
        mobject: &Self,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::StaticKey;
    fn dynamic_allocate(
        mobject: &Self,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::DynamicKey;
    fn static_prepare<'s>(
        mobject: &Self,
        key: &Self::StaticKey,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        result: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s>;
    fn dynamic_prepare<'s>(
        mobject: &Self,
        key: &Self::DynamicKey,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        result: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s>;
    fn generic_render(resource: Self::ResourceRef<'_>, render_pass: &mut wgpu::RenderPass);
}

pub trait Timeline<M>: 'static + Send + Sized + Sync
where
    M: Mobject,
{
    type Observe: Send + Sync;
    type Key;

    fn allocate(
        timeline: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key;
    fn prepare(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &Self::Observe,
        key: &Self::Key,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        result: &mut ResourceReuseResult,
    );
}

pub trait Prepare: Mobject {
    type Resource: 'static + Send + Sync;

    fn prepare_new(
        input: Self::ResourceRefInput<'_>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Resource;
    fn prepare_incremental(
        resource: &mut Self::Resource,
        input: Self::ResourceRefInput<'_>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> ResourceReuseResult;
}

pub trait Render: Mobject {
    fn render(resource: Self::ResourceRef<'_>, render_pass: &mut wgpu::RenderPass);
}

pub trait Variant<P>: StoreType
where
    P: Prepare,
{
}

pub struct StaticVariant<P>(P);

impl<P> StoreType for StaticVariant<P>
where
    P: Prepare,
{
    type KeyInput = P;
    type Slot = ArcSlot<P::Resource>;
}

impl<P> Variant<P> for StaticVariant<P> where P: Prepare {}

pub struct DynamicVariant<P>(P);

impl<P> StoreType for DynamicVariant<P>
where
    P: Prepare,
{
    type KeyInput = ();
    type Slot = VecSlot<P::Resource>;
}

impl<P> Variant<P> for DynamicVariant<P> where P: Prepare {}

pub trait Refresh<M>: 'static + Send + Sync
where
    M: Mobject,
{
    fn refresh(&self, clock: Clock, clock_span: ClockSpan, mobject: M) -> M;
}

pub struct StaticTimeline;

impl<M> Timeline<M> for StaticTimeline
where
    M: Mobject,
{
    type Observe = Arc<M>;
    type Key = M::StaticKey;

    fn allocate(
        _timeline: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key {
        M::static_allocate(observe.as_ref(), slot_key_generator_map)
    }

    fn prepare(
        _clock: Clock,
        _clock_span: ClockSpan,
        _timeline: &Self,
        observe: &Self::Observe,
        key: &Self::Key,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        result: &mut ResourceReuseResult,
    ) {
        M::static_prepare(
            observe.as_ref(),
            key,
            storage_type_map,
            device,
            queue,
            format,
            result,
        );
    }
}

pub struct DynamicTimeline<R> {
    refresh: R,
}

impl<M, R> Timeline<M> for DynamicTimeline<R>
where
    M: Mobject,
    R: Refresh<M>,
{
    type Observe = M;
    type Key = M::DynamicKey;

    fn allocate(
        _timeline: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key {
        M::dynamic_allocate(observe, slot_key_generator_map)
    }

    fn prepare(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &Self::Observe,
        key: &Self::Key,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        result: &mut ResourceReuseResult,
    ) {
        M::dynamic_prepare(
            &timeline.refresh.refresh(clock, clock_span, observe.clone()),
            key,
            storage_type_map,
            device,
            queue,
            format,
            result,
        );
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
    type StaticKey = ();
    type DynamicKey = ();
    type ResourceRefInput<'s> = ();
    type ResourceRef<'s> = T;

    fn static_allocate(
        _mobject: &Self,
        _slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::StaticKey {
        ()
    }

    fn dynamic_allocate(
        _mobject: &Self,
        _slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::DynamicKey {
        ()
    }

    fn static_prepare<'s>(
        mobject: &Self,
        _key: &Self::StaticKey,
        _storage_type_map: &'s mut StorageTypeMap,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        _result: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        (**mobject).clone()
    }

    fn dynamic_prepare<'s>(
        mobject: &Self,
        _key: &Self::DynamicKey,
        _storage_type_map: &'s mut StorageTypeMap,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        _result: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        (**mobject).clone()
    }

    fn generic_render(_resource_ref: Self::ResourceRef<'_>, _render_pass: &mut wgpu::RenderPass) {}
}

// demo

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject0<MA = Data<f32>, MB = Data<f32>> {
    ma: MA,
    mb: MB,
}

impl Mobject for MyMobject0 {
    type StaticKey =
        MyMobject0<<Data<f32> as Mobject>::StaticKey, <Data<f32> as Mobject>::StaticKey>;
    type DynamicKey =
        MyMobject0<<Data<f32> as Mobject>::DynamicKey, <Data<f32> as Mobject>::DynamicKey>;
    type ResourceRefInput<'s> = MyMobject0<
        <Data<f32> as Mobject>::ResourceRef<'s>,
        <Data<f32> as Mobject>::ResourceRef<'s>,
    >;
    type ResourceRef<'s> = MyMobject0<
        <Data<f32> as Mobject>::ResourceRef<'s>,
        <Data<f32> as Mobject>::ResourceRef<'s>,
    >;

    fn static_allocate(
        mobject: &Self,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::StaticKey {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::static_allocate(&mobject.ma, slot_key_generator_map),
            mb: <Data<f32> as Mobject>::static_allocate(&mobject.mb, slot_key_generator_map),
        }
    }

    fn dynamic_allocate(
        mobject: &Self,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::DynamicKey {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::dynamic_allocate(&mobject.ma, slot_key_generator_map),
            mb: <Data<f32> as Mobject>::dynamic_allocate(&mobject.mb, slot_key_generator_map),
        }
    }

    fn static_prepare<'s>(
        mobject: &Self,
        key: &Self::StaticKey,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        result: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::static_prepare(
                &mobject.ma,
                &key.ma,
                storage_type_map,
                device,
                queue,
                format,
                result,
            ),
            mb: <Data<f32> as Mobject>::static_prepare(
                &mobject.mb,
                &key.mb,
                storage_type_map,
                device,
                queue,
                format,
                result,
            ),
        }
    }

    fn dynamic_prepare<'s>(
        mobject: &Self,
        key: &Self::DynamicKey,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        result: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::dynamic_prepare(
                &mobject.ma,
                &key.ma,
                storage_type_map,
                device,
                queue,
                format,
                result,
            ),
            mb: <Data<f32> as Mobject>::dynamic_prepare(
                &mobject.mb,
                &key.mb,
                storage_type_map,
                device,
                queue,
                format,
                result,
            ),
        }
    }

    fn generic_render(resource: Self::ResourceRef<'_>, render_pass: &mut wgpu::RenderPass) {
        <Data<f32> as Mobject>::generic_render(resource.ma, render_pass);
        <Data<f32> as Mobject>::generic_render(resource.mb, render_pass);
    }
}

impl<MA, MB> Timeline<MyMobject0<Data<f32>, Data<f32>>> for MyMobject0<MA, MB>
where
    MA: Timeline<Data<f32>>,
    MB: Timeline<Data<f32>>,
{
    type Observe = MyMobject0<MA::Observe, MB::Observe>;
    type Key = MyMobject0<MA::Key, MB::Key>;

    fn allocate(
        timeline: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key {
        MyMobject0 {
            ma: MA::allocate(&timeline.ma, &observe.ma, slot_key_generator_map),
            mb: MB::allocate(&timeline.mb, &observe.mb, slot_key_generator_map),
        }
    }

    fn prepare(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &Self::Observe,
        key: &Self::Key,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        result: &mut ResourceReuseResult,
    ) {
        MA::prepare(
            clock,
            clock_span,
            &timeline.ma,
            &observe.ma,
            &key.ma,
            storage_type_map,
            device,
            queue,
            format,
            result,
        );
        MB::prepare(
            clock,
            clock_span,
            &timeline.mb,
            &observe.mb,
            &key.mb,
            storage_type_map,
            device,
            queue,
            format,
            result,
        );
    }
}

//

impl Prepare for MyMobject1 {
    type Resource = [f32; 4];

    fn prepare_new(
        input: <MyMobject1 as Mobject>::ResourceRefInput<'_>,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> Self::Resource {
        [input.ma.ma, input.ma.mb, input.mb.ma, input.mb.mb]
    }

    fn prepare_incremental(
        resource: &mut Self::Resource,
        input: <MyMobject1 as Mobject>::ResourceRefInput<'_>,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> ResourceReuseResult {
        *resource = [input.ma.ma, input.ma.mb, input.mb.ma, input.mb.mb];
        Ok(())
    }
}

impl Render for MyMobject1 {
    fn render(
        _resource: <MyMobject1 as Mobject>::ResourceRef<'_>,
        _render_pass: &mut wgpu::RenderPass,
    ) {
        ()
    }
}

//

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject1<MA = MyMobject0, MB = MyMobject0> {
    ma: MA,
    mb: MB,
}

impl Mobject for MyMobject1 {
    type StaticKey = (
        MyMobject1<<MyMobject0 as Mobject>::StaticKey, <MyMobject0 as Mobject>::StaticKey>,
        StorageKey<StaticVariant<MyMobject1>>,
    );
    type DynamicKey = (
        MyMobject1<<MyMobject0 as Mobject>::StaticKey, <MyMobject0 as Mobject>::StaticKey>,
        StorageKey<DynamicVariant<MyMobject1>>,
    );
    type ResourceRefInput<'s> = MyMobject1<
        <MyMobject0 as Mobject>::ResourceRef<'s>,
        <MyMobject0 as Mobject>::ResourceRef<'s>,
    >;
    type ResourceRef<'s> = &'s <MyMobject1 as Prepare>::Resource;

    fn static_allocate(
        mobject: &Self,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::StaticKey {
        (
            MyMobject1 {
                ma: <MyMobject0 as Mobject>::static_allocate(&mobject.ma, slot_key_generator_map),
                mb: <MyMobject0 as Mobject>::static_allocate(&mobject.mb, slot_key_generator_map),
            },
            slot_key_generator_map.allocate(mobject),
        )
    }

    fn dynamic_allocate(
        mobject: &Self,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::DynamicKey {
        (
            MyMobject1 {
                ma: <MyMobject0 as Mobject>::dynamic_allocate(&mobject.ma, slot_key_generator_map),
                mb: <MyMobject0 as Mobject>::dynamic_allocate(&mobject.mb, slot_key_generator_map),
            },
            slot_key_generator_map.allocate(&()),
        )
    }

    fn static_prepare<'s>(
        mobject: &Self,
        key: &Self::StaticKey,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        result: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        let input = MyMobject1 {
            ma: <MyMobject0 as Mobject>::static_prepare(
                &mobject.ma,
                &key.0.ma,
                storage_type_map,
                device,
                queue,
                format,
                result,
            ),
            mb: <MyMobject0 as Mobject>::static_prepare(
                &mobject.mb,
                &key.0.mb,
                storage_type_map,
                device,
                queue,
                format,
                result,
            ),
        };
        storage_type_map.update_or_insert(
            &key.1,
            (input, device, queue, format),
            result,
            |(input, device, queue, format)| {
                Arc::new(<MyMobject1 as Prepare>::prepare_new(
                    input, device, queue, format,
                ))
            },
            |(_input, _device, _queue, _format), _resource, _result| {},
        )
    }

    fn dynamic_prepare<'s>(
        mobject: &Self,
        key: &Self::DynamicKey,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        result: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        let input = MyMobject1 {
            ma: <MyMobject0 as Mobject>::dynamic_prepare(
                &mobject.ma,
                &key.0.ma,
                storage_type_map,
                device,
                queue,
                format,
                result,
            ),
            mb: <MyMobject0 as Mobject>::dynamic_prepare(
                &mobject.mb,
                &key.0.mb,
                storage_type_map,
                device,
                queue,
                format,
                result,
            ),
        };
        storage_type_map.update_or_insert(
            &key.1,
            (input, device, queue, format),
            result,
            |(input, device, queue, format)| {
                <MyMobject1 as Prepare>::prepare_new(input, device, queue, format)
            },
            |(input, device, queue, format), resource, result| {
                *result = <MyMobject1 as Prepare>::prepare_incremental(
                    resource, input, device, queue, format,
                );
            },
        )
    }

    fn generic_render(resource: Self::ResourceRef<'_>, render_pass: &mut wgpu::RenderPass) {
        <MyMobject1 as Render>::render(resource, render_pass);
    }
}

impl<MA, MB> Timeline<MyMobject1<MyMobject0, MyMobject0>> for MyMobject1<MA, MB>
where
    MA: Timeline<MyMobject0>,
    MB: Timeline<MyMobject0>,
{
    type Observe = MyMobject1<MA::Observe, MB::Observe>;
    type Key = MyMobject1<MA::Key, MB::Key>;

    fn allocate(
        timeline: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key {
        MyMobject1 {
            ma: MA::allocate(&timeline.ma, &observe.ma, slot_key_generator_map),
            mb: MB::allocate(&timeline.mb, &observe.mb, slot_key_generator_map),
        }
    }

    fn prepare(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &Self::Observe,
        key: &Self::Key,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        result: &mut ResourceReuseResult,
    ) {
        MA::prepare(
            clock,
            clock_span,
            &timeline.ma,
            &observe.ma,
            &key.ma,
            storage_type_map,
            device,
            queue,
            format,
            result,
        );
        MB::prepare(
            clock,
            clock_span,
            &timeline.mb,
            &observe.mb,
            &key.mb,
            storage_type_map,
            device,
            queue,
            format,
            result,
        );
    }
}
