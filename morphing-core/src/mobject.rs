use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use super::storage::MultitonSlot;
use super::storage::ResourceReuseResult;
use super::storage::SingletonSlot;
use super::storage::Slot;
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
    type MobjectRef<'m>: serde::Serialize;
    type ResourceRef<'s>;
    type ResourceRefInput<'s>;

    fn mobject_ref<'m>(mobject: &'m Self) -> Self::MobjectRef<'m>;
}

pub trait VariantFields<M>
where
    M: Mobject,
{
    // type Observe;
    // type ObserveRef<'m>;
    type Keys;

    fn allocate(
        mobject_ref: M::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys;
    fn get<'s>(keys: &Self::Keys, storage_type_map: &'s StorageTypeMap) -> M::ResourceRefInput<'s>;
    fn prepare(
        mobject_ref: M::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    );
    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    );
}

pub trait Variant<M>
where
    M: Mobject,
{
    // type Observe;
    // type ObserveRef<'m>;
    // type MobjectRef<'m>;
    type Keys;

    // fn mobject_ref<'m>(observe_ref: Self::ObserveRef<'m>) -> M::MobjectRef<'m>;
    fn allocate(
        mobject_ref: M::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys;
    fn get<'s>(keys: &Self::Keys, storage_type_map: &'s StorageTypeMap) -> M::ResourceRef<'s>;
    fn prepare(
        mobject_ref: M::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    );
    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    );
}

pub trait Timeline<M>
where
    M: Mobject,
{
    type Observe;
    type Variant: Variant<M>;

    fn observe(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &Self::Observe,
    ) -> Self::Observe;
    fn mobject_ref<'m>(observe: &'m Self::Observe) -> M::MobjectRef<'m>;
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

pub trait Prepare: Mobject {
    type ResourceRepr;
    type Resource: Resource<Self::ResourceRepr>;

    fn prepare(input: &Self::ResourceRefInput<'_>) -> Self::ResourceRepr;
}

pub trait Render: Mobject {
    fn render(resource: &Self::ResourceRef<'_>, render_pass: &mut wgpu::RenderPass);
}

pub struct Derivation<E, I> {
    pub extrinsic: E,
    pub intrinsic: I,
}

pub struct StaticStoreType<M>(M);

impl<M> StoreType for StaticStoreType<M>
where
    M: Prepare,
{
    type Slot = SwapSlot<SingletonSlot<M::Resource>>;
    type KeyInput<'s> = M::MobjectRef<'s>;
    type Input<'s> = (
        M::ResourceRepr,
        &'s wgpu::Device,
        &'s wgpu::Queue,
        wgpu::TextureFormat,
    );
    type Output<'s> = &'s M::Resource;

    fn insert(input: Self::Input<'_>) -> <Self::Slot as Slot>::Value {
        let (resource_repr, device, queue, format) = input;
        <M::Resource as Resource<M::ResourceRepr>>::new(resource_repr, device, queue, format)
    }

    fn update(
        _input: Self::Input<'_>,
        _value: &mut <Self::Slot as Slot>::Value,
        _reuse: &mut ResourceReuseResult,
    ) {
    }

    fn fetch(value: &<Self::Slot as Slot>::Value) -> Self::Output<'_> {
        value
    }
}

pub struct DynamicStoreType<M>(M);

impl<M> StoreType for DynamicStoreType<M>
where
    M: Prepare,
{
    // type KeyInput = ();
    type Slot = SwapSlot<MultitonSlot<M::Resource>>;
    type KeyInput<'s> = ();
    type Input<'s> = (
        M::ResourceRepr,
        &'s wgpu::Device,
        &'s wgpu::Queue,
        wgpu::TextureFormat,
    );
    type Output<'s> = &'s M::Resource;

    fn insert(input: Self::Input<'_>) -> <Self::Slot as Slot>::Value {
        let (resource_repr, device, queue, format) = input;
        <M::Resource as Resource<M::ResourceRepr>>::new(resource_repr, device, queue, format)
    }

    fn update(
        input: Self::Input<'_>,
        value: &mut <Self::Slot as Slot>::Value,
        reuse: &mut ResourceReuseResult,
    ) {
        let (resource_repr, device, queue, format) = input;
        <M::Resource as Resource<M::ResourceRepr>>::update(
            value,
            resource_repr,
            device,
            queue,
            format,
            reuse,
        )
    }

    fn fetch(value: &<Self::Slot as Slot>::Value) -> Self::Output<'_> {
        value
    }
}

pub trait Refresh<M>: 'static + Send + Sync
where
    M: Mobject,
{
    fn refresh(&self, clock: Clock, clock_span: ClockSpan, mobject: &mut M);
}

pub struct StaticVariant;

pub struct DynamicVariant;

pub struct StaticTimeline;

pub struct DynamicTimeline<R> {
    refresh: R,
}

// data

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

impl<T> Prepare for Data<T>
where
    T: 'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    type ResourceRepr = T;
    type Resource = T;

    fn prepare(input: &<Self as Mobject>::ResourceRefInput<'_>) -> Self::ResourceRepr {
        (*input).clone()
    }
}

impl<T> Mobject for Data<T>
where
    T: 'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    type MobjectRef<'m> = &'m T;
    type ResourceRef<'s> = &'s T;
    type ResourceRefInput<'s> = &'s T;

    fn mobject_ref<'m>(mobject: &'m Self) -> Self::MobjectRef<'m> {
        mobject
    }
}

// impl<T> Variant<Data<T>> for StaticDerivationVariant
// where
//     T: 'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
// {
//     type Target = Data<T>;
//     type Observe = Data<T>;
//     type Keys = StorageKey<StaticStoreType<Data<T>>>;

//     fn target(observe: &Self::Observe) -> &Self::Target {
//         observe
//     }

//     fn allocate(
//         target: &Self::Target,
//         slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> Self::Keys {
//         slot_key_generator_map.allocate(target)
//     }

//     fn get<'s>(
//         keys: &Self::Keys,
//         storage_type_map: &'s StorageTypeMap,
//     ) -> <Data<T> as Mobject>::ResourceRef<'s> {
//         storage_type_map.read(keys)
//     }

