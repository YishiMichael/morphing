use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use super::storage::MultitonSlot;
use super::storage::ResourceReuseResult;
use super::storage::SingletonSlot;
use super::storage::SlotKeyGeneratorTypeMap;
use super::storage::StorageKey;
use super::storage::StorageTypeMap;
use super::storage::StoreType;
use super::storage::SwapSlot;
use super::timer::Clock;
use super::timer::ClockSpan;

pub trait Mobject:
    'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
{
    type StaticKeys;
    type DynamicKeys;
    // type ResourceInput;
    // type ResourceIntrinsic;
    // type ResourceExtrinsic;
    type ResourceRef<'s>;
    type ResourceRefInput<'s>;
    // type ResourceRef<'s>;

    fn static_allocate(
        mobject: &Self,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::StaticKeys;
    fn dynamic_allocate(slot_key_generator_map: &mut SlotKeyGeneratorTypeMap) -> Self::DynamicKeys;
    // fn prepare_intrinsic_new(
    //     mobject: &Self,
    //     // key: &Self::StaticKey,
    //     // storage_type_map: &'s mut StorageTypeMap,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     // reuse: &mut ResourceReuseResult,
    // ) -> Self::ResourceIntrinsic;
    // fn prepare_intrinsic_update(
    //     mobject: &Self,
    //     resource_intrinsic: &mut Self::ResourceIntrinsic,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     reuse: &mut ResourceReuseResult,
    // );
    // fn prepare_extrinsic_new(
    //     resource_intrinsic: &Self::ResourceIntrinsic,
    //     // key: &Self::StaticKey,
    //     // storage_type_map: &'s mut StorageTypeMap,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     // reuse: &mut ResourceReuseResult,
    // ) -> Self::ResourceExtrinsic;
    // fn prepare_extrinsic_update(
    //     resource_intrinsic: &Self::ResourceIntrinsic,
    //     resource_extrinsic: &mut Self::ResourceExtrinsic,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     reuse: &mut ResourceReuseResult,
    // );
    // fn prepare_new(
    //     mobject: &Self,
    //     // key: &Self::StaticKey,
    //     // storage_type_map: &'s mut StorageTypeMap,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     // reuse: &mut ResourceReuseResult,
    // ) -> Self::Resource;
    // fn prepare_update(
    //     mobject: &Self,
    //     resource: &mut Self::Resource,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     reuse: &mut ResourceReuseResult,
    // );
    fn static_refresh<'s>(
        mobject: &Self,
        static_keys: &Self::StaticKeys,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s>;
    fn dynamic_refresh<'s>(
        mobject: &Self,
        dynamic_keys: &Self::DynamicKeys,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s>;
    fn static_fetch<'s>(
        static_keys: &Self::StaticKeys,
        storage_type_map: &'s StorageTypeMap,
    ) -> Self::ResourceRef<'s>;
    fn dynamic_fetch<'s>(
        dynamic_keys: &Self::DynamicKeys,
        storage_type_map: &'s StorageTypeMap,
    ) -> Self::ResourceRef<'s>;
    // fn static_prepare<'s>(resource_ref_input: Self::ResourceRefInput<'s>, )
    // fn dynamic_prepare_new<'s>(
    //     mobject: &'s Self,
    //     key: &Self::DynamicKey,
    //     storage_type_map: &'s mut StorageTypeMap,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     reuse: &mut ResourceReuseResult,
    // ) -> Self::Resource;
    // fn static_render(
    //     key: &Self::StaticKey,
    //     storage_type_map: &StorageTypeMap,
    //     render_pass: &mut wgpu::RenderPass,
    // );
    // fn dynamic_render(
    //     key: &Self::DynamicKey,
    //     storage_type_map: &StorageTypeMap,
    //     render_pass: &mut wgpu::RenderPass,
    // );
    fn render(resource: Self::ResourceRef<'_>, render_pass: &mut wgpu::RenderPass);
}

pub trait Variant<M>
where
    M: Mobject,
{
    type Observe: Send + Sync;
    type Keys;
    // type ResourceRef<'s>;

    fn allocate(
        // timeline: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys;
    fn prepare<'s>(
        observe: &Self::Observe,
        keys: &Self::Keys,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> M::ResourceRef<'s>;
    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    );
}

pub trait Timeline<M>: 'static + Send + Sized + Sync
where
    M: Mobject,
{
    // type Mobject: Mobject;
    type Variant: Variant<M>;
    // type Observe: Send + Sync;
    // type Variant: Variant<M>;

    fn observe(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &<Self::Variant as Variant<M>>::Observe,
    ) -> <Self::Variant as Variant<M>>::Observe;
}

pub trait Resource<RR>: 'static + Send + Sync {
    fn new(
        resource_repr: RR,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self;
    fn update(
        resource: &mut Self,
        resource_repr: RR,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    );
}

// pub trait PreparableMobject: Mobject {
//     type ResourceInput;
// }

pub trait Prepare<M>
where
    M: Mobject,
{
    type ResourceRepr;
    type Resource: Resource<Self::ResourceRepr>;

    fn prepare(input: M::ResourceRefInput<'_>) -> Self::ResourceRepr;

    fn static_prepare<'s>(
        input: M::ResourceRefInput<'_>,
        static_key: &StorageKey<StaticStoreType<M, Self::Resource>>,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> &'s Self::Resource {
        storage_type_map.update_or_insert(
            static_key,
            (input, device, queue, format),
            reuse,
            |(input, device, queue, format)| {
                Arc::new(<Self::Resource as Resource<Self::ResourceRepr>>::new(
                    Self::prepare(input),
                    device,
                    queue,
                    format,
                ))
            },
            |(_input, _device, _queue, _format), _resource, _reuse| {},
        )
    }

    fn dynamic_prepare<'s>(
        input: M::ResourceRefInput<'_>,
        dynamic_key: &StorageKey<DynamicStoreType<M, Self::Resource>>,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> &'s Self::Resource {
        storage_type_map.update_or_insert(
            dynamic_key,
            (input, device, queue, format),
            reuse,
            |(input, device, queue, format)| {
                <Self::Resource as Resource<Self::ResourceRepr>>::new(
                    Self::prepare(input),
                    device,
                    queue,
                    format,
                )
            },
            |(input, device, queue, format), resource, reuse| {
                <Self::Resource as Resource<Self::ResourceRepr>>::update(
                    resource,
                    Self::prepare(input),
                    device,
                    queue,
                    format,
                    reuse,
                )
            },
        )
    }
}

