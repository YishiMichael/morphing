use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use super::storable::ArcSlot;
use super::storable::ResourceReuseResult;
use super::storable::SlotKeyGeneratorTypeMap;
use super::storable::StorageKey;
use super::storable::StorageTypeMap;
use super::storable::StoreType;
use super::storable::SwapSlot;
use super::storable::VecSlot;
use super::timer::Clock;
use super::timer::ClockSpan;

pub trait Mobject:
    'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
{
    // type StaticKey;
    // type DynamicKey;
    // type ResourceInput;
    type ResourceIntrinsic;
    type ResourceExtrinsic;
    type Resource: 'static + Send + Sync;
    // type ResourceRefInput<'s>;
    // type ResourceRef<'s>;

    // fn static_allocate(
    //     mobject: &Self,
    //     slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    // ) -> Self::StaticKey;
    // fn dynamic_allocate(
    //     mobject: &Self,
    //     slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    // ) -> Self::DynamicKey;
    fn prepare_intrinsic_new(
        mobject: &Self,
        // key: &Self::StaticKey,
        // storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        // reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceIntrinsic;
    fn prepare_intrinsic_update(
        mobject: &Self,
        resource_intrinsic: &mut Self::ResourceIntrinsic,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    );
    fn prepare_extrinsic_new(
        resource_intrinsic: &Self::ResourceIntrinsic,
        // key: &Self::StaticKey,
        // storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        // reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceExtrinsic;
    fn prepare_extrinsic_update(
        resource_intrinsic: &Self::ResourceIntrinsic,
        resource_extrinsic: &mut Self::ResourceExtrinsic,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    );
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
    fn render(resource: &Self::Resource, render_pass: &mut wgpu::RenderPass);
}

pub trait Variant<M> where M: Mobject {
    type Observe: Send + Sync;
    type Key;
    type ResourceRef<'s>;

    fn allocate(
        // timeline: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key;
    fn prepare<'s>(
        observe: &Self::Observe,
        key: &Self::Key,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s>;
    fn render(
        key: &Self::Key,
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

pub trait Resource<RR> {
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

pub trait Prepare<M>
where
    M: Mobject,
{
    type ResourceRepr;
    type ResourceExtrinsic: Resource<Self::ResourceRepr>;

    fn prepare(resource_intrinsic: &M::ResourceIntrinsic) -> Self::ResourceRepr;
}

pub trait Render<M>
where
    M: Mobject,
{
    fn render(resource: &M::Resource, render_pass: &mut wgpu::RenderPass);
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

pub struct StaticStoreType<R>(R);

impl<R> StoreType for StaticStoreType<R>
where
    R: 'static + Send + Sync,
{
    // type KeyInput = M;
    type Slot = SwapSlot<ArcSlot<R>>;
}

// impl<M> Variant<M> for StaticVariant<M>
// where
//     M: Mobject,
// {
//     type Observe = Arc<M>;
// }

pub struct DynamicStoreType<R>(R);

impl<R> StoreType for DynamicStoreType<R>
where
    R: 'static + Send + Sync,
{
    // type KeyInput = ();
    type Slot = SwapSlot<VecSlot<R>>;
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
    type Key = StorageKey<StaticStoreType<M::Resource>>;
    type ResourceRef<'s> = &'s M::Resource;

    fn allocate(
        // _timeline: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key {
        slot_key_generator_map.allocate(observe.as_ref())
        // M::static_allocate(observe.as_ref(), slot_key_generator_map)
    }

    fn prepare<'s>(
        observe: &Self::Observe,
        key: &Self::Key,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        // let mobject = observe.as_ref();
        storage_type_map.update_or_insert(
            key,
            (observe, device, queue, format),
            reuse,
            |(observe, device, queue, format)| {
                Arc::new(M::prepare_new(observe.as_ref(), device, queue, format))
            },
            |(_observe, _device, _queue, _format), _resource, _reuse| {},
        )
        // M::static_prepare(
        //     observe.as_ref(),
        //     key,
        //     storage_type_map,
        //     device,
        //     queue,
        //     format,
        //     reuse,
        // );
    }

    fn render(
        key: &Self::Key,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        M::render(storage_type_map.get_and_unwrap(key), render_pass);
        // M::static_render(key, storage_type_map, render_pass);
    }
}

pub struct StaticTimeline;

impl<M> Timeline<M> for StaticTimeline where M: Mobject {
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
    type Key = StorageKey<DynamicStoreType<M::Resource>>;
    type ResourceRef<'s> = &'s M::Resource;

    fn allocate(
        // _timeline: &Self,
        _observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key {
        slot_key_generator_map.allocate(&())
        // M::dynamic_allocate(observe, slot_key_generator_map)
    }

    fn prepare<'s>(
        observe: &Self::Observe,
        key: &Self::Key,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        storage_type_map.update_or_insert(
            key,
            (observe, device, queue, format),
            reuse,
            |(observe, device, queue, format)| M::prepare_new(observe, device, queue, format),
            |(observe, device, queue, format), resource, reuse| {
                M::prepare_update(observe, resource, device, queue, format, reuse)
            },
        )
        // M::dynamic_prepare(
        //     &timeline.refresh.refresh(clock, clock_span, observe.clone()),
        //     key,
        //     storage_type_map,
        //     device,
        //     queue,
        //     format,
        //     reuse,
        // );
    }

    fn render(
        key: &Self::Key,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        M::render(storage_type_map.get_and_unwrap(key), render_pass);
        // M::dynamic_render(key, storage_type_map, render_pass);
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
    // type StaticKey = ();
    // type DynamicKey = ();
    type ResourceIntrinsic = T;
    type ResourceExtrinsic = ();
    type Resource = Self::ResourceIntrinsic;
    // type ResourceRef<'s> = &'s T;

    fn prepare_intrinsic_new(
        mobject: &Self,
        // key: &Self::StaticKey,
        // storage_type_map: &'s mut StorageTypeMap,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        // reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceIntrinsic {
        (&**mobject).clone()
    }

    fn prepare_intrinsic_update(
        mobject: &Self,
        resource_intrinsic: &mut Self::ResourceIntrinsic,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        _reuse: &mut ResourceReuseResult,
    ) {
        *resource_intrinsic = (&**mobject).clone();
    }

    fn prepare_extrinsic_new(
        _resource_intrinsic: &Self::ResourceIntrinsic,
        // key: &Self::StaticKey,
        // storage_type_map: &'s mut StorageTypeMap,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        // reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceExtrinsic {
        ()
    }

    fn prepare_extrinsic_update(
        _resource_intrinsic: &Self::ResourceIntrinsic,
        _resource_extrinsic: &mut Self::ResourceExtrinsic,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        _reuse: &mut ResourceReuseResult,
    ) {}


    fn prepare_new(
        mobject: &Self,
        // key: &Self::StaticKey,
        // storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        // reuse: &mut ResourceReuseResult,
    ) -> Self::Resource {
        Self::prepare_intrinsic_new(mobject, device, queue, format)
    }

    fn prepare_update(
        mobject: &Self,
        resource: &mut Self::Resource,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        Self::prepare_intrinsic_update(mobject, resource, device, queue, format, reuse);
    }

    fn render(_resource: &Self::Resource, _render_pass: &mut wgpu::RenderPass) {}

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
    // type StaticKey =
    //     MyMobject0<<Data<f32> as Mobject>::StaticKey, <Data<f32> as Mobject>::StaticKey>;
    // type DynamicKey =
    //     MyMobject0<<Data<f32> as Mobject>::DynamicKey, <Data<f32> as Mobject>::DynamicKey>;
    type ResourceIntrinsic =
        MyMobject0<<Data<f32> as Mobject>::Resource, <Data<f32> as Mobject>::Resource>;
    type ResourceExtrinsic = ();
    type Resource = Self::ResourceIntrinsic;
    // type ResourceRef<'s> = MyMobject0<
    //     <Data<f32> as Mobject>::ResourceRef<'s>,
    //     <Data<f32> as Mobject>::ResourceRef<'s>,
    // >;

    fn prepare_intrinsic_new(
        mobject: &Self,
        // key: &Self::StaticKey,
        // storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        // reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceIntrinsic {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::prepare_new(&mobject.ma, device, queue, format),
            mb: <Data<f32> as Mobject>::prepare_new(&mobject.mb, device, queue, format),
        }
    }

    fn prepare_intrinsic_update(
        mobject: &Self,
        resource_intrinsic: &mut Self::ResourceIntrinsic,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <Data<f32> as Mobject>::prepare_update(
            &mobject.ma,
            &mut resource_intrinsic.ma,
            device,
            queue,
            format,
            reuse,
        );
        <Data<f32> as Mobject>::prepare_update(
            &mobject.mb,
            &mut resource_intrinsic.mb,
            device,
            queue,
            format,
            reuse,
        );
    }

    fn prepare_extrinsic_new(
        _resource_intrinsic: &Self::ResourceIntrinsic,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> Self::ResourceExtrinsic {
        ()
    }

    fn prepare_extrinsic_update(
        _resource_intrinsic: &Self::ResourceIntrinsic,
        _resource_extrinsic: &mut Self::ResourceExtrinsic,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        _reuse: &mut ResourceReuseResult,
    ) {}

    fn prepare_new(
        mobject: &Self,
        // key: &Self::StaticKey,
        // storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        // reuse: &mut ResourceReuseResult,
    ) -> Self::Resource {
        Self::prepare_intrinsic_new(mobject, device, queue, format)
    }

    fn prepare_update(
        mobject: &Self,
        resource: &mut Self::Resource,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        Self::prepare_intrinsic_update(mobject, resource, device, queue, format, reuse);
    }

    fn render(resource: &Self::Resource, render_pass: &mut wgpu::RenderPass) {
        <Data<f32> as Mobject>::render(&resource.ma, render_pass);
        <Data<f32> as Mobject>::render(&resource.mb, render_pass);
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
    type Key = MyMobject0<MA::Key, MB::Key>;
    type ResourceRef<'s> = MyMobject0<MA::ResourceRef<'s>, MB::ResourceRef<'s>>;

    fn allocate(
        // timeline: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key {
        MyMobject0 {
            ma: MA::allocate(&observe.ma, slot_key_generator_map),
            mb: MB::allocate(&observe.mb, slot_key_generator_map),
        }
    }

    fn prepare<'s>(
        observe: &Self::Observe,
        key: &Self::Key,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        MyMobject0 {
            ma: MA::prepare(
                &observe.ma,
                &key.ma,
                storage_type_map,
                device,
                queue,
                format,
                reuse,
            ),
            mb: MB::prepare(
                &observe.mb,
                &key.mb,
                storage_type_map,
                device,
                queue,
                format,
                reuse,
            ),
        }
    }

    fn render(
        key: &Self::Key,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        MA::render(&key.ma, storage_type_map, render_pass);
        MB::render(&key.mb, storage_type_map, render_pass);
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
    type ResourceExtrinsic = MyMobject1Resource;

    fn prepare(
        resource_intrinsic: &<MyMobject1 as Mobject>::ResourceIntrinsic,
    ) -> Self::ResourceRepr {
        [
            resource_intrinsic.ma.ma,
            resource_intrinsic.ma.mb,
            resource_intrinsic.mb.ma,
            resource_intrinsic.mb.mb,
        ]
    }
}

pub struct MyMobject1Render;

impl Render<MyMobject1> for MyMobject1Render {
    fn render(_resource: &<MyMobject1 as Mobject>::Resource, _render_pass: &mut wgpu::RenderPass) {
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
    // type StaticKey = (
    //     MyMobject1<<MyMobject0 as Mobject>::StaticKey, <MyMobject0 as Mobject>::StaticKey>,
    //     StorageKey<StaticVariant<MyMobject1>>,
    // );
    // type DynamicKey = (
    //     MyMobject1<<MyMobject0 as Mobject>::StaticKey, <MyMobject0 as Mobject>::StaticKey>,
    //     StorageKey<DynamicVariant<MyMobject1>>,
    // );
    type ResourceIntrinsic =
        MyMobject1<<MyMobject0 as Mobject>::Resource, <MyMobject0 as Mobject>::Resource>;
    type ResourceExtrinsic = <MyMobject1Prepare as Prepare<MyMobject1>>::ResourceExtrinsic;
    type Resource = Self::ResourceExtrinsic;
    // type ResourceRefInput<'s> =
    //     MyMobject1<&'s <MyMobject0 as Mobject>::Resource, &'s <MyMobject0 as Mobject>::Resource>;
    // type ResourceRef<'s> = &'s <MyMobject1Prepare as Prepare<MyMobject1>>::Resource;

    fn prepare_intrinsic_new(
        mobject: &Self,
        // key: &Self::StaticKey,
        // storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        // reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceIntrinsic {
        MyMobject1 {
            ma: <MyMobject0 as Mobject>::prepare_new(&mobject.ma, device, queue, format),
            mb: <MyMobject0 as Mobject>::prepare_new(&mobject.mb, device, queue, format),
        }
    }

    fn prepare_intrinsic_update(
        mobject: &Self,
        resource_intrinsic: &mut Self::ResourceIntrinsic,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <MyMobject0 as Mobject>::prepare_update(
            &mobject.mb,
            &mut resource_intrinsic.ma,
            device,
            queue,
            format,
            reuse,
        );
        <MyMobject0 as Mobject>::prepare_update(
            &mobject.mb,
            &mut resource_intrinsic.mb,
            device,
            queue,
            format,
            reuse,
        );
    }

    fn prepare_extrinsic_new(
        resource_intrinsic: &Self::ResourceIntrinsic,
        // key: &Self::StaticKey,
        // storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        // reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceExtrinsic {
        <<MyMobject1Prepare as Prepare<MyMobject1>>::ResourceExtrinsic as Resource<
            <MyMobject1Prepare as Prepare<MyMobject1>>::ResourceRepr,
        >>::new(
            <MyMobject1Prepare as Prepare<MyMobject1>>::prepare(&resource_intrinsic),
            device,
            queue,
            format,
        )
    }

    fn prepare_extrinsic_update(
        resource_intrinsic: &Self::ResourceIntrinsic,
        resource_extrinsic: &mut Self::ResourceExtrinsic,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <<MyMobject1Prepare as Prepare<MyMobject1>>::ResourceExtrinsic as Resource<
            <MyMobject1Prepare as Prepare<MyMobject1>>::ResourceRepr,
        >>::update(
            resource_extrinsic,
            <MyMobject1Prepare as Prepare<MyMobject1>>::prepare(&resource_intrinsic),
            device,
            queue,
            format,
            reuse,
        )
    }

    fn prepare_new(
        mobject: &Self,
        // key: &Self::StaticKey,
        // storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        // reuse: &mut ResourceReuseResult,
    ) -> Self::Resource {
        Self::prepare_extrinsic_new(&Self::prepare_intrinsic_new(mobject, device, queue, format), device, queue, format)
        // Derivation {
        //     intrinsic: resource_intrinsic,
        //     extrinsic: resource_extrinsic,
        // }
    }

    fn prepare_update(
        mobject: &Self,
        resource: &mut Self::Resource,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        Self::prepare_intrinsic_update(mobject, &mut resource.intrinsic, device, queue, format, reuse);
        Self::prepare_extrinsic_update(&resource.intrinsic, &mut resource.extrinsic, device, queue, format, reuse);
    }

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

    fn render(resource: &Self::Resource, render_pass: &mut wgpu::RenderPass) {
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
    type Key = Derivation<
        MyMobject1<MA::Key, MB::Key>,
        StorageKey<DynamicStoreType<<MyMobject1 as Mobject>::ResourceExtrinsic>>,
    >;
    type ResourceRef<'s> = Derivation<MyMobject1<MA::ResourceRef<'s>, MB::ResourceRef<'s>>, &'s <MyMobject1 as Mobject>::ResourceExtrinsic>;

    fn allocate(
        // timeline: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key {
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
        key: &Self::Key,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        let resource_intrinsic = MyMobject1 {
            ma: MA::prepare(
                &observe.ma,
                &key.intrinsic.ma,
                storage_type_map,
                device,
                queue,
                format,
                reuse,
            ),
            mb: MB::prepare(
                &observe.mb,
                &key.intrinsic.mb,
                storage_type_map,
                device,
                queue,
                format,
                reuse,
            ),
        };
        let resource_extrinsic = storage_type_map.update_or_insert(
            &key.extrinsic,
            (resource_intrinsic, device, queue, format),
            reuse,
            |(resource_intrinsic, device, queue, format)| <MyMobject1 as Mobject>::prepare_extrinsic_new(resource_intrinsic, device, queue, format),
            |(resource_intrinsic, device, queue, format), resource, reuse| {
                <MyMobject1 as Mobject>::prepare_extrinsic_update(resource_intrinsic, resource, device, queue, format, reuse)
            },
        );
        Derivation {
            intrinsic: resource_intrinsic,
            extrinsic: resource_extrinsic,
        }
    }

    fn render(
        key: &Self::Key,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        MA::render(&key.intrinsic.ma, storage_type_map, render_pass);
        MB::render(&key.intrinsic.mb, storage_type_map, render_pass);
        // TODO
        <MyMobject1 as Mobject>::render(Derivation {
            intrinsic: 
            extrinsic: storage_type_map.get_and_unwrap(&key.extrinsic),
        }, render_pass);
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