//     fn prepare(
//         target: &Self::Target,
//         keys: &Self::Keys,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         reuse: &mut ResourceReuseResult,
//     ) {
//         storage_type_map.write(
//             keys,
//             reuse,
//             (
//                 <Data<T> as Prepare>::prepare(&&**target),
//                 device,
//                 queue,
//                 format,
//             ),
//         );
//     }

//     fn render(
//         _keys: &Self::Keys,
//         _storage_type_map: &StorageTypeMap,
//         _render_pass: &mut wgpu::RenderPass,
//     ) {
//     }
// }

// impl<T> Variant<Data<T>> for DynamicDerivationVariant
// where
//     T: 'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
// {
//     type Target = Data<T>;
//     type Observe = Data<T>;
//     type Keys = StorageKey<DynamicStoreType<Data<T>>>;

//     fn target(observe: &Self::Observe) -> &Self::Target {
//         observe
//     }

//     fn allocate(
//         _target: &Self::Target,
//         slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> Self::Keys {
//         slot_key_generator_map.allocate(&())
//     }

//     fn get<'s>(
//         keys: &Self::Keys,
//         storage_type_map: &'s StorageTypeMap,
//     ) -> <Data<T> as Mobject>::ResourceRef<'s> {
//         storage_type_map.read(keys)
//     }

//     fn prepare(
//         target: &Self::Target,
//         keys: &Self::Keys,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         reuse: &mut ResourceReuseResult,
//     ) {
//         storage_type_map.write(
//             keys,
//             reuse,
//             (
//                 <Data<T> as Prepare>::prepare(&&**target),
//                 device,
//                 queue,
//                 format,
//             ),
//         );
//     }

//     fn render(
//         _keys: &Self::Keys,
//         _storage_type_map: &StorageTypeMap,
//         _render_pass: &mut wgpu::RenderPass,
//     ) {
//     }
// }

impl<T> Variant<Data<T>> for StaticVariant
where
    T: 'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    // type Observe = Arc<Data<T>>;
    // type ObserveRef<'m> = &'m Data<T>;
    type Keys = StorageKey<StaticStoreType<Data<T>>>;

    // fn mobject_ref<'m>(observe_ref: Self::ObserveRef<'m>) -> <Data<T> as Mobject>::MobjectRef<'m> {
    //     observe_ref
    // }

    fn allocate(
        mobject_ref: <Data<T> as Mobject>::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        slot_key_generator_map.allocate(&mobject_ref)
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <Data<T> as Mobject>::ResourceRef<'s> {
        storage_type_map.read(keys)
    }

    fn prepare(
        mobject_ref: <Data<T> as Mobject>::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        storage_type_map.write(
            keys,
            reuse,
            (
                <Data<T> as Prepare>::prepare(&mobject_ref),
                device,
                queue,
                format,
            ),
        );
    }

    fn render(
        _keys: &Self::Keys,
        _storage_type_map: &StorageTypeMap,
        _render_pass: &mut wgpu::RenderPass,
    ) {
    }
}

impl<T> Variant<Data<T>> for DynamicVariant
where
    T: 'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    // type Observe = Data<T>;
    // type ObserveRef<'m> = &'m Data<T>;
    type Keys = StorageKey<DynamicStoreType<Data<T>>>;

    // fn mobject_ref<'m>(observe_ref: Self::ObserveRef<'m>) -> <Data<T> as Mobject>::MobjectRef<'m> {
    //     observe_ref
    // }

    fn allocate(
        _mobject_ref: <Data<T> as Mobject>::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        slot_key_generator_map.allocate(&())
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <Data<T> as Mobject>::ResourceRef<'s> {
        storage_type_map.read(keys)
    }

    fn prepare(
        mobject_ref: <Data<T> as Mobject>::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        storage_type_map.write(
            keys,
            reuse,
            (
                <Data<T> as Prepare>::prepare(&mobject_ref),
                device,
                queue,
                format,
            ),
        );
    }

    fn render(
        _keys: &Self::Keys,
        _storage_type_map: &StorageTypeMap,
        _render_pass: &mut wgpu::RenderPass,
    ) {
    }
}

impl<T> Timeline<Data<T>> for StaticTimeline
where
    T: 'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    type Observe = Arc<Data<T>>;
    type Variant = StaticVariant;

    fn observe(
        _clock: Clock,
        _clock_span: ClockSpan,
        _timeline: &Self,
        observe: &Self::Observe,
    ) -> Self::Observe {
        observe.clone()
    }

    fn mobject_ref<'m>(observe: &'m Self::Observe) -> <Data<T> as Mobject>::MobjectRef<'m> {
        observe.as_ref()
    }
}

impl<T, R> Timeline<Data<T>> for DynamicTimeline<R>
where
    T: 'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
    R: Refresh<Data<T>>,
{
    type Observe = Data<T>;
    type Variant = DynamicVariant;

    fn observe(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &Self::Observe,
    ) -> Self::Observe {
        let mut observe = observe.clone();
        timeline.refresh.refresh(clock, clock_span, &mut observe);
        observe
    }

    fn mobject_ref<'m>(observe: &'m Self::Observe) -> <Data<T> as Mobject>::MobjectRef<'m> {
        observe
    }
}

// group