pub trait Render<M>
where
    M: Mobject,
{
    fn render(resource_ref: M::ResourceRef<'_>, render_pass: &mut wgpu::RenderPass);
}

pub struct Derivation<I, E> {
    pub intrinsic: I,
    pub extrinsic: E,
}

// pub trait Variant<M>: StoreType
// where
//     M: Mobject,
// {
//     type Observe;
//     type KeyInput: serde::Serialize;
//     type Slot: Slot;
// }

pub struct StaticStoreType<M, R>(M, R);

impl<M, R> StoreType for StaticStoreType<M, R>
where
    M: Mobject,
    R: 'static + Send + Sync,
{
    type KeyInput = M;
    type Slot = SwapSlot<SingletonSlot<R>>;
}

// impl<M> Variant<M> for StaticVariant<M>
// where
//     M: Mobject,
// {
//     type Observe = Arc<M>;
// }

pub struct DynamicStoreType<M, R>(M, R);

impl<M, R> StoreType for DynamicStoreType<M, R>
where
    M: Mobject,
    R: 'static + Send + Sync,
{
    type KeyInput = ();
    type Slot = SwapSlot<MultitonSlot<R>>;
}

// impl<M> Variant<M> for DynamicVariant<M>
// where
//     M: Mobject,
// {
//     type Observe = M;
// }

pub trait Refresh<M>: 'static + Send + Sync
where
    M: Mobject,
{
    fn refresh(&self, clock: Clock, clock_span: ClockSpan, mobject: M) -> M;
}

pub struct StaticVariant;

impl<M> Variant<M> for StaticVariant
where
    M: Mobject,
{
    type Observe = Arc<M>;
    type Keys = M::StaticKeys;
    // type ResourceRef<'s> = M::ResourceRef<'s>;

    fn allocate(
        // _timeline: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        // slot_key_generator_map.allocate(observe.as_ref())
        M::static_allocate(observe.as_ref(), slot_key_generator_map)
    }

    fn prepare<'s>(
        observe: &Self::Observe,
        keys: &Self::Keys,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> M::ResourceRef<'s> {
        // let mobject = observe.as_ref();
        // storage_type_map.update_or_insert(
        //     key,
        //     (observe, device, queue, format),
        //     reuse,
        //     |(observe, device, queue, format)| {
        //         Arc::new(M::prepare_new(observe.as_ref(), device, queue, format))
        //     },
        //     |(_observe, _device, _queue, _format), _resource, _reuse| {},
        // )
        M::static_refresh(
            observe.as_ref(),
            keys,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        )
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        // M::render(storage_type_map.get_and_unwrap(key), render_pass);
        M::render(M::static_fetch(keys, storage_type_map), render_pass);
    }
}

pub struct StaticTimeline;

impl<M> Timeline<M> for StaticTimeline
where
    M: Mobject,
{
    type Variant = StaticVariant;

    fn observe(
        _clock: Clock,
        _clock_span: ClockSpan,
        _timeline: &Self,
        observe: &<Self::Variant as Variant<M>>::Observe,
    ) -> <Self::Variant as Variant<M>>::Observe {
        observe.clone()
    }
}

pub struct DynamicVariant;