/*
impl<M> Mobject for Vec<M>
where
    M: Mobject,
{
    type ResourceRef<'s> = Vec<M::ResourceRef<'s>>;
    type ResourceRefInput<'s> = Vec<M::ResourceRef<'s>>;
}

impl<M> Variant<Vec<M>> for StaticDerivationVariant
where
    M: Mobject,
    StaticDerivationVariant: Variant<M, Observe = M>,
{
    type Observe = Vec<M>;
    type Keys = Derivation<Vec<<StaticDerivationVariant as Variant<M>>::Keys>, ()>;

    fn allocate(
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        Derivation {
            intrinsic: observe
                .iter()
                .map(|mobject| {
                    <StaticDerivationVariant as Variant<M>>::allocate(
                        mobject,
                        slot_key_generator_map,
                    )
                })
                .collect(),
            extrinsic: (),
        }
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <Vec<M> as Mobject>::ResourceRef<'s> {
        keys.intrinsic
            .iter()
            .map(|key| <StaticDerivationVariant as Variant<M>>::get(key, storage_type_map))
            .collect()
    }

    fn prepare(
        observe: &Self::Observe,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        observe
            .iter()
            .zip(&keys.intrinsic)
            .for_each(|(mobject, key)| {
                <StaticDerivationVariant as Variant<M>>::prepare(
                    mobject,
                    key,
                    storage_type_map,
                    device,
                    queue,
                    format,
                    reuse,
                )
            });
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        keys.intrinsic.iter().for_each(|key| {
            <StaticDerivationVariant as Variant<M>>::render(key, storage_type_map, render_pass)
        });
    }
}

impl<M> Variant<Vec<M>> for DynamicDerivationVariant
where
    M: Mobject,
    DynamicDerivationVariant: Variant<M, Observe = M>,
{
    type Observe = Vec<M>;
    type Keys = Derivation<Vec<<DynamicDerivationVariant as Variant<M>>::Keys>, ()>;

    fn allocate(
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        Derivation {
            intrinsic: observe
                .iter()
                .map(|mobject| {
                    <DynamicDerivationVariant as Variant<M>>::allocate(
                        mobject,
                        slot_key_generator_map,
                    )
                })
                .collect(),
            extrinsic: (),
        }
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <Vec<M> as Mobject>::ResourceRef<'s> {
        keys.intrinsic
            .iter()
            .map(|key| <DynamicDerivationVariant as Variant<M>>::get(key, storage_type_map))
            .collect()
    }

    fn prepare(
        observe: &Self::Observe,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        observe
            .iter()
            .zip(&keys.intrinsic)
            .for_each(|(mobject, key)| {
                <DynamicDerivationVariant as Variant<M>>::prepare(
                    mobject,
                    key,
                    storage_type_map,
                    device,
                    queue,
                    format,
                    reuse,
                )
            });
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        keys.intrinsic.iter().for_each(|key| {
            <DynamicDerivationVariant as Variant<M>>::render(key, storage_type_map, render_pass)
        });
    }
}

impl<M> Variant<Vec<M>> for StaticVariant where M: Mobject,  {
    type Observe = Arc<Vec<M>>;
    type Keys = Vec<<StaticDerivationVariant as Variant<M>>::Keys>;

    fn allocate(
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        <StaticDerivationVariant as Variant<MyMobject0>>::allocate(
            observe.as_ref(),
            slot_key_generator_map,
        )
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <MyMobject0 as Mobject>::ResourceRef<'s> {
        <StaticDerivationVariant as Variant<MyMobject0>>::get(keys, storage_type_map)
    }

    fn prepare(
        observe: &Self::Observe,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <StaticDerivationVariant as Variant<MyMobject0>>::prepare(
            observe.as_ref(),
            keys,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        <StaticDerivationVariant as Variant<MyMobject0>>::render(
            keys,
            storage_type_map,
            render_pass,
        );
    }
}

impl Variant<MyMobject0> for DynamicVariant {
    type Observe = MyMobject0;
    type Keys = <DynamicDerivationVariant as Variant<MyMobject0>>::Keys;

    fn allocate(
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        <DynamicDerivationVariant as Variant<MyMobject0>>::allocate(observe, slot_key_generator_map)
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <MyMobject0 as Mobject>::ResourceRef<'s> {
        <DynamicDerivationVariant as Variant<MyMobject0>>::get(keys, storage_type_map)
    }

    fn prepare(
        observe: &Self::Observe,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <DynamicDerivationVariant as Variant<MyMobject0>>::prepare(
            observe,
            keys,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        <DynamicDerivationVariant as Variant<MyMobject0>>::render(
            keys,
            storage_type_map,
            render_pass,
        );
    }
}

impl<MA, MB> Variant<MyMobject0> for MyMobject0<MA, MB>
where
    MA: Variant<Data<f32>>,
    MB: Variant<Data<f32>>,
{
    type Observe = MyMobject0<MA::Observe, MB::Observe>;
    type Keys = Derivation<MyMobject0<MA::Keys, MB::Keys>, ()>;

    fn allocate(
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        Derivation {
            intrinsic: MyMobject0 {
                ma: MA::allocate(&observe.ma, slot_key_generator_map),
                mb: MB::allocate(&observe.mb, slot_key_generator_map),
            },
            extrinsic: (),
        }
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <MyMobject0 as Mobject>::ResourceRef<'s> {
        MyMobject0 {
            ma: MA::get(&keys.intrinsic.ma, storage_type_map),
            mb: MB::get(&keys.intrinsic.mb, storage_type_map),
        }
    }

    fn prepare(
        observe: &Self::Observe,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        MA::prepare(
            &observe.ma,
            &keys.intrinsic.ma,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
        MB::prepare(
            &observe.mb,
            &keys.intrinsic.mb,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        MA::render(&keys.intrinsic.ma, storage_type_map, render_pass);
        MB::render(&keys.intrinsic.mb, storage_type_map, render_pass);
    }
}

impl Timeline<MyMobject0> for StaticTimeline {
    type Variant = StaticVariant;

    fn observe(
        _clock: Clock,
        _clock_span: ClockSpan,
        _timeline: &Self,
        observe: &<Self::Variant as Variant<MyMobject0>>::Observe,
    ) -> <Self::Variant as Variant<MyMobject0>>::Observe {
        observe.clone()
    }
}

impl<R> Timeline<MyMobject0> for DynamicTimeline<R>
where
    R: Refresh<MyMobject0>,
{
    type Variant = DynamicVariant;

    fn observe(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &<Self::Variant as Variant<MyMobject0>>::Observe,
    ) -> <Self::Variant as Variant<MyMobject0>>::Observe {
        let mut observe = observe.clone();
        timeline.refresh.refresh(clock, clock_span, &mut observe);
        observe
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
*/

// demo

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject0<MA = Data<f32>, MB = Data<f32>> {
    ma: MA,
    mb: MB,
}

impl Mobject for MyMobject0 {
    type MobjectRef<'m> =
        MyMobject0<<Data<f32> as Mobject>::MobjectRef<'m>, <Data<f32> as Mobject>::MobjectRef<'m>>;
    type ResourceRef<'s> = MyMobject0<
        <Data<f32> as Mobject>::ResourceRef<'s>,
        <Data<f32> as Mobject>::ResourceRef<'s>,
    >;
    type ResourceRefInput<'s> = MyMobject0<
        <Data<f32> as Mobject>::ResourceRef<'s>,
        <Data<f32> as Mobject>::ResourceRef<'s>,
    >;