impl<M> Variant<M> for DynamicVariant
where
    M: Mobject,
{
    type Observe = M;
    type Keys = M::DynamicKeys;
    // type ResourceRef<'s> = M::ResourceRef<'s>;

    fn allocate(
        // _timeline: &Self,
        _observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        // slot_key_generator_map.allocate(&())
        M::dynamic_allocate(slot_key_generator_map)
    }

    fn prepare<'s>(
        observe: &Self::Observe,
        keys: &Self::Keys,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> M::ResourceRef<'s> {
        // storage_type_map.update_or_insert(
        //     key,
        //     (observe, device, queue, format),
        //     reuse,
        //     |(observe, device, queue, format)| M::prepare_new(observe, device, queue, format),
        //     |(observe, device, queue, format), resource, reuse| {
        //         M::prepare_update(observe, resource, device, queue, format, reuse)
        //     },
        // )
        M::dynamic_refresh(
            observe,
            // &timeline.refresh.refresh(clock, clock_span, observe.clone()),
            keys,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        )
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        // M::render(storage_type_map.get_and_unwrap(key), render_pass);
        M::render(M::dynamic_fetch(keys, storage_type_map), render_pass);
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
    type Variant = DynamicVariant;

    fn observe(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &<Self::Variant as Variant<M>>::Observe,
    ) -> <Self::Variant as Variant<M>>::Observe {
        timeline.refresh.refresh(clock, clock_span, observe.clone())
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[repr(transparent)]
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

impl<T> Resource<T> for T
where
    T: 'static + Send + Sync,
{
    fn new(
        resource_repr: T,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> Self {
        resource_repr
    }

    fn update(
        resource: &mut Self,
        resource_repr: T,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        _reuse: &mut ResourceReuseResult,
    ) {
        *resource = resource_repr;
    }
}

pub struct DataPrepare;

impl<T> Prepare<Data<T>> for DataPrepare
where
    T: 'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    type ResourceRepr = T;
    type Resource = T;

    fn prepare(input: <Data<T> as Mobject>::ResourceRefInput<'_>) -> Self::ResourceRepr {
        input.clone()
    }
}

impl<T> Mobject for Data<T>
where
    T: 'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    type StaticKeys = StorageKey<StaticStoreType<Data<T>, T>>;
    type DynamicKeys = StorageKey<DynamicStoreType<Data<T>, T>>;
    // type ResourceIntrinsic = T;
    // type ResourceExtrinsic = ();
    type ResourceRef<'s> = &'s T;
    type ResourceRefInput<'s> = &'s T;

    fn static_allocate(
        mobject: &Self,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::StaticKeys {
        slot_key_generator_map.allocate(mobject)
    }

    fn dynamic_allocate(slot_key_generator_map: &mut SlotKeyGeneratorTypeMap) -> Self::DynamicKeys {
        slot_key_generator_map.allocate(&())
    }

    fn static_refresh<'s>(
        mobject: &Self,
        static_keys: &Self::StaticKeys,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        // storage_type_map.update_or_insert(
        //     static_keys,
        //     (mobject, device, queue, format),
        //     reuse,
        //     |(mobject, device, queue, format)| {
        //         Arc::new(M::prepare_new(mobject, device, queue, format))
        //     },
        //     |(_mobject, _device, _queue, _format), _resource, _reuse| {},
        // )
        <DataPrepare as Prepare<Data<T>>>::static_prepare(
            mobject,
            static_keys,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        )
        // storage_type_map.update_or_insert(
        //     static_keys,
        //     mobject,
        //     reuse,
        //     |mobject| Arc::new((&**mobject).clone()),
        //     |_mobject, _resource, _reuse| {},
        // )
    }

    fn dynamic_refresh<'s>(
        mobject: &Self,
        dynamic_keys: &Self::DynamicKeys,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        // storage_type_map.update_or_insert(
        //     key,
        //     (observe, device, queue, format),
        //     reuse,
        //     |(observe, device, queue, format)| M::prepare_new(observe, device, queue, format),
        //     |(observe, device, queue, format), resource, reuse| {
        //         M::prepare_update(observe, resource, device, queue, format, reuse)
        //     },
        // )
        <DataPrepare as Prepare<Data<T>>>::dynamic_prepare(
            mobject,
            dynamic_keys,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        )
    }

    fn static_fetch<'s>(
        static_keys: &Self::StaticKeys,
        storage_type_map: &'s StorageTypeMap,
    ) -> Self::ResourceRef<'s> {
        storage_type_map.get_and_unwrap(static_keys)
    }

    fn dynamic_fetch<'s>(
        dynamic_keys: &Self::DynamicKeys,
        storage_type_map: &'s StorageTypeMap,
    ) -> Self::ResourceRef<'s> {
        storage_type_map.get_and_unwrap(dynamic_keys)
    }

    // fn prepare_new(
    //     mobject: &Self,
    //     // key: &Self::StaticKey,
    //     // storage_type_map: &'s mut StorageTypeMap,
    //     _device: &wgpu::Device,
    //     _queue: &wgpu::Queue,
    //     _format: wgpu::TextureFormat,
    //     // reuse: &mut ResourceReuseResult,
    // ) -> Self::Resource {
    //     (&**mobject).clone()
    // }

    // fn prepare_update(
    //     mobject: &Self,
    //     resource: &mut Self::Resource,
    //     _device: &wgpu::Device,
    //     _queue: &wgpu::Queue,
    //     _format: wgpu::TextureFormat,
    //     _reuse: &mut ResourceReuseResult,
    // ) {
    //     *resource = (&**mobject).clone();
    // }

    // fn prepare_extrinsic_new(
    //     _resource_intrinsic: &Self::ResourceIntrinsic,
    //     // key: &Self::StaticKey,
    //     // storage_type_map: &'s mut StorageTypeMap,
    //     _device: &wgpu::Device,
    //     _queue: &wgpu::Queue,
    //     _format: wgpu::TextureFormat,
    //     // reuse: &mut ResourceReuseResult,
    // ) -> Self::ResourceExtrinsic {
    //     ()
    // }

    // fn prepare_extrinsic_update(
    //     _resource_intrinsic: &Self::ResourceIntrinsic,
    //     _resource_extrinsic: &mut Self::ResourceExtrinsic,
    //     _device: &wgpu::Device,
    //     _queue: &wgpu::Queue,
    //     _format: wgpu::TextureFormat,
    //     _reuse: &mut ResourceReuseResult,
    // ) {}

    // fn prepare_new(
    //     mobject: &Self,
    //     // key: &Self::StaticKey,
    //     // storage_type_map: &'s mut StorageTypeMap,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     // reuse: &mut ResourceReuseResult,
    // ) -> Self::Resource {
    //     Self::prepare_intrinsic_new(mobject, device, queue, format)
    // }

    // fn prepare_update(
    //     mobject: &Self,
    //     resource: &mut Self::Resource,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     reuse: &mut ResourceReuseResult,
    // ) {
    //     Self::prepare_intrinsic_update(mobject, resource, device, queue, format, reuse);
    // }

    fn render(_resource_ref: Self::ResourceRef<'_>, _render_pass: &mut wgpu::RenderPass) {}

    // fn static_allocate(
    //     _mobject: &Self,
    //     _slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    // ) -> Self::StaticKey {
    //     ()
    // }

    // fn dynamic_allocate(
    //     _mobject: &Self,
    //     _slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    // ) -> Self::DynamicKey {
    //     ()
    // }

    // fn static_prepare<'s>(
    //     mobject: &'s Self,
    //     _key: &Self::StaticKey,
    //     _storage_type_map: &'s mut StorageTypeMap,
    //     _device: &wgpu::Device,
    //     _queue: &wgpu::Queue,
    //     _format: wgpu::TextureFormat,
    //     _reuse: &mut ResourceReuseResult,
    // ) -> Self::ResourceRef<'s> {
    //     &mobject
    // }

    // fn dynamic_prepare<'s>(
    //     mobject: &'s Self,
    //     _key: &Self::DynamicKey,
    //     _storage_type_map: &'s mut StorageTypeMap,
    //     _device: &wgpu::Device,
    //     _queue: &wgpu::Queue,
    //     _format: wgpu::TextureFormat,
    //     _reuse: &mut ResourceReuseResult,
    // ) -> Self::ResourceRef<'s> {
    //     &mobject
    // }

    // fn static_render(
    //     _key: &Self::StaticKey,
    //     _storage_type_map: &StorageTypeMap,
    //     _render_pass: &mut wgpu::RenderPass,
    // ) {
    // }

    // fn dynamic_render(
    //     _key: &Self::DynamicKey,
    //     _storage_type_map: &StorageTypeMap,
    //     _render_pass: &mut wgpu::RenderPass,
    // ) {
    // }
}

// demo

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject0<MA = Data<f32>, MB = Data<f32>> {
    ma: MA,
    mb: MB,
}

impl Mobject for MyMobject0 {
    type StaticKeys =
        MyMobject0<<Data<f32> as Mobject>::StaticKeys, <Data<f32> as Mobject>::StaticKeys>;
    type DynamicKeys =
        MyMobject0<<Data<f32> as Mobject>::DynamicKeys, <Data<f32> as Mobject>::DynamicKeys>;
    // type ResourceIntrinsic =
    //     MyMobject0<<Data<f32> as Mobject>::Resource, <Data<f32> as Mobject>::Resource>;
    // type ResourceExtrinsic = ();
    // type Resource = MyMobject0<<Data<f32> as Mobject>::Resource, <Data<f32> as Mobject>::Resource>;
    type ResourceRef<'s> = MyMobject0<
        <Data<f32> as Mobject>::ResourceRef<'s>,
        <Data<f32> as Mobject>::ResourceRef<'s>,
    >;
    type ResourceRefInput<'s> = MyMobject0<
        <Data<f32> as Mobject>::ResourceRef<'s>,
        <Data<f32> as Mobject>::ResourceRef<'s>,
    >;

    fn static_allocate(
        mobject: &Self,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::StaticKeys {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::static_allocate(&mobject.ma, slot_key_generator_map),
            mb: <Data<f32> as Mobject>::static_allocate(&mobject.mb, slot_key_generator_map),
        }
    }

    fn dynamic_allocate(slot_key_generator_map: &mut SlotKeyGeneratorTypeMap) -> Self::DynamicKeys {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::dynamic_allocate(slot_key_generator_map),
            mb: <Data<f32> as Mobject>::dynamic_allocate(slot_key_generator_map),
        }
    }

    fn static_refresh<'s>(
        mobject: &Self,
        static_keys: &Self::StaticKeys,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::static_refresh(
                &mobject.ma,
                &static_keys.ma,
                storage_type_map,
                device,
                queue,
                format,
                reuse,
            ),
            mb: <Data<f32> as Mobject>::static_refresh(
                &mobject.mb,
                &static_keys.mb,
                storage_type_map,
                device,
                queue,
                format,
                reuse,
            ),
        }
    }

    fn dynamic_refresh<'s>(
        mobject: &Self,
        dynamic_keys: &Self::DynamicKeys,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::dynamic_refresh(
                &mobject.ma,
                &dynamic_keys.ma,
                storage_type_map,
                device,
                queue,
                format,
                reuse,
            ),
            mb: <Data<f32> as Mobject>::dynamic_refresh(
                &mobject.mb,
                &dynamic_keys.mb,
                storage_type_map,
                device,
                queue,
                format,
                reuse,
            ),
        }
    }

    fn static_fetch<'s>(
        static_keys: &Self::StaticKeys,
        storage_type_map: &'s StorageTypeMap,
    ) -> Self::ResourceRef<'s> {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::static_fetch(&static_keys.ma, storage_type_map),
            mb: <Data<f32> as Mobject>::static_fetch(&static_keys.mb, storage_type_map),
        }
    }

    fn dynamic_fetch<'s>(
        dynamic_keys: &Self::DynamicKeys,
        storage_type_map: &'s StorageTypeMap,
    ) -> Self::ResourceRef<'s> {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::dynamic_fetch(&dynamic_keys.ma, storage_type_map),
            mb: <Data<f32> as Mobject>::dynamic_fetch(&dynamic_keys.mb, storage_type_map),
        }
    }

    // fn prepare_new(
    //     mobject: &Self,
    //     // key: &Self::StaticKey,
    //     // storage_type_map: &'s mut StorageTypeMap,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     // reuse: &mut ResourceReuseResult,
    // ) -> Self::Resource {
    //     MyMobject0 {
    //         ma: <Data<f32> as Mobject>::prepare_new(&mobject.ma, device, queue, format),
    //         mb: <Data<f32> as Mobject>::prepare_new(&mobject.mb, device, queue, format),
    //     }
    // }

    // fn prepare_update(
    //     mobject: &Self,
    //     resource_intrinsic: &mut Self::Resource,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     reuse: &mut ResourceReuseResult,
    // ) {
    //     <Data<f32> as Mobject>::prepare_update(
    //         &mobject.ma,
    //         &mut resource_intrinsic.ma,
    //         device,
    //         queue,
    //         format,
    //         reuse,
    //     );
    //     <Data<f32> as Mobject>::prepare_update(
    //         &mobject.mb,
    //         &mut resource_intrinsic.mb,
    //         device,
    //         queue,
    //         format,
    //         reuse,
    //     );
    // }

    // fn prepare_extrinsic_new(
    //     _resource_intrinsic: &Self::ResourceIntrinsic,
    //     _device: &wgpu::Device,
    //     _queue: &wgpu::Queue,
    //     _format: wgpu::TextureFormat,
    // ) -> Self::ResourceExtrinsic {
    //     ()
    // }

    // fn prepare_extrinsic_update(
    //     _resource_intrinsic: &Self::ResourceIntrinsic,
    //     _resource_extrinsic: &mut Self::ResourceExtrinsic,
    //     _device: &wgpu::Device,
    //     _queue: &wgpu::Queue,
    //     _format: wgpu::TextureFormat,
    //     _reuse: &mut ResourceReuseResult,
    // ) {}

    // fn prepare_new(
    //     mobject: &Self,
    //     // key: &Self::StaticKey,
    //     // storage_type_map: &'s mut StorageTypeMap,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     // reuse: &mut ResourceReuseResult,
    // ) -> Self::Resource {
    //     Self::prepare_intrinsic_new(mobject, device, queue, format)
    // }

    // fn prepare_update(
    //     mobject: &Self,
    //     resource: &mut Self::Resource,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     reuse: &mut ResourceReuseResult,
    // ) {
    //     Self::prepare_intrinsic_update(mobject, resource, device, queue, format, reuse);
    // }

    fn render(resource_ref: Self::ResourceRef<'_>, render_pass: &mut wgpu::RenderPass) {
        <Data<f32> as Mobject>::render(resource_ref.ma, render_pass);
        <Data<f32> as Mobject>::render(resource_ref.mb, render_pass);
    }

    // fn static_allocate(
    //     mobject: &Self,
    //     slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    // ) -> Self::StaticKey {
    //     MyMobject0 {
    //         ma: <Data<f32> as Mobject>::static_allocate(&mobject.ma, slot_key_generator_map),
    //         mb: <Data<f32> as Mobject>::static_allocate(&mobject.mb, slot_key_generator_map),
    //     }
    // }

    // fn dynamic_allocate(
    //     mobject: &Self,
    //     slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    // ) -> Self::DynamicKey {
    //     MyMobject0 {
    //         ma: <Data<f32> as Mobject>::dynamic_allocate(&mobject.ma, slot_key_generator_map),
    //         mb: <Data<f32> as Mobject>::dynamic_allocate(&mobject.mb, slot_key_generator_map),
    //     }
    // }

    // fn static_prepare<'s>(
    //     mobject: &'s Self,
    //     key: &Self::StaticKey,
    //     storage_type_map: &'s mut StorageTypeMap,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     reuse: &mut ResourceReuseResult,
    // ) -> &'s Self::Resource {
    //     MyMobject0 {
    //         ma: <Data<f32> as Mobject>::static_prepare(
    //             &mobject.ma,
    //             &key.ma,
    //             storage_type_map,
    //             device,
    //             queue,
    //             format,
    //             reuse,
    //         ),
    //         mb: <Data<f32> as Mobject>::static_prepare(
    //             &mobject.mb,
    //             &key.mb,
    //             storage_type_map,
    //             device,
    //             queue,
    //             format,
    //             reuse,
    //         ),
    //     }
    // }

    // fn dynamic_prepare<'s>(
    //     mobject: &'s Self,
    //     key: &Self::DynamicKey,
    //     storage_type_map: &'s mut StorageTypeMap,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     reuse: &mut ResourceReuseResult,
    // ) -> &'s Self::Resource {
    //     MyMobject0 {
    //         ma: <Data<f32> as Mobject>::dynamic_prepare(
    //             &mobject.ma,
    //             &key.ma,
    //             storage_type_map,
    //             device,
    //             queue,
    //             format,
    //             reuse,
    //         ),
    //         mb: <Data<f32> as Mobject>::dynamic_prepare(
    //             &mobject.mb,
    //             &key.mb,
    //             storage_type_map,
    //             device,
    //             queue,
    //             format,
    //             reuse,
    //         ),
    //     }
    // }

    // fn render(resource: Self::ResourceRef<'_>, render_pass: &mut wgpu::RenderPass) {
    //     <Data<f32> as Mobject>::render(resource.ma, render_pass);
    //     <Data<f32> as Mobject>::render(resource.mb, render_pass);
    // }
}