    fn mobject_ref<'m>(mobject: &'m Self) -> Self::MobjectRef<'m> {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::mobject_ref(&mobject.ma),
            mb: <Data<f32> as Mobject>::mobject_ref(&mobject.mb),
        }
    }
}

// impl Variant<MyMobject0> for StaticDerivationVariant {
//     type Observe = MyMobject0;
//     type Keys = MyMobject0<
//         <StaticDerivationVariant as Variant<Data<f32>>>::Keys,
//         <StaticDerivationVariant as Variant<Data<f32>>>::Keys,
//     >;

//     fn allocate(
//         observe: &Self::Observe,
//         slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> Self::Keys {
//         MyMobject0 {
//             ma: <StaticDerivationVariant as Variant<Data<f32>>>::allocate(
//                 &observe.ma,
//                 slot_key_generator_map,
//             ),
//             mb: <StaticDerivationVariant as Variant<Data<f32>>>::allocate(
//                 &observe.mb,
//                 slot_key_generator_map,
//             ),
//         }
//     }

//     fn get<'s>(
//         keys: &Self::Keys,
//         storage_type_map: &'s StorageTypeMap,
//     ) -> <MyMobject0 as Mobject>::ResourceRef<'s> {
//         MyMobject0 {
//             ma: <StaticDerivationVariant as Variant<Data<f32>>>::get(&keys.ma, storage_type_map),
//             mb: <StaticDerivationVariant as Variant<Data<f32>>>::get(&keys.mb, storage_type_map),
//         }
//     }

//     fn prepare(
//         observe: &Self::Observe,
//         keys: &Self::Keys,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         reuse: &mut ResourceReuseResult,
//     ) {
//         <StaticDerivationVariant as Variant<Data<f32>>>::prepare(
//             &observe.ma,
//             &keys.ma,
//             storage_type_map,
//             device,
//             queue,
//             format,
//             reuse,
//         );
//         <StaticDerivationVariant as Variant<Data<f32>>>::prepare(
//             &observe.mb,
//             &keys.mb,
//             storage_type_map,
//             device,
//             queue,
//             format,
//             reuse,
//         );
//     }

//     fn render(
//         keys: &Self::Keys,
//         storage_type_map: &StorageTypeMap,
//         render_pass: &mut wgpu::RenderPass,
//     ) {
//         <StaticDerivationVariant as Variant<Data<f32>>>::render(
//             &keys.ma,
//             storage_type_map,
//             render_pass,
//         );
//         <StaticDerivationVariant as Variant<Data<f32>>>::render(
//             &keys.mb,
//             storage_type_map,
//             render_pass,
//         );
//     }
// }

// impl Variant<MyMobject0> for DynamicDerivationVariant {
//     type Observe = MyMobject0;
//     type Keys = MyMobject0<
//         <DynamicDerivationVariant as Variant<Data<f32>>>::Keys,
//         <DynamicDerivationVariant as Variant<Data<f32>>>::Keys,
//     >;

//     fn allocate(
//         observe: &Self::Observe,
//         slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> Self::Keys {
//         MyMobject0 {
//             ma: <DynamicDerivationVariant as Variant<Data<f32>>>::allocate(
//                 &observe.ma,
//                 slot_key_generator_map,
//             ),
//             mb: <DynamicDerivationVariant as Variant<Data<f32>>>::allocate(
//                 &observe.mb,
//                 slot_key_generator_map,
//             ),
//         }
//     }

//     fn get<'s>(
//         keys: &Self::Keys,
//         storage_type_map: &'s StorageTypeMap,
//     ) -> <MyMobject0 as Mobject>::ResourceRef<'s> {
//         MyMobject0 {
//             ma: <DynamicDerivationVariant as Variant<Data<f32>>>::get(&keys.ma, storage_type_map),
//             mb: <DynamicDerivationVariant as Variant<Data<f32>>>::get(&keys.mb, storage_type_map),
//         }
//     }

//     fn prepare(
//         observe: &Self::Observe,
//         keys: &Self::Keys,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         reuse: &mut ResourceReuseResult,
//     ) {
//         <DynamicDerivationVariant as Variant<Data<f32>>>::prepare(
//             &observe.ma,
//             &keys.ma,
//             storage_type_map,
//             device,
//             queue,
//             format,
//             reuse,
//         );
//         <DynamicDerivationVariant as Variant<Data<f32>>>::prepare(
//             &observe.mb,
//             &keys.mb,
//             storage_type_map,
//             device,
//             queue,
//             format,
//             reuse,
//         );
//     }

//     fn render(
//         keys: &Self::Keys,
//         storage_type_map: &StorageTypeMap,
//         render_pass: &mut wgpu::RenderPass,
//     ) {
//         <DynamicDerivationVariant as Variant<Data<f32>>>::render(
//             &keys.ma,
//             storage_type_map,
//             render_pass,
//         );
//         <DynamicDerivationVariant as Variant<Data<f32>>>::render(
//             &keys.mb,
//             storage_type_map,
//             render_pass,
//         );
//     }
// }

impl VariantFields<MyMobject0> for StaticVariant {
    // type Observe = MyMobject0;
    // type ObserveRef<'m> = MyMobject0<
    //     <StaticVariant as Variant<Data<f32>>>::ObserveRef<'m>,
    //     <StaticVariant as Variant<Data<f32>>>::ObserveRef<'m>,
    // >;
    type Keys = MyMobject0<<Self as Variant<Data<f32>>>::Keys, <Self as Variant<Data<f32>>>::Keys>;

    // fn mobject_ref<'m>(
    //     observe_ref: Self::ObserveRef<'m>,
    // ) -> <MyMobject0 as Mobject>::MobjectRef<'m> {
    //     MyMobject0 {
    //         ma: <StaticVariant as Variant<Data<f32>>>::mobject_ref(observe_ref.ma),
    //         mb: <StaticVariant as Variant<Data<f32>>>::mobject_ref(observe_ref.mb),
    //     }
    // }

    fn allocate(
        mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        MyMobject0 {
            ma: <Self as Variant<Data<f32>>>::allocate(mobject_ref.ma, slot_key_generator_map),
            mb: <Self as Variant<Data<f32>>>::allocate(mobject_ref.mb, slot_key_generator_map),
        }
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <MyMobject0 as Mobject>::ResourceRefInput<'s> {
        MyMobject0 {
            ma: <Self as Variant<Data<f32>>>::get(&keys.ma, storage_type_map),
            mb: <Self as Variant<Data<f32>>>::get(&keys.mb, storage_type_map),
        }
    }

    fn prepare(
        mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <Self as Variant<Data<f32>>>::prepare(
            mobject_ref.ma,
            &keys.ma,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
        <Self as Variant<Data<f32>>>::prepare(
            mobject_ref.mb,
            &keys.mb,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        <Self as Variant<Data<f32>>>::render(&keys.ma, storage_type_map, render_pass);
        <Self as Variant<Data<f32>>>::render(&keys.mb, storage_type_map, render_pass);
    }
}

impl VariantFields<MyMobject0> for DynamicVariant {
    // type Observe = MyMobject0;
    // type ObserveRef<'m> = MyMobject0<
    //     <DynamicVariant as Variant<Data<f32>>>::ObserveRef<'m>,
    //     <DynamicVariant as Variant<Data<f32>>>::ObserveRef<'m>,
    // >;
    type Keys = MyMobject0<<Self as Variant<Data<f32>>>::Keys, <Self as Variant<Data<f32>>>::Keys>;

    // fn mobject_ref<'m>(
    //     observe_ref: Self::ObserveRef<'m>,
    // ) -> <MyMobject0 as Mobject>::MobjectRef<'m> {
    //     MyMobject0 {
    //         ma: <Self as Variant<Data<f32>>>::mobject_ref(observe_ref.ma),
    //         mb: <Self as Variant<Data<f32>>>::mobject_ref(observe_ref.mb),
    //     }
    // }

    fn allocate(
        mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        MyMobject0 {
            ma: <Self as Variant<Data<f32>>>::allocate(mobject_ref.ma, slot_key_generator_map),
            mb: <Self as Variant<Data<f32>>>::allocate(mobject_ref.mb, slot_key_generator_map),
        }
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <MyMobject0 as Mobject>::ResourceRefInput<'s> {
        MyMobject0 {
            ma: <Self as Variant<Data<f32>>>::get(&keys.ma, storage_type_map),
            mb: <Self as Variant<Data<f32>>>::get(&keys.mb, storage_type_map),
        }
    }

    fn prepare(
        mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <Self as Variant<Data<f32>>>::prepare(
            mobject_ref.ma,
            &keys.ma,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
        <Self as Variant<Data<f32>>>::prepare(
            mobject_ref.mb,
            &keys.mb,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        <Self as Variant<Data<f32>>>::render(&keys.ma, storage_type_map, render_pass);
        <Self as Variant<Data<f32>>>::render(&keys.mb, storage_type_map, render_pass);
    }
}

impl<MA, MB> VariantFields<MyMobject0> for MyMobject0<MA, MB>
where
    MA: Variant<Data<f32>>,
    MB: Variant<Data<f32>>,
{
    // type Observe = MyMobject0<MA::Observe, MB::Observe>;
    // type MobjectRef<'m> = MyMobject0<MA::MobjectRef<'m>, MB::MobjectRef<'m>>;
    type Keys = MyMobject0<MA::Keys, MB::Keys>;

    // fn mobject_ref<'m>(observe_ref: Self::ObserveRef<'m>) -> Self::MobjectRef<'m> {
    //     MyMobject0 {
    //         ma: MA::mobject_ref(&observe.ma),
    //         mb: MB::mobject_ref(&observe.mb),
    //     }
    // }

    fn allocate(
        mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        MyMobject0 {
            ma: MA::allocate(mobject_ref.ma, slot_key_generator_map),
            mb: MB::allocate(mobject_ref.mb, slot_key_generator_map),
        }
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <MyMobject0 as Mobject>::ResourceRefInput<'s> {
        MyMobject0 {
            ma: MA::get(&keys.ma, storage_type_map),
            mb: MB::get(&keys.mb, storage_type_map),
        }
    }

    fn prepare(
        mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        MA::prepare(
            mobject_ref.ma,
            &keys.ma,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
        MB::prepare(
            mobject_ref.mb,
            &keys.mb,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
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

impl Variant<MyMobject0> for StaticVariant {
    // type ObserveRef<'m> = <StaticVariant as VariantFields<MyMobject0>>::ObserveRef<'m>;
    type Keys = <Self as VariantFields<MyMobject0>>::Keys;

    // fn mobject_ref<'m>(
    //     observe_ref: Self::ObserveRef<'m>,
    // ) -> <MyMobject0 as Mobject>::MobjectRef<'m> {
    //     <Self as VariantFields<MyMobject0>>::mobject_ref(observe_ref)
    // }

    fn allocate(
        mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        <Self as VariantFields<MyMobject0>>::allocate(mobject_ref, slot_key_generator_map)
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <MyMobject0 as Mobject>::ResourceRef<'s> {
        <Self as VariantFields<MyMobject0>>::get(keys, storage_type_map)
    }

    fn prepare(
        mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <Self as VariantFields<MyMobject0>>::prepare(
            mobject_ref,
            keys,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        <Self as VariantFields<MyMobject0>>::render(keys, storage_type_map, render_pass);
    }
}

impl Variant<MyMobject0> for DynamicVariant {
    // type ObserveRef<'m> = <DynamicVariant as VariantFields<MyMobject0>>::ObserveRef<'m>;
    type Keys = <Self as VariantFields<MyMobject0>>::Keys;

    // fn mobject_ref<'m>(
    //     observe_ref: Self::ObserveRef<'m>,
    // ) -> <MyMobject0 as Mobject>::MobjectRef<'m> {
    //     <Self as VariantFields<MyMobject0>>::mobject_ref(observe_ref)
    // }

    fn allocate(
        mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        <Self as VariantFields<MyMobject0>>::allocate(mobject_ref, slot_key_generator_map)
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <MyMobject0 as Mobject>::ResourceRef<'s> {
        <Self as VariantFields<MyMobject0>>::get(keys, storage_type_map)
    }

    fn prepare(
        mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <Self as VariantFields<MyMobject0>>::prepare(
            mobject_ref,
            keys,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        <Self as VariantFields<MyMobject0>>::render(keys, storage_type_map, render_pass);
    }
}

impl<MA, MB> Variant<MyMobject0> for MyMobject0<MA, MB>
where
    MA: Variant<Data<f32>>,
    MB: Variant<Data<f32>>,
{
    // type ObserveRef<'m> = <DynamicVariant as VariantFields<MyMobject0>>::ObserveRef<'m>;
    type Keys = <Self as VariantFields<MyMobject0>>::Keys;

    // fn mobject_ref<'m>(
    //     observe_ref: Self::ObserveRef<'m>,
    // ) -> <MyMobject0 as Mobject>::MobjectRef<'m> {
    //     <Self as VariantFields<MyMobject0>>::mobject_ref(observe_ref)
    // }

    fn allocate(
        mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        <Self as VariantFields<MyMobject0>>::allocate(mobject_ref, slot_key_generator_map)
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <MyMobject0 as Mobject>::ResourceRef<'s> {
        <Self as VariantFields<MyMobject0>>::get(keys, storage_type_map)
    }

    fn prepare(
        mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <Self as VariantFields<MyMobject0>>::prepare(
            mobject_ref,
            keys,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        <Self as VariantFields<MyMobject0>>::render(keys, storage_type_map, render_pass);
    }
}

impl Timeline<MyMobject0> for StaticTimeline {
    type Observe = Arc<MyMobject0>;
    type Variant = StaticVariant;

    fn observe(
        _clock: Clock,
        _clock_span: ClockSpan,
        _timeline: &Self,
        observe: &Self::Observe,
    ) -> Self::Observe {
        observe.clone()
    }

    fn mobject_ref<'m>(observe: &'m Self::Observe) -> <MyMobject0 as Mobject>::MobjectRef<'m> {
        <MyMobject0 as Mobject>::mobject_ref(observe.as_ref())
    }
}

impl<R> Timeline<MyMobject0> for DynamicTimeline<R>
where
    R: Refresh<MyMobject0>,
{
    type Observe = MyMobject0;
    type Variant = DynamicVariant;

    fn observe(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &Self::Observe,
    ) -> Self::Observe {
        let mut observe = observe.clone();
        timeline.refresh.refresh(clock, clock_span, &mut observe);
        observe
    }

    fn mobject_ref<'m>(observe: &'m Self::Observe) -> <MyMobject0 as Mobject>::MobjectRef<'m> {
        <MyMobject0 as Mobject>::mobject_ref(observe)
    }
}

impl<MA, MB> Timeline<MyMobject0> for MyMobject0<MA, MB>
where
    MA: Timeline<Data<f32>>,
    MB: Timeline<Data<f32>>,
{
    type Observe = MyMobject0<MA::Observe, MB::Observe>;
    type Variant = MyMobject0<MA::Variant, MB::Variant>;

    fn observe(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &Self::Observe,
    ) -> Self::Observe {
        MyMobject0 {
            ma: MA::observe(clock, clock_span, &timeline.ma, &observe.ma),
            mb: MB::observe(clock, clock_span, &timeline.mb, &observe.mb),
        }
    }

    fn mobject_ref<'m>(observe: &'m Self::Observe) -> <MyMobject0 as Mobject>::MobjectRef<'m> {
        MyMobject0 {
            ma: MA::mobject_ref(&observe.ma),
            mb: MB::mobject_ref(&observe.mb),
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

impl Prepare for MyMobject1 {
    type ResourceRepr = [f32; 4];
    type Resource = MyMobject1Resource;

    fn prepare(input: &Self::ResourceRefInput<'_>) -> Self::ResourceRepr {
        [*input.ma.ma, *input.ma.mb, *input.mb.ma, *input.mb.mb]
    }
}

impl Render for MyMobject1 {
    fn render(resource: &Self::ResourceRef<'_>, _render_pass: &mut wgpu::RenderPass) {
        println!("{:?}", resource.0);
    }
}

//

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject1<MA = MyMobject0, MB = MyMobject0> {
    ma: MA,
    mb: MB,
}

impl Mobject for MyMobject1 {
    type MobjectRef<'m> = MyMobject1<
        <MyMobject0 as Mobject>::MobjectRef<'m>,
        <MyMobject0 as Mobject>::MobjectRef<'m>,
    >;
    type ResourceRef<'s> = &'s <MyMobject1 as Prepare>::Resource;
    type ResourceRefInput<'s> = MyMobject1<
        <MyMobject0 as Mobject>::ResourceRef<'s>,
        <MyMobject0 as Mobject>::ResourceRef<'s>,
    >;

    fn mobject_ref<'m>(mobject: &'m Self) -> Self::MobjectRef<'m> {
        MyMobject1 {
            ma: <MyMobject0 as Mobject>::mobject_ref(&mobject.ma),
            mb: <MyMobject0 as Mobject>::mobject_ref(&mobject.mb),
        }
    }
}

impl VariantFields<MyMobject1> for StaticVariant {
    // type Observe = MyMobject1;
    // type ObserveRef<'m> = MyMobject1<
    //     <StaticVariant as Variant<MyMobject0>>::ObserveRef<'m>,
    //     <StaticVariant as Variant<MyMobject0>>::ObserveRef<'m>,
    // >;
    type Keys =
        MyMobject1<<Self as Variant<MyMobject0>>::Keys, <Self as Variant<MyMobject0>>::Keys>;

    // fn mobject_ref<'m>(
    //     observe_ref: Self::ObserveRef<'m>,
    // ) -> <MyMobject1 as Mobject>::MobjectRef<'m> {
    //     MyMobject1 {
    //         ma: <StaticVariant as Variant<MyMobject0>>::mobject_ref(observe_ref.ma),
    //         mb: <StaticVariant as Variant<MyMobject0>>::mobject_ref(observe_ref.mb),
    //     }
    // }

    fn allocate(
        mobject_ref: <MyMobject1 as Mobject>::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        MyMobject1 {
            ma: <Self as Variant<MyMobject0>>::allocate(mobject_ref.ma, slot_key_generator_map),
            mb: <Self as Variant<MyMobject0>>::allocate(mobject_ref.mb, slot_key_generator_map),
        }
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <MyMobject1 as Mobject>::ResourceRefInput<'s> {
        MyMobject1 {
            ma: <Self as Variant<MyMobject0>>::get(&keys.ma, storage_type_map),
            mb: <Self as Variant<MyMobject0>>::get(&keys.mb, storage_type_map),
        }
    }

    fn prepare(
        mobject_ref: <MyMobject1 as Mobject>::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <Self as Variant<MyMobject0>>::prepare(
            mobject_ref.ma,
            &keys.ma,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
        <Self as Variant<MyMobject0>>::prepare(
            mobject_ref.mb,
            &keys.mb,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        <Self as Variant<MyMobject0>>::render(&keys.ma, storage_type_map, render_pass);
        <Self as Variant<MyMobject0>>::render(&keys.mb, storage_type_map, render_pass);
    }
}

impl VariantFields<MyMobject1> for DynamicVariant {
    // type Observe = MyMobject1;
    // type ObserveRef<'m> = MyMobject1<
    //     <DynamicVariant as Variant<MyMobject0>>::ObserveRef<'m>,
    //     <DynamicVariant as Variant<MyMobject0>>::ObserveRef<'m>,
    // >;
    type Keys =
        MyMobject1<<Self as Variant<MyMobject0>>::Keys, <Self as Variant<MyMobject0>>::Keys>;

    // fn mobject_ref<'m>(
    //     observe_ref: Self::ObserveRef<'m>,
    // ) -> <MyMobject1 as Mobject>::MobjectRef<'m> {
    //     MyMobject1 {
    //         ma: <Self as Variant<MyMobject0>>::mobject_ref(observe_ref.ma),
    //         mb: <Self as Variant<MyMobject0>>::mobject_ref(observe_ref.mb),
    //     }
    // }

    fn allocate(
        mobject_ref: <MyMobject1 as Mobject>::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        MyMobject1 {
            ma: <Self as Variant<MyMobject0>>::allocate(mobject_ref.ma, slot_key_generator_map),
            mb: <Self as Variant<MyMobject0>>::allocate(mobject_ref.mb, slot_key_generator_map),
        }
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <MyMobject1 as Mobject>::ResourceRefInput<'s> {
        MyMobject1 {
            ma: <Self as Variant<MyMobject0>>::get(&keys.ma, storage_type_map),
            mb: <Self as Variant<MyMobject0>>::get(&keys.mb, storage_type_map),
        }
    }

    fn prepare(
        mobject_ref: <MyMobject1 as Mobject>::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <Self as Variant<MyMobject0>>::prepare(
            mobject_ref.ma,
            &keys.ma,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
        <Self as Variant<MyMobject0>>::prepare(
            mobject_ref.mb,
            &keys.mb,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        <Self as Variant<MyMobject0>>::render(&keys.ma, storage_type_map, render_pass);
        <Self as Variant<MyMobject0>>::render(&keys.mb, storage_type_map, render_pass);
    }
}

impl<MA, MB> VariantFields<MyMobject1> for MyMobject1<MA, MB>
where
    MA: Variant<MyMobject0>,
    MB: Variant<MyMobject0>,
{
    // type Observe = MyMobject1<MA::Observe, MB::Observe>;
    // type MobjectRef<'m> = MyMobject1<MA::MobjectRef<'m>, MB::MobjectRef<'m>>;
    type Keys = MyMobject1<MA::Keys, MB::Keys>;

    // fn mobject_ref<'m>(observe_ref: Self::ObserveRef<'m>) -> Self::MobjectRef<'m> {
    //     MyMobject1 {
    //         ma: MA::mobject_ref(&observe.ma),
    //         mb: MB::mobject_ref(&observe.mb),
    //     }
    // }

    fn allocate(
        mobject_ref: <MyMobject1 as Mobject>::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        MyMobject1 {
            ma: MA::allocate(mobject_ref.ma, slot_key_generator_map),
            mb: MB::allocate(mobject_ref.mb, slot_key_generator_map),
        }
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <MyMobject1 as Mobject>::ResourceRefInput<'s> {
        MyMobject1 {
            ma: MA::get(&keys.ma, storage_type_map),
            mb: MB::get(&keys.mb, storage_type_map),
        }
    }

    fn prepare(
        mobject_ref: <MyMobject1 as Mobject>::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        MA::prepare(
            mobject_ref.ma,
            &keys.ma,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
        MB::prepare(
            mobject_ref.mb,
            &keys.mb,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
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

impl Variant<MyMobject1> for StaticVariant {
    // type Observe = Arc<<StaticVariant as VariantFields<MyMobject1>>::Observe>;
    // type MobjectRef<'m> = <StaticVariant as VariantFields<MyMobject1>>::MobjectRef<'m>;
    type Keys = Derivation<
        StorageKey<StaticStoreType<MyMobject1>>,
        <StaticVariant as VariantFields<MyMobject1>>::Keys,
    >;

    // fn mobject_ref<'m>(observe_ref: Self::ObserveRef<'m>) -> Self::MobjectRef<'m> {
    //     <StaticVariant as VariantFields<MyMobject1>>::mobject_ref(observe.as_ref())
    // }

    fn allocate(
        mobject_ref: <MyMobject1 as Mobject>::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        Derivation {
            extrinsic: slot_key_generator_map.allocate(&mobject_ref),
            intrinsic: <StaticVariant as VariantFields<MyMobject1>>::allocate(
                mobject_ref,
                slot_key_generator_map,
            ),
        }
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <MyMobject1 as Mobject>::ResourceRef<'s> {
        storage_type_map.read(&keys.extrinsic)
    }

    fn prepare(
        mobject_ref: <MyMobject1 as Mobject>::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <StaticVariant as VariantFields<MyMobject1>>::prepare(
            mobject_ref,
            &keys.intrinsic,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
        let resource_repr = <MyMobject1 as Prepare>::prepare(&<StaticVariant as VariantFields<
            MyMobject1,
        >>::get(
            &keys.intrinsic, storage_type_map
        ));
        storage_type_map.write(
            &keys.extrinsic,
            reuse,
            (resource_repr, device, queue, format),
        );
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        <MyMobject1 as Render>::render(
            &<Self as Variant<MyMobject1>>::get(keys, storage_type_map),
            render_pass,
        );
    }
}

impl Variant<MyMobject1> for DynamicVariant {
    // type Observe = <DynamicVariant as VariantFields<MyMobject1>>::Observe;
    // type MobjectRef<'m> = <DynamicVariant as VariantFields<MyMobject1>>::MobjectRef<'m>;
    type Keys = Derivation<
        StorageKey<StaticStoreType<MyMobject1>>,
        <DynamicVariant as VariantFields<MyMobject1>>::Keys,
    >;

    // fn mobject_ref<'m>(observe_ref: Self::ObserveRef<'m>) -> Self::MobjectRef<'m> {
    //     <DynamicVariant as VariantFields<MyMobject1>>::mobject_ref(observe)
    // }

    fn allocate(
        mobject_ref: <MyMobject1 as Mobject>::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        Derivation {
            extrinsic: slot_key_generator_map.allocate(&mobject_ref),
            intrinsic: <DynamicVariant as VariantFields<MyMobject1>>::allocate(
                mobject_ref,
                slot_key_generator_map,
            ),
        }
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <MyMobject1 as Mobject>::ResourceRef<'s> {
        storage_type_map.read(&keys.extrinsic)
    }

    fn prepare(
        mobject_ref: <MyMobject1 as Mobject>::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <DynamicVariant as VariantFields<MyMobject1>>::prepare(
            mobject_ref,
            &keys.intrinsic,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
        let resource_repr = <MyMobject1 as Prepare>::prepare(&<DynamicVariant as VariantFields<
            MyMobject1,
        >>::get(
            &keys.intrinsic, storage_type_map
        ));
        storage_type_map.write(
            &keys.extrinsic,
            reuse,
            (resource_repr, device, queue, format),
        );
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        <MyMobject1 as Render>::render(
            &<Self as Variant<MyMobject1>>::get(keys, storage_type_map),
            render_pass,
        );
    }
}

impl<MA, MB> Variant<MyMobject1> for MyMobject1<MA, MB>
where
    MA: Variant<MyMobject0>,
    MB: Variant<MyMobject0>,
{
    // type Observe = <Self as VariantFields<MyMobject1>>::Observe;
    // type MobjectRef<'m> = <Self as VariantFields<MyMobject1>>::MobjectRef<'m>;
    type Keys = Derivation<
        StorageKey<DynamicStoreType<MyMobject1>>,
        <Self as VariantFields<MyMobject1>>::Keys,
    >;

    // fn mobject_ref<'m>(observe_ref: Self::ObserveRef<'m>) -> Self::MobjectRef<'m> {
    //     <Self as VariantFields<MyMobject1>>::mobject_ref(observe)
    // }

    fn allocate(
        mobject_ref: <MyMobject1 as Mobject>::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        Derivation {
            extrinsic: slot_key_generator_map.allocate(&()),
            intrinsic: <Self as VariantFields<MyMobject1>>::allocate(
                mobject_ref,
                slot_key_generator_map,
            ),
        }
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <MyMobject1 as Mobject>::ResourceRef<'s> {
        storage_type_map.read(&keys.extrinsic)
    }

    fn prepare(
        mobject_ref: <MyMobject1 as Mobject>::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <Self as VariantFields<MyMobject1>>::prepare(
            mobject_ref,
            &keys.intrinsic,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
        let resource_repr = <MyMobject1 as Prepare>::prepare(&<Self as VariantFields<
            MyMobject1,
        >>::get(
            &keys.intrinsic, storage_type_map
        ));
        storage_type_map.write(
            &keys.extrinsic,
            reuse,
            (resource_repr, device, queue, format),
        );
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        <MyMobject1 as Render>::render(
            &<Self as Variant<MyMobject1>>::get(keys, storage_type_map),
            render_pass,
        );
    }
}

impl Timeline<MyMobject1> for StaticTimeline {
    type Observe = Arc<MyMobject1>;
    type Variant = StaticVariant;

    fn observe(
        _clock: Clock,
        _clock_span: ClockSpan,
        _timeline: &Self,
        observe: &Self::Observe,
    ) -> Self::Observe {
        observe.clone()
    }

    fn mobject_ref<'m>(observe: &'m Self::Observe) -> <MyMobject1 as Mobject>::MobjectRef<'m> {
        <MyMobject1 as Mobject>::mobject_ref(observe.as_ref())
    }
}

impl<R> Timeline<MyMobject1> for DynamicTimeline<R>
where
    R: Refresh<MyMobject1>,
{
    type Observe = MyMobject1;
    type Variant = DynamicVariant;

    fn observe(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &Self::Observe,
    ) -> Self::Observe {
        let mut observe = observe.clone();
        timeline.refresh.refresh(clock, clock_span, &mut observe);
        observe
    }

    fn mobject_ref<'m>(observe: &'m Self::Observe) -> <MyMobject1 as Mobject>::MobjectRef<'m> {
        <MyMobject1 as Mobject>::mobject_ref(observe)
    }
}

impl<MA, MB> Timeline<MyMobject1> for MyMobject1<MA, MB>
where
    MA: Timeline<MyMobject0>,
    MB: Timeline<MyMobject0>,
{
    type Observe = MyMobject1<MA::Observe, MB::Observe>;
    type Variant = MyMobject1<MA::Variant, MB::Variant>;

    fn observe(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &Self::Observe,
    ) -> Self::Observe {
        MyMobject1 {
            ma: MA::observe(clock, clock_span, &timeline.ma, &observe.ma),
            mb: MB::observe(clock, clock_span, &timeline.mb, &observe.mb),
        }
    }

    fn mobject_ref<'m>(observe: &'m Self::Observe) -> <MyMobject1 as Mobject>::MobjectRef<'m> {
        MyMobject1 {
            ma: MA::mobject_ref(&observe.ma),
            mb: MB::mobject_ref(&observe.mb),
        }
    }
}