impl<MA, MB> Variant<MyMobject0> for MyMobject0<MA, MB>
where
    MA: Variant<Data<f32>>,
    MB: Variant<Data<f32>>,
{
    type Observe = MyMobject0<MA::Observe, MB::Observe>;
    type Keys = MyMobject0<MA::Keys, MB::Keys>;
    // type ResourceRef<'s> = MyMobject0<MA::ResourceRef<'s>, MB::ResourceRef<'s>>;

    fn allocate(
        // timeline: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        MyMobject0 {
            ma: MA::allocate(&observe.ma, slot_key_generator_map),
            mb: MB::allocate(&observe.mb, slot_key_generator_map),
        }
    }

    fn prepare<'s>(
        observe: &Self::Observe,
        keys: &Self::Keys,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> <MyMobject0 as Mobject>::ResourceRef<'s> {
        MyMobject0 {
            ma: MA::prepare(
                &observe.ma,
                &keys.ma,
                storage_type_map,
                device,
                queue,
                format,
                reuse,
            ),
            mb: MB::prepare(
                &observe.mb,
                &keys.mb,
                storage_type_map,
                device,
                queue,
                format,
                reuse,
            ),
        }
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        MA::render(&keys.ma, storage_type_map, render_pass);
        MB::render(&keys.mb, storage_type_map, render_pass);
    }
}

impl<MA, MB> Timeline<MyMobject0> for MyMobject0<MA, MB>
where
    MA: Timeline<Data<f32>>,
    MB: Timeline<Data<f32>>,
{
    type Variant = MyMobject0<MA::Variant, MB::Variant>;

    fn observe(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &<Self::Variant as Variant<MyMobject0>>::Observe,
    ) -> <Self::Variant as Variant<MyMobject0>>::Observe {
        MyMobject0 {
            ma: MA::observe(clock, clock_span, &timeline.ma, &observe.ma),
            mb: MB::observe(clock, clock_span, &timeline.mb, &observe.mb),
        }
    }
}

//

pub struct MyMobject1Resource([[f32; 4]; 2]);

impl Resource<[f32; 4]> for MyMobject1Resource {
    fn new(
        resource_repr: [f32; 4],
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> Self {
        Self([resource_repr, resource_repr])
    }

    fn update(
        resource: &mut Self,
        resource_repr: [f32; 4],
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        _reuse: &mut ResourceReuseResult,
    ) {
        resource.0 = [resource_repr, resource_repr];
    }
}

pub struct MyMobject1Prepare;

impl Prepare<MyMobject1> for MyMobject1Prepare {
    type ResourceRepr = [f32; 4];
    type Resource = MyMobject1Resource;

    fn prepare(input: <MyMobject1 as Mobject>::ResourceRefInput<'_>) -> Self::ResourceRepr {
        [*input.ma.ma, *input.ma.mb, *input.mb.ma, *input.mb.mb]
    }
}

pub struct MyMobject1Render;

impl Render<MyMobject1> for MyMobject1Render {
    fn render(
        _resource_ref: <MyMobject1 as Mobject>::ResourceRef<'_>,
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
    type StaticKeys = Derivation<
        MyMobject1<<MyMobject0 as Mobject>::StaticKeys, <MyMobject0 as Mobject>::StaticKeys>,
        StorageKey<
            StaticStoreType<MyMobject1, <MyMobject1Prepare as Prepare<MyMobject1>>::Resource>,
        >,
    >;
    type DynamicKeys = Derivation<
        MyMobject1<<MyMobject0 as Mobject>::DynamicKeys, <MyMobject0 as Mobject>::DynamicKeys>,
        StorageKey<
            DynamicStoreType<MyMobject1, <MyMobject1Prepare as Prepare<MyMobject1>>::Resource>,
        >,
    >;
    // type ResourceIntrinsic =
    //     MyMobject1<<MyMobject0 as Mobject>::Resource, <MyMobject0 as Mobject>::Resource>;
    // type ResourceExtrinsic = <MyMobject1Prepare as Prepare<MyMobject1>>::ResourceExtrinsic;
    type ResourceRef<'s> = &'s <MyMobject1Prepare as Prepare<MyMobject1>>::Resource;
    type ResourceRefInput<'s> = MyMobject1<
        <MyMobject0 as Mobject>::ResourceRef<'s>,
        <MyMobject0 as Mobject>::ResourceRef<'s>,
    >;
    // type ResourceRefInput<'s> =
    //     MyMobject1<&'s <MyMobject0 as Mobject>::Resource, &'s <MyMobject0 as Mobject>::Resource>;
    // type ResourceRef<'s> = &'s <MyMobject1Prepare as Prepare<MyMobject1>>::Resource;

    fn static_allocate(
        mobject: &Self,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::StaticKeys {
        Derivation {
            intrinsic: MyMobject1 {
                ma: <MyMobject0 as Mobject>::static_allocate(&mobject.ma, slot_key_generator_map),
                mb: <MyMobject0 as Mobject>::static_allocate(&mobject.mb, slot_key_generator_map),
            },
            extrinsic: slot_key_generator_map.allocate(mobject),
        }
    }

    fn dynamic_allocate(slot_key_generator_map: &mut SlotKeyGeneratorTypeMap) -> Self::DynamicKeys {
        Derivation {
            intrinsic: MyMobject1 {
                ma: <MyMobject0 as Mobject>::dynamic_allocate(slot_key_generator_map),
                mb: <MyMobject0 as Mobject>::dynamic_allocate(slot_key_generator_map),
            },
            extrinsic: slot_key_generator_map.allocate(&()),
        }
    }

    fn static_refresh<'s>(
        mobject: &Self,
        static_keys: &Self::StaticKeys,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        <MyMobject1Prepare as Prepare<MyMobject1>>::static_prepare(
            MyMobject1 {
                ma: <MyMobject0 as Mobject>::static_refresh(
                    &mobject.ma,
                    &static_keys.intrinsic.ma,
                    storage_type_map,
                    device,
                    queue,
                    format,
                    reuse,
                ),
                mb: <MyMobject0 as Mobject>::static_refresh(
                    &mobject.mb,
                    &static_keys.intrinsic.mb,
                    storage_type_map,
                    device,
                    queue,
                    format,
                    reuse,
                ),
            },
            &static_keys.extrinsic,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        )
        // storage_type_map.update_or_insert(
        //     key,
        //     (mobject, device, queue, format),
        //     reuse,
        //     |(mobject, device, queue, format)| M::prepare_new(mobject, device, queue, format),
        //     |(mobject, device, queue, format), resource, reuse| {
        //         M::prepare_update(mobject, resource, device, queue, format, reuse)
        //     },
        // )
        // <<MyMobject1Prepare as Prepare<MyMobject1>>::Resource as Resource<
        //     <MyMobject1Prepare as Prepare<MyMobject1>>::ResourceRepr,
        // >>::new(
        //     MyMobject1 {
        //         ma: <MyMobject0 as Mobject>::static_refresh(&mobject.ma, &static_keys.ma, storage_type_map, device, queue, format, reuse),
        //         mb: <MyMobject0 as Mobject>::static_refresh(&mobject.mb, &static_keys.mb, storage_type_map, device, queue, format, reuse),
        //     },
        //     device,
        //     queue,
        //     format,
        // )
    }

    fn dynamic_refresh<'s>(
        mobject: &Self,
        dynamic_keys: &Self::DynamicKeys,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        <MyMobject1Prepare as Prepare<MyMobject1>>::dynamic_prepare(
            MyMobject1 {
                ma: <MyMobject0 as Mobject>::dynamic_refresh(
                    &mobject.ma,
                    &dynamic_keys.intrinsic.ma,
                    storage_type_map,
                    device,
                    queue,
                    format,
                    reuse,
                ),
                mb: <MyMobject0 as Mobject>::dynamic_refresh(
                    &mobject.mb,
                    &dynamic_keys.intrinsic.mb,
                    storage_type_map,
                    device,
                    queue,
                    format,
                    reuse,
                ),
            },
            &dynamic_keys.extrinsic,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        )
        // let input = MyMobject1 {
        //                 ma: <MyMobject0 as Mobject>::dynamic_refresh(&mobject.ma, &dynamic_keys.intrinsic.ma, storage_type_map, device, queue, format, reuse),
        //                 mb: <MyMobject0 as Mobject>::dynamic_refresh(&mobject.mb, &dynamic_keys.intrinsic.mb, storage_type_map, device, queue, format, reuse),
        //             };
        // storage_type_map.update_or_insert(
        //     &dynamic_keys.extrinsic,
        //     (input, device, queue, format),
        //     reuse,
        //     |(input, device, queue, format)| <<MyMobject1Prepare as Prepare<MyMobject1>>::Resource as Resource<
        //             <MyMobject1Prepare as Prepare<MyMobject1>>::ResourceRepr,
        //         >>::new(
        //             <MyMobject1Prepare as Prepare<MyMobject1>>::prepare(input),
        //             device,
        //             queue,
        //             format,
        //         ),
        //     |(input, device, queue, format), resource, reuse| {
        //         <<MyMobject1Prepare as Prepare<MyMobject1>>::Resource as Resource<
        //             <MyMobject1Prepare as Prepare<MyMobject1>>::ResourceRepr,
        //         >>::update(
        //             resource,
        //             <MyMobject1Prepare as Prepare<MyMobject1>>::prepare(input),
        //             device,
        //             queue,
        //             format,
        //             reuse,
        //         )
        //     },
        // )
    }

    fn static_fetch<'s>(
        static_keys: &Self::StaticKeys,
        storage_type_map: &'s StorageTypeMap,
    ) -> Self::ResourceRef<'s> {
        storage_type_map.get_and_unwrap(&static_keys.extrinsic)
    }

    fn dynamic_fetch<'s>(
        dynamic_keys: &Self::DynamicKeys,
        storage_type_map: &'s StorageTypeMap,
    ) -> Self::ResourceRef<'s> {
        storage_type_map.get_and_unwrap(&dynamic_keys.extrinsic)
    }

    // fn prepare_new(
    //     mobject: &Self,
    //     // key: &Self::StaticKey,
    //     // storage_type_map: &'s mut StorageTypeMap,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     // reuse: &mut ResourceReuseResult,
    // ) -> Self::Resource {
    //     let input = MyMobject1 {
    //         ma: <MyMobject0 as Mobject>::prepare_new(&mobject.ma, device, queue, format),
    //         mb: <MyMobject0 as Mobject>::prepare_new(&mobject.mb, device, queue, format),
    //     };
    //     <<MyMobject1Prepare as Prepare<MyMobject1>>::Resource as Resource<
    //         <MyMobject1Prepare as Prepare<MyMobject1>>::ResourceRepr,
    //     >>::new(
    //         <MyMobject1Prepare as Prepare<MyMobject1>>::prepare(&input),
    //         device,
    //         queue,
    //         format,
    //     )
    // }

    // fn prepare_update(
    //     mobject: &Self,
    //     resource: &mut Self::Resource,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     reuse: &mut ResourceReuseResult,
    // ) {
    //     <MyMobject0 as Mobject>::prepare_update(
    //         &mobject.mb,
    //         &mut resource.ma,
    //         device,
    //         queue,
    //         format,
    //         reuse,
    //     );
    //     <MyMobject0 as Mobject>::prepare_update(
    //         &mobject.mb,
    //         &mut resource.mb,
    //         device,
    //         queue,
    //         format,
    //         reuse,
    //     );
    //     <<MyMobject1Prepare as Prepare<MyMobject1>>::Resource as Resource<
    //         <MyMobject1Prepare as Prepare<MyMobject1>>::ResourceRepr,
    //     >>::update(
    //         resource,
    //         <MyMobject1Prepare as Prepare<MyMobject1>>::prepare(&resource_intrinsic),
    //         device,
    //         queue,
    //         format,
    //         reuse,
    //     )
    // }

    // fn prepare_extrinsic_new(
    //     resource_intrinsic: &Self::ResourceIntrinsic,
    //     // key: &Self::StaticKey,
    //     // storage_type_map: &'s mut StorageTypeMap,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     // reuse: &mut ResourceReuseResult,
    // ) -> Self::ResourceExtrinsic {
    //     <<MyMobject1Prepare as Prepare<MyMobject1>>::ResourceExtrinsic as Resource<
    //         <MyMobject1Prepare as Prepare<MyMobject1>>::ResourceRepr,
    //     >>::new(
    //         <MyMobject1Prepare as Prepare<MyMobject1>>::prepare(&resource_intrinsic),
    //         device,
    //         queue,
    //         format,
    //     )
    // }

    // fn prepare_extrinsic_update(
    //     resource_intrinsic: &Self::ResourceIntrinsic,
    //     resource_extrinsic: &mut Self::ResourceExtrinsic,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     reuse: &mut ResourceReuseResult,
    // ) {
    //     <<MyMobject1Prepare as Prepare<MyMobject1>>::ResourceExtrinsic as Resource<
    //         <MyMobject1Prepare as Prepare<MyMobject1>>::ResourceRepr,
    //     >>::update(
    //         resource_extrinsic,
    //         <MyMobject1Prepare as Prepare<MyMobject1>>::prepare(&resource_intrinsic),
    //         device,
    //         queue,
    //         format,
    //         reuse,
    //     )
    // }

    // fn prepare_new(
    //     mobject: &Self,
    //     // key: &Self::StaticKey,
    //     // storage_type_map: &'s mut StorageTypeMap,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     // reuse: &mut ResourceReuseResult,
    // ) -> Self::Resource {
    //     Self::prepare_extrinsic_new(&Self::prepare_intrinsic_new(mobject, device, queue, format), device, queue, format)
    //     // Derivation {
    //     //     intrinsic: resource_intrinsic,
    //     //     extrinsic: resource_extrinsic,
    //     // }
    // }

    // fn prepare_update(
    //     mobject: &Self,
    //     resource: &mut Self::Resource,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     reuse: &mut ResourceReuseResult,
    // ) {
    //     Self::prepare_intrinsic_update(mobject, &mut resource.intrinsic, device, queue, format, reuse);
    //     Self::prepare_extrinsic_update(&resource.intrinsic, &mut resource.extrinsic, device, queue, format, reuse);
    // }

    // fn prepare_new(
    //     mobject: &Self,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    // ) -> Self::Resource {
    //     let intrinsic = MyMobject1 {
    //         ma: <MyMobject0 as Mobject>::prepare_new(&mobject.ma, device, queue, format),
    //         mb: <MyMobject0 as Mobject>::prepare_new(&mobject.mb, device, queue, format),
    //     };
    //     let extrinsic = <<MyMobject1Prepare as Prepare<MyMobject1>>::ResourceExtrinsic as Resource<
    //         <MyMobject1Prepare as Prepare<MyMobject1>>::ResourceRepr,
    //     >>::new(
    //         <MyMobject1Prepare as Prepare<MyMobject1>>::prepare(&intrinsic),
    //         device,
    //         queue,
    //         format,
    //     );
    //     Derivation { intrinsic, extrinsic }
    // }

    // fn prepare_update(
    //     mobject: &Self,
    //     resource: &mut Self::Resource,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     reuse: &mut ResourceReuseResult,
    // ) {
    //     <MyMobject0 as Mobject>::prepare_update(
    //         &mobject.mb,
    //         &mut resource.intrinsic.ma,
    //         device,
    //         queue,
    //         format,
    //         reuse,
    //     );
    //     <MyMobject0 as Mobject>::prepare_update(
    //         &mobject.mb,
    //         &mut resource.intrinsic.mb,
    //         device,
    //         queue,
    //         format,
    //         reuse,
    //     );
    //     <<MyMobject1Prepare as Prepare<MyMobject1>>::ResourceExtrinsic as Resource<
    //         <MyMobject1Prepare as Prepare<MyMobject1>>::ResourceRepr,
    //     >>::update(
    //         &mut resource.extrinsic,
    //         <MyMobject1Prepare as Prepare<MyMobject1>>::prepare(&resource.intrinsic),
    //         device,
    //         queue,
    //         format,
    //         reuse,
    //     )
    // }

    fn render(resource: Self::ResourceRef<'_>, render_pass: &mut wgpu::RenderPass) {
        <MyMobject1Render as Render<MyMobject1>>::render(resource, render_pass);
    }

    // fn static_allocate(
    //     mobject: &Self,
    //     slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    // ) -> Self::StaticKey {
    //     (
    //         MyMobject1 {
    //             ma: <MyMobject0 as Mobject>::static_allocate(&mobject.ma, slot_key_generator_map),
    //             mb: <MyMobject0 as Mobject>::static_allocate(&mobject.mb, slot_key_generator_map),
    //         },
    //         slot_key_generator_map.allocate(mobject),
    //     )
    // }

    // fn dynamic_allocate(
    //     mobject: &Self,
    //     slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    // ) -> Self::DynamicKey {
    //     (
    //         MyMobject1 {
    //             ma: <MyMobject0 as Mobject>::dynamic_allocate(&mobject.ma, slot_key_generator_map),
    //             mb: <MyMobject0 as Mobject>::dynamic_allocate(&mobject.mb, slot_key_generator_map),
    //         },
    //         slot_key_generator_map.allocate(&()),
    //     )
    // }

    // fn static_prepare<'s>(
    //     mobject: &'s Self,
    //     key: &Self::StaticKey,
    //     storage_type_map: &'s mut StorageTypeMap,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     reuse: &mut ResourceReuseResult,
    // ) -> &'s Self::Resource {
    //     let input = MyMobject1 {
    //         ma: <MyMobject0 as Mobject>::static_prepare(
    //             &mobject.ma,
    //             &key.0.ma,
    //             storage_type_map,
    //             device,
    //             queue,
    //             format,
    //             reuse,
    //         ),
    //         mb: <MyMobject0 as Mobject>::static_prepare(
    //             &mobject.mb,
    //             &key.0.mb,
    //             storage_type_map,
    //             device,
    //             queue,
    //             format,
    //             reuse,
    //         ),
    //     };
    //     storage_type_map.update_or_insert(
    //         &key.1,
    //         (input, device, queue, format),
    //         reuse,
    //         |(input, device, queue, format)| {
    //             Arc::new(<MyMobject1Resource as Resource<
    //                 <MyMobject1Prepare as Prepare<MyMobject1>>::ResourceRepr,
    //             >>::new(
    //                 <MyMobject1Prepare as Prepare<MyMobject1>>::prepare(input),
    //                 device,
    //                 queue,
    //                 format,
    //             ))
    //         },
    //         |(_input, _device, _queue, _format), _resource, _reuse| {},
    //     )
    // }

    // fn dynamic_prepare<'s>(
    //     mobject: &Self,
    //     key: &Self::DynamicKey,
    //     storage_type_map: &'s mut StorageTypeMap,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     reuse: &mut ResourceReuseResult,
    // ) -> Self::ResourceRef<'s> {
    //     let input = MyMobject1 {
    //         ma: <MyMobject0 as Mobject>::dynamic_prepare(
    //             &mobject.ma,
    //             &key.0.ma,
    //             storage_type_map,
    //             device,
    //             queue,
    //             format,
    //             reuse,
    //         ),
    //         mb: <MyMobject0 as Mobject>::dynamic_prepare(
    //             &mobject.mb,
    //             &key.0.mb,
    //             storage_type_map,
    //             device,
    //             queue,
    //             format,
    //             reuse,
    //         ),
    //     };
    //     storage_type_map.update_or_insert(
    //         &key.1,
    //         (input, device, queue, format),
    //         reuse,
    //         |(input, device, queue, format)| {
    //             <MyMobject1Resource as Resource<
    //                 <MyMobject1Prepare as Prepare<MyMobject1>>::ResourceRepr,
    //             >>::new(
    //                 <MyMobject1Prepare as Prepare<MyMobject1>>::prepare(input),
    //                 device,
    //                 queue,
    //                 format,
    //             )
    //         },
    //         |(input, device, queue, format), resource, reuse| {
    //             <MyMobject1Resource as Resource<
    //                 <MyMobject1Prepare as Prepare<MyMobject1>>::ResourceRepr,
    //             >>::update(
    //                 resource,
    //                 <MyMobject1Prepare as Prepare<MyMobject1>>::prepare(input),
    //                 device,
    //                 queue,
    //                 format,
    //                 reuse,
    //             );
    //         },
    //     )
    // }

    // fn render(resource: Self::ResourceRef<'_>, render_pass: &mut wgpu::RenderPass) {
    //     <MyMobject1Render as Render<MyMobject1>>::render(resource, render_pass);
    // }
}

impl<MA, MB> Variant<MyMobject1> for MyMobject1<MA, MB>
where
    MA: Variant<MyMobject0>,
    MB: Variant<MyMobject0>,
{
    type Observe = MyMobject1<MA::Observe, MB::Observe>;
    type Keys = Derivation<
        MyMobject1<MA::Keys, MB::Keys>,
        StorageKey<
            DynamicStoreType<MyMobject1, <MyMobject1Prepare as Prepare<MyMobject1>>::Resource>,
        >,
    >;
    // type ResourceRef<'s> = &'s <MyMobject1Prepare as Prepare<MyMobject1>>::Resource;

    fn allocate(
        // timeline: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        Derivation {
            intrinsic: MyMobject1 {
                ma: MA::allocate(&observe.ma, slot_key_generator_map),
                mb: MB::allocate(&observe.mb, slot_key_generator_map),
            },
            extrinsic: slot_key_generator_map.allocate(&()),
        }
    }

    fn prepare<'s>(
        observe: &Self::Observe,
        keys: &Self::Keys,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> <MyMobject1 as Mobject>::ResourceRef<'s> {
        // let input = MyMobject1 {
        //     ma: MA::prepare(
        //         &observe.ma,
        //         &keys.intrinsic.ma,
        //         storage_type_map,
        //         device,
        //         queue,
        //         format,
        //         reuse,
        //     ),
        //     mb: MB::prepare(
        //         &observe.mb,
        //         &keys.intrinsic.mb,
        //         storage_type_map,
        //         device,
        //         queue,
        //         format,
        //         reuse,
        //     ),
        // };
        // storage_type_map.update_or_insert(
        //     &keys.extrinsic,
        //     (input, device, queue, format),
        //     reuse,
        //     |(input, device, queue, format)| <<MyMobject1Prepare as Prepare<MyMobject1>>::Resource as Resource<
        //             <MyMobject1Prepare as Prepare<MyMobject1>>::ResourceRepr,
        //         >>::new(
        //             <MyMobject1Prepare as Prepare<MyMobject1>>::prepare(input),
        //             device,
        //             queue,
        //             format,
        //         ),
        //     |(input, device, queue, format), resource, reuse| {
        //         <<MyMobject1Prepare as Prepare<MyMobject1>>::Resource as Resource<
        //             <MyMobject1Prepare as Prepare<MyMobject1>>::ResourceRepr,
        //         >>::update(
        //             resource,
        //             <MyMobject1Prepare as Prepare<MyMobject1>>::prepare(input),
        //             device,
        //             queue,
        //             format,
        //             reuse,
        //         )
        //     },
        // )

        <MyMobject1Prepare as Prepare<MyMobject1>>::dynamic_prepare(
            MyMobject1 {
                ma: MA::prepare(
                    &observe.ma,
                    &keys.intrinsic.ma,
                    storage_type_map,
                    device,
                    queue,
                    format,
                    reuse,
                ),
                mb: MB::prepare(
                    &observe.mb,
                    &keys.intrinsic.mb,
                    storage_type_map,
                    device,
                    queue,
                    format,
                    reuse,
                ),
            },
            &keys.extrinsic,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        )
        // storage_type_map.update_or_insert(
        //     &keys.extrinsic,
        //     (resource_intrinsic, device, queue, format),
        //     reuse,
        //     |(resource_intrinsic, device, queue, format)| <MyMobject1 as Mobject>::prepare_extrinsic_new(resource_intrinsic, device, queue, format),
        //     |(resource_intrinsic, device, queue, format), resource, reuse| {
        //         <MyMobject1 as Mobject>::prepare_extrinsic_update(resource_intrinsic, resource, device, queue, format, reuse)
        //     },
        // )
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        // MA::render(&keys.intrinsic.ma, storage_type_map, render_pass);
        // MB::render(&keys.intrinsic.mb, storage_type_map, render_pass);
        // TODO
        <MyMobject1 as Mobject>::render(
            storage_type_map.get_and_unwrap(&keys.extrinsic),
            render_pass,
        );
    }
}

impl<MA, MB> Timeline<MyMobject1> for MyMobject1<MA, MB>
where
    MA: Timeline<MyMobject0>,
    MB: Timeline<MyMobject0>,
{
    type Variant = MyMobject1<MA::Variant, MB::Variant>;

    fn observe(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &<Self::Variant as Variant<MyMobject1>>::Observe,
    ) -> <Self::Variant as Variant<MyMobject1>>::Observe {
        MyMobject1 {
            ma: MA::observe(clock, clock_span, &timeline.ma, &observe.ma),
            mb: MB::observe(clock, clock_span, &timeline.mb, &observe.mb),
        }
    }
}
