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

trait Coerce<T> {
    fn coerce(self) -> T;
    fn coerce_ref(&self) -> &T;
}

impl<T> Coerce<T> for T {
    fn coerce(self) -> T {
        self
    }

    fn coerce_ref(&self) -> &T {
        self
    }
}

pub trait Mobject:
    'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
{
    type MobjectCategory: MobjectCategory;
    type MobjectRef<'m>: serde::Serialize;
    type ResourceRef<'s>;
    type ResourceRefInput<'s>;
    type Variant<VS>: Variant<Self::MobjectCategory, Self>
    where
        VS: VariantSeed;
    // type DynamicVariant: Variant<Self>;
    type PreVariant<VS>
    where
        VS: VariantSeed;
    // type DynamicVariantFields;

    fn mobject_ref<'m>(mobject: &'m Self) -> Self::MobjectRef<'m>;
}

pub trait PreVariant<M>
where
    M: Mobject,
{
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

pub trait Variant<MC, M>
where
    MC: MobjectCategory,
    M: Mobject,
{
    type Keys;

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
    type Variant: Variant<M::MobjectCategory, M>;

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

pub trait MobjectCategory: 'static + Send + Sync {}

pub struct NotPrepareNotRenderMobjectCategory;

pub struct NotPrepareRenderMobjectCategory;

pub struct PrepareNotRenderMobjectCategory;

pub struct PrepareRenderMobjectCategory;

impl MobjectCategory for NotPrepareNotRenderMobjectCategory {}

impl MobjectCategory for NotPrepareRenderMobjectCategory {}

impl MobjectCategory for PrepareNotRenderMobjectCategory {}

impl MobjectCategory for PrepareRenderMobjectCategory {}

pub trait VariantSeed: 'static + Send + Sync {
    type Slot<V>: Slot<Value = V>
    where
        V: 'static + Send + Sync;

    fn key<'s, KI>(key_input: &'s KI) -> &'s dyn serde_traitobject::Serialize
    where
        KI: serde::Serialize;
    fn insert<RR, R>(
        resource_repr: RR,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> R
    where
        R: Resource<RR>;
    fn update<RR, R>(
        value: &mut R,
        resource_repr: RR,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) where
        R: Resource<RR>;
}

pub struct StaticVariantSeed;

impl VariantSeed for StaticVariantSeed {
    type Slot<V> = SwapSlot<SingletonSlot<V>> where V: 'static + Send + Sync;

    fn key<'s, KI>(key_input: &'s KI) -> &'s dyn serde_traitobject::Serialize
    where
        KI: serde::Serialize,
    {
        key_input
    }

    fn insert<RR, R>(
        resource_repr: RR,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> R
    where
        R: Resource<RR>,
    {
        R::new(resource_repr, device, queue, format)
    }

    fn update<RR, R>(
        _value: &mut R,
        _resource_repr: RR,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        _reuse: &mut ResourceReuseResult,
    ) where
        R: Resource<RR>,
    {
    }
}

pub struct DynamicVariantSeed;

impl VariantSeed for DynamicVariantSeed {
    type Slot<V> = SwapSlot<MultitonSlot<V>> where V: 'static + Send + Sync;

    fn key<'s, KI>(_key_input: &'s KI) -> &'s dyn serde_traitobject::Serialize
    where
        KI: serde::Serialize,
    {
        &()
    }

    fn insert<RR, R>(
        resource_repr: RR,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> R
    where
        R: Resource<RR>,
    {
        R::new(resource_repr, device, queue, format)
    }

    fn update<RR, R>(
        value: &mut R,
        resource_repr: RR,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) where
        R: Resource<RR>,
    {
        R::update(value, resource_repr, device, queue, format, reuse)
    }
}

pub trait AbstractVariant<M> {
    type VariantSeed: VariantSeed;
    type StoreType;
}

pub struct VariantLeaf<M, VS>(M, VS);

pub struct VariantBranch<PV>(PV);

// pub struct CategorizedVariant<MC, AV>(MC, AV);

// pub struct StoreTypeSpec<M, VS>(M, VS);

impl<M, VS> StoreType for VariantLeaf<M, VS>
where
    M: Prepare,
    VS: VariantSeed,
{
    type Slot = VS::Slot<M::Resource>;
    type KeyInput<'s> = M::MobjectRef<'s>;
    type Input<'s> = (
        M::ResourceRepr,
        &'s wgpu::Device,
        &'s wgpu::Queue,
        wgpu::TextureFormat,
    );
    // type Output<'s> = &'s M::Resource;

    fn key<'s>(key_input: &'s Self::KeyInput<'_>) -> &'s dyn serde_traitobject::Serialize {
        VS::key(key_input)
    }

    fn insert(input: Self::Input<'_>) -> <Self::Slot as Slot>::Value {
        let (resource_repr, device, queue, format) = input;
        VS::insert::<M::ResourceRepr, M::Resource>(resource_repr, device, queue, format)
    }

    fn update(
        input: Self::Input<'_>,
        value: &mut <Self::Slot as Slot>::Value,
        reuse: &mut ResourceReuseResult,
    ) {
        let (resource_repr, device, queue, format) = input;
        VS::update::<M::ResourceRepr, M::Resource>(
            value,
            resource_repr,
            device,
            queue,
            format,
            reuse,
        )
    }
}

// pub struct DynamicStoreType<M>(M);

pub struct Derivation<E, I> {
    pub extrinsic: E,
    pub intrinsic: I,
}

pub trait Refresh<M>: 'static + Send + Sync
where
    M: Mobject,
{
    fn refresh(&self, clock: Clock, clock_span: ClockSpan, mobject: &mut M);
}

pub struct StaticTimeline;

pub struct DynamicTimeline<R> {
    refresh: R,
}

// data

trait DataTrait:
    'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
{
}

impl<T> DataTrait for T where
    T: 'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
{
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

impl<T> Mobject for Data<T>
where
    T: DataTrait,
{
    type MobjectCategory = NotPrepareNotRenderMobjectCategory;
    type MobjectRef<'m> = &'m T;
    type ResourceRef<'s> = &'s T;
    type ResourceRefInput<'s> = &'s T;
    type Variant<VS> = VariantLeaf<Self, VS>
    where
        VS: VariantSeed;
    type PreVariant<VS> = VariantLeaf<Self, VS>
    where
        VS: VariantSeed;

    fn mobject_ref<'m>(mobject: &'m Self) -> Self::MobjectRef<'m> {
        mobject
    }
}

impl<T> Resource<T> for T
where
    T: DataTrait,
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
    T: DataTrait,
{
    type ResourceRepr = T;
    type Resource = T;

    fn prepare(input: &<Self as Mobject>::ResourceRefInput<'_>) -> Self::ResourceRepr {
        (*input).clone()
    }
}

impl<T, VS> PreVariant<Data<T>> for VariantLeaf<Data<T>, VS>
where
    // MC: MobjectCategory,
    T: DataTrait,
    VS: VariantSeed,
    Self: StoreType,
    // M: Mobject<MobjectCategory = PrepareNotRenderMobjectCategory> + Prepare,
    // M::AbstractVariantInput<VS>: AbstractVariantInput<M>,
    // Self: StoreType,
    for<'s> <Data<T> as Mobject>::MobjectRef<'s>: Coerce<<Self as StoreType>::KeyInput<'s>>,
    for<'s> (
        <Data<T> as Prepare>::ResourceRepr,
        &'s wgpu::Device,
        &'s wgpu::Queue,
        wgpu::TextureFormat,
    ): Coerce<<Self as StoreType>::Input<'s>>,
    for<'s> &'s <<Self as StoreType>::Slot as Slot>::Value:
        Coerce<<Data<T> as Mobject>::ResourceRef<'s>>,
{
    type Keys = StorageKey<Self>;

    fn allocate(
        mobject_ref: <Data<T> as Mobject>::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        slot_key_generator_map.allocate(&mobject_ref.coerce())
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <Data<T> as Mobject>::ResourceRef<'s> {
        storage_type_map.read(keys).coerce()
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
        let resource_repr = <Data<T> as Prepare>::prepare(&mobject_ref);
        storage_type_map.write(
            keys,
            reuse,
            ((resource_repr, device, queue, format)).coerce(),
        );
    }

    fn render(
        _keys: &Self::Keys,
        _storage_type_map: &StorageTypeMap,
        _render_pass: &mut wgpu::RenderPass,
    ) {
    }
}

// fn is_variant<T, U>()
// where
//     U: Mobject,
//     T: Variant<U>,
// {
// }

// fn is_not_data<T: NotData>() {}

// fn test() {
//     is_variant::<
//         VariantLeaf<Data<f32>, NotPrepareNotRenderMobjectCategory, StaticVariantSeed>,
//         Data<f32>,
//     >();
//     is_variant::<
//         VariantLeaf<Data<f32>, NotPrepareNotRenderMobjectCategory, DynamicVariantSeed>,
//         Data<f32>,
//     >();
//     is_variant::<
//         VariantLeaf<MyMobject0, NotPrepareNotRenderMobjectCategory, DynamicVariantSeed>,
//         MyMobject0,
//     >();
//     is_not_data::<MyMobject0>();
// }

// impl<T> Variant<Data<T>> for <Data<T> as Mobject>::DynamicVariant
// where
//     T: DataTrait,
// {
//     type Keys = StorageKey<DynamicStoreType<Data<T>>>;

//     fn allocate(
//         _mobject_ref: <Data<T> as Mobject>::MobjectRef<'_>,
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
//         mobject_ref: <Data<T> as Mobject>::MobjectRef<'_>,
//         keys: &Self::Keys,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         reuse: &mut ResourceReuseResult,
//     ) {
//         let resource_repr = <Data<T> as Prepare>::prepare(&mobject_ref);
//         storage_type_map.write(keys, reuse, (resource_repr, device, queue, format));
//     }

//     fn render(
//         _keys: &Self::Keys,
//         _storage_type_map: &StorageTypeMap,
//         _render_pass: &mut wgpu::RenderPass,
//     ) {
//     }
// }

// impl<T> Timeline<Data<T>> for StaticTimeline
// where
//     T: DataTrait,
// {
//     type Observe = Arc<Data<T>>;
//     type Variant = CategorizedVariant<
//         <Data<T> as Mobject>::MobjectCategory,
//         <Data<T> as Mobject>::AbstractVariant<StaticVariantSeed>,
//     >;

//     fn observe(
//         _clock: Clock,
//         _clock_span: ClockSpan,
//         _timeline: &Self,
//         observe: &Self::Observe,
//     ) -> Self::Observe {
//         observe.clone()
//     }

//     fn mobject_ref<'m>(observe: &'m Self::Observe) -> <Data<T> as Mobject>::MobjectRef<'m> {
//         observe.as_ref()
//     }
// }

// impl<T, R> Timeline<Data<T>> for DynamicTimeline<R>
// where
//     T: DataTrait,
//     R: Refresh<Data<T>>,
// {
//     type Observe = Data<T>;
//     type Variant = CategorizedVariant<
//         <Data<T> as Mobject>::MobjectCategory,
//         <Data<T> as Mobject>::AbstractVariant<DynamicVariantSeed>,
//     >;

//     fn observe(
//         clock: Clock,
//         clock_span: ClockSpan,
//         timeline: &Self,
//         observe: &Self::Observe,
//     ) -> Self::Observe {
//         let mut observe = observe.clone();
//         timeline.refresh.refresh(clock, clock_span, &mut observe);
//         observe
//     }

//     fn mobject_ref<'m>(observe: &'m Self::Observe) -> <Data<T> as Mobject>::MobjectRef<'m> {
//         observe
//     }
// }

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
    type MobjectCategory = NotPrepareNotRenderMobjectCategory;
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
    type Variant<VS> = VariantLeaf<Self, VS>
    where
        VS: VariantSeed;
    // type DynamicVariant = DynamicVariant<Self::MobjectCategory>;
    type PreVariant<VS> =
        MyMobject0<<Data<f32> as Mobject>::Variant<VS>, <Data<f32> as Mobject>::Variant<VS>>
    where
        VS: VariantSeed;
    // type DynamicVariantFields =
    //     MyMobject0<<Data<f32> as Mobject>::DynamicVariant, <Data<f32> as Mobject>::DynamicVariant>;

    fn mobject_ref<'m>(mobject: &'m Self) -> Self::MobjectRef<'m> {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::mobject_ref(&mobject.ma),
            mb: <Data<f32> as Mobject>::mobject_ref(&mobject.mb),
        }
    }
}

// impl AbstractVariantInput<MyMobject0> for StaticVariant {
//     type Keys = MyMobject0<<Self as Variant<Data<f32>>>::Keys, <Self as Variant<Data<f32>>>::Keys>;

//     fn allocate(
//         mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
//         slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> Self::Keys {
//         MyMobject0 {
//             ma: <Self as Variant<Data<f32>>>::allocate(mobject_ref.ma, slot_key_generator_map),
//             mb: <Self as Variant<Data<f32>>>::allocate(mobject_ref.mb, slot_key_generator_map),
//         }
//     }

//     fn get<'s>(
//         keys: &Self::Keys,
//         storage_type_map: &'s StorageTypeMap,
//     ) -> <MyMobject0 as Mobject>::ResourceRefInput<'s> {
//         MyMobject0 {
//             ma: <Self as Variant<Data<f32>>>::get(&keys.ma, storage_type_map),
//             mb: <Self as Variant<Data<f32>>>::get(&keys.mb, storage_type_map),
//         }
//     }

//     fn prepare(
//         mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
//         keys: &Self::Keys,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         reuse: &mut ResourceReuseResult,
//     ) {
//         <Self as Variant<Data<f32>>>::prepare(
//             mobject_ref.ma,
//             &keys.ma,
//             storage_type_map,
//             device,
//             queue,
//             format,
//             reuse,
//         );
//         <Self as Variant<Data<f32>>>::prepare(
//             mobject_ref.mb,
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
//         <Self as Variant<Data<f32>>>::render(&keys.ma, storage_type_map, render_pass);
//         <Self as Variant<Data<f32>>>::render(&keys.mb, storage_type_map, render_pass);
//     }
// }

// impl AbstractVariantInput<MyMobject0> for DynamicVariant {
//     type Keys = MyMobject0<<Self as Variant<Data<f32>>>::Keys, <Self as Variant<Data<f32>>>::Keys>;

//     fn allocate(
//         mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
//         slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> Self::Keys {
//         MyMobject0 {
//             ma: <Self as Variant<Data<f32>>>::allocate(mobject_ref.ma, slot_key_generator_map),
//             mb: <Self as Variant<Data<f32>>>::allocate(mobject_ref.mb, slot_key_generator_map),
//         }
//     }

//     fn get<'s>(
//         keys: &Self::Keys,
//         storage_type_map: &'s StorageTypeMap,
//     ) -> <MyMobject0 as Mobject>::ResourceRefInput<'s> {
//         MyMobject0 {
//             ma: <Self as Variant<Data<f32>>>::get(&keys.ma, storage_type_map),
//             mb: <Self as Variant<Data<f32>>>::get(&keys.mb, storage_type_map),
//         }
//     }

//     fn prepare(
//         mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
//         keys: &Self::Keys,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         reuse: &mut ResourceReuseResult,
//     ) {
//         <Self as Variant<Data<f32>>>::prepare(
//             mobject_ref.ma,
//             &keys.ma,
//             storage_type_map,
//             device,
//             queue,
//             format,
//             reuse,
//         );
//         <Self as Variant<Data<f32>>>::prepare(
//             mobject_ref.mb,
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
//         <Self as Variant<Data<f32>>>::render(&keys.ma, storage_type_map, render_pass);
//         <Self as Variant<Data<f32>>>::render(&keys.mb, storage_type_map, render_pass);
//     }
// }

impl<MA, MB> PreVariant<MyMobject0> for MyMobject0<MA, MB>
where
    MA: Variant<<Data<f32> as Mobject>::MobjectCategory, Data<f32>>,
    MB: Variant<<Data<f32> as Mobject>::MobjectCategory, Data<f32>>,
{
    type Keys = MyMobject0<MA::Keys, MB::Keys>;

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

// impl Variant<MyMobject0> for StaticVariant {
//     type Keys = <<MyMobject0 as Mobject>::StaticVariant as AbstractVariantInput<MyMobject0>>::Keys;

//     fn allocate(
//         mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
//         slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> Self::Keys {
//         <<MyMobject0 as Mobject>::StaticVariant as AbstractVariantInput<MyMobject0>>::allocate(
//             mobject_ref,
//             slot_key_generator_map,
//         )
//     }

//     fn get<'s>(
//         keys: &Self::Keys,
//         storage_type_map: &'s StorageTypeMap,
//     ) -> <MyMobject0 as Mobject>::ResourceRef<'s> {
//         <<MyMobject0 as Mobject>::StaticVariant as AbstractVariantInput<MyMobject0>>::get(
//             keys,
//             storage_type_map,
//         )
//     }

//     fn prepare(
//         mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
//         keys: &Self::Keys,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         reuse: &mut ResourceReuseResult,
//     ) {
//         <<MyMobject0 as Mobject>::StaticVariant as AbstractVariantInput<MyMobject0>>::prepare(
//             mobject_ref,
//             keys,
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
//         <<MyMobject0 as Mobject>::StaticVariant as AbstractVariantInput<MyMobject0>>::render(
//             keys,
//             storage_type_map,
//             render_pass,
//         );
//     }
// }

// impl Variant<MyMobject0> for DynamicVariant {
//     type Keys = <<MyMobject0 as Mobject>::DynamicVariant as AbstractVariantInput<MyMobject0>>::Keys;

//     fn allocate(
//         mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
//         slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> Self::Keys {
//         <<MyMobject0 as Mobject>::DynamicVariant as AbstractVariantInput<MyMobject0>>::allocate(
//             mobject_ref,
//             slot_key_generator_map,
//         )
//     }

//     fn get<'s>(
//         keys: &Self::Keys,
//         storage_type_map: &'s StorageTypeMap,
//     ) -> <MyMobject0 as Mobject>::ResourceRef<'s> {
//         <<MyMobject0 as Mobject>::DynamicVariant as AbstractVariantInput<MyMobject0>>::get(
//             keys,
//             storage_type_map,
//         )
//     }

//     fn prepare(
//         mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
//         keys: &Self::Keys,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         reuse: &mut ResourceReuseResult,
//     ) {
//         <<MyMobject0 as Mobject>::DynamicVariant as AbstractVariantInput<MyMobject0>>::prepare(
//             mobject_ref,
//             keys,
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
//         <<MyMobject0 as Mobject>::DynamicVariant as AbstractVariantInput<MyMobject0>>::render(
//             keys,
//             storage_type_map,
//             render_pass,
//         );
//     }
// }

impl<MA, MB> Timeline<MyMobject0> for MyMobject0<MA, MB>
where
    MA: Timeline<Data<f32>>,
    MB: Timeline<Data<f32>>,
{
    type Observe = MyMobject0<MA::Observe, MB::Observe>;
    type Variant = VariantBranch<MyMobject0<MA::Variant, MB::Variant>>;

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
    type MobjectCategory = PrepareRenderMobjectCategory;
    type MobjectRef<'m> = MyMobject1<
        <MyMobject0 as Mobject>::MobjectRef<'m>,
        <MyMobject0 as Mobject>::MobjectRef<'m>,
    >;
    type ResourceRef<'s> = &'s <MyMobject1 as Prepare>::Resource;
    type ResourceRefInput<'s> = MyMobject1<
        <MyMobject0 as Mobject>::ResourceRef<'s>,
        <MyMobject0 as Mobject>::ResourceRef<'s>,
    >;
    type Variant<VS> = VariantLeaf<Self, VS>
    where
        VS: VariantSeed;
    // type DynamicVariant = DynamicVariant<Self::MobjectCategory>;
    type PreVariant<VS> =
        MyMobject1<<MyMobject0 as Mobject>::Variant<VS>, <MyMobject0 as Mobject>::Variant<VS>>
    where
        VS: VariantSeed;
    // type DynamicVariantFields = MyMobject1<
    //     <MyMobject0 as Mobject>::DynamicVariant,
    //     <MyMobject0 as Mobject>::DynamicVariant,
    // >;

    fn mobject_ref<'m>(mobject: &'m Self) -> Self::MobjectRef<'m> {
        MyMobject1 {
            ma: <MyMobject0 as Mobject>::mobject_ref(&mobject.ma),
            mb: <MyMobject0 as Mobject>::mobject_ref(&mobject.mb),
        }
    }
}

// impl AbstractVariantInput<MyMobject1> for StaticVariant {
//     type Keys =
//         MyMobject1<<Self as Variant<MyMobject0>>::Keys, <Self as Variant<MyMobject0>>::Keys>;

//     fn allocate(
//         mobject_ref: <MyMobject1 as Mobject>::MobjectRef<'_>,
//         slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> Self::Keys {
//         MyMobject1 {
//             ma: <Self as Variant<MyMobject0>>::allocate(mobject_ref.ma, slot_key_generator_map),
//             mb: <Self as Variant<MyMobject0>>::allocate(mobject_ref.mb, slot_key_generator_map),
//         }
//     }

//     fn get<'s>(
//         keys: &Self::Keys,
//         storage_type_map: &'s StorageTypeMap,
//     ) -> <MyMobject1 as Mobject>::ResourceRefInput<'s> {
//         MyMobject1 {
//             ma: <Self as Variant<MyMobject0>>::get(&keys.ma, storage_type_map),
//             mb: <Self as Variant<MyMobject0>>::get(&keys.mb, storage_type_map),
//         }
//     }

//     fn prepare(
//         mobject_ref: <MyMobject1 as Mobject>::MobjectRef<'_>,
//         keys: &Self::Keys,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         reuse: &mut ResourceReuseResult,
//     ) {
//         <Self as Variant<MyMobject0>>::prepare(
//             mobject_ref.ma,
//             &keys.ma,
//             storage_type_map,
//             device,
//             queue,
//             format,
//             reuse,
//         );
//         <Self as Variant<MyMobject0>>::prepare(
//             mobject_ref.mb,
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
//         <Self as Variant<MyMobject0>>::render(&keys.ma, storage_type_map, render_pass);
//         <Self as Variant<MyMobject0>>::render(&keys.mb, storage_type_map, render_pass);
//     }
// }

// impl AbstractVariantInput<MyMobject1> for DynamicVariant {
//     type Keys =
//         MyMobject1<<Self as Variant<MyMobject0>>::Keys, <Self as Variant<MyMobject0>>::Keys>;

//     fn allocate(
//         mobject_ref: <MyMobject1 as Mobject>::MobjectRef<'_>,
//         slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> Self::Keys {
//         MyMobject1 {
//             ma: <Self as Variant<MyMobject0>>::allocate(mobject_ref.ma, slot_key_generator_map),
//             mb: <Self as Variant<MyMobject0>>::allocate(mobject_ref.mb, slot_key_generator_map),
//         }
//     }

//     fn get<'s>(
//         keys: &Self::Keys,
//         storage_type_map: &'s StorageTypeMap,
//     ) -> <MyMobject1 as Mobject>::ResourceRefInput<'s> {
//         MyMobject1 {
//             ma: <Self as Variant<MyMobject0>>::get(&keys.ma, storage_type_map),
//             mb: <Self as Variant<MyMobject0>>::get(&keys.mb, storage_type_map),
//         }
//     }

//     fn prepare(
//         mobject_ref: <MyMobject1 as Mobject>::MobjectRef<'_>,
//         keys: &Self::Keys,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         reuse: &mut ResourceReuseResult,
//     ) {
//         <Self as Variant<MyMobject0>>::prepare(
//             mobject_ref.ma,
//             &keys.ma,
//             storage_type_map,
//             device,
//             queue,
//             format,
//             reuse,
//         );
//         <Self as Variant<MyMobject0>>::prepare(
//             mobject_ref.mb,
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
//         <Self as Variant<MyMobject0>>::render(&keys.ma, storage_type_map, render_pass);
//         <Self as Variant<MyMobject0>>::render(&keys.mb, storage_type_map, render_pass);
//     }
// }

impl<MA, MB> PreVariant<MyMobject1> for MyMobject1<MA, MB>
where
    MA: Variant<<MyMobject0 as Mobject>::MobjectCategory, MyMobject0>,
    MB: Variant<<MyMobject0 as Mobject>::MobjectCategory, MyMobject0>,
{
    type Keys = MyMobject1<MA::Keys, MB::Keys>;

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

impl<M, VS> AbstractVariant<M> for VariantLeaf<M, VS>
where
    M: Mobject,
    VS: VariantSeed,
{
    type VariantSeed = VS;
    type StoreType = Self;
}

impl<M, PV> AbstractVariant<M> for VariantBranch<PV>
where
    M: Mobject,
    PV: PreVariant<M>,
{
    type VariantSeed = DynamicVariantSeed;
    type StoreType = M::Variant<DynamicVariantSeed>;
}

impl<M, AV> Variant<NotPrepareNotRenderMobjectCategory, M> for AV
where
    M: Mobject<MobjectCategory = NotPrepareNotRenderMobjectCategory>,
    AV: AbstractVariant<M>,
    M::PreVariant<AV::VariantSeed>: PreVariant<M>,
    for<'s> M::ResourceRefInput<'s>: Coerce<M::ResourceRef<'s>>,
{
    type Keys = <M::PreVariant<AV::VariantSeed> as PreVariant<M>>::Keys;

    fn allocate(
        mobject_ref: M::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        <M::PreVariant<AV::VariantSeed> as PreVariant<M>>::allocate(
            mobject_ref,
            slot_key_generator_map,
        )
    }

    fn get<'s>(keys: &Self::Keys, storage_type_map: &'s StorageTypeMap) -> M::ResourceRef<'s> {
        <M::PreVariant<AV::VariantSeed> as PreVariant<M>>::get(keys, storage_type_map).coerce()
    }

    fn prepare(
        mobject_ref: M::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <M::PreVariant<AV::VariantSeed> as PreVariant<M>>::prepare(
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
        <M::PreVariant<AV::VariantSeed> as PreVariant<M>>::render(
            keys,
            storage_type_map,
            render_pass,
        );
    }
}

impl<M, AV> Variant<NotPrepareRenderMobjectCategory, M> for AV
where
    M: Mobject<MobjectCategory = NotPrepareRenderMobjectCategory> + Render,
    AV: AbstractVariant<M>,
    M::PreVariant<AV::VariantSeed>: PreVariant<M>,
    for<'s> M::ResourceRefInput<'s>: Coerce<M::ResourceRef<'s>>,
{
    type Keys = <M::PreVariant<AV::VariantSeed> as PreVariant<M>>::Keys;

    fn allocate(
        mobject_ref: M::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        <M::PreVariant<AV::VariantSeed> as PreVariant<M>>::allocate(
            mobject_ref,
            slot_key_generator_map,
        )
    }

    fn get<'s>(keys: &Self::Keys, storage_type_map: &'s StorageTypeMap) -> M::ResourceRef<'s> {
        <M::PreVariant<AV::VariantSeed> as PreVariant<M>>::get(keys, storage_type_map).coerce()
    }

    fn prepare(
        mobject_ref: M::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <M::PreVariant<AV::VariantSeed> as PreVariant<M>>::prepare(
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
        M::render(
            &<Self as Variant<NotPrepareRenderMobjectCategory, M>>::get(keys, storage_type_map),
            render_pass,
        );
    }
}

impl<M, AV> Variant<PrepareNotRenderMobjectCategory, M> for AV
where
    M: Mobject<MobjectCategory = PrepareNotRenderMobjectCategory> + Prepare,
    AV: AbstractVariant<M>,
    M::PreVariant<AV::VariantSeed>: PreVariant<M>,
    AV::StoreType: StoreType,
    for<'s> M::MobjectRef<'s>: Coerce<<AV::StoreType as StoreType>::KeyInput<'s>>,
    for<'s> (
        M::ResourceRepr,
        &'s wgpu::Device,
        &'s wgpu::Queue,
        wgpu::TextureFormat,
    ): Coerce<<AV::StoreType as StoreType>::Input<'s>>,
    for<'s> &'s <<AV::StoreType as StoreType>::Slot as Slot>::Value: Coerce<M::ResourceRef<'s>>,
{
    type Keys = Derivation<
        StorageKey<AV::StoreType>,
        <M::PreVariant<AV::VariantSeed> as PreVariant<M>>::Keys,
    >;

    fn allocate(
        mobject_ref: M::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        Derivation {
            extrinsic: slot_key_generator_map.allocate(mobject_ref.coerce_ref()),
            intrinsic: <M::PreVariant<AV::VariantSeed> as PreVariant<M>>::allocate(
                mobject_ref,
                slot_key_generator_map,
            ),
        }
    }

    fn get<'s>(keys: &Self::Keys, storage_type_map: &'s StorageTypeMap) -> M::ResourceRef<'s> {
        storage_type_map.read(&keys.extrinsic).coerce()
    }

    fn prepare(
        mobject_ref: M::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <M::PreVariant<AV::VariantSeed> as PreVariant<M>>::prepare(
            mobject_ref,
            &keys.intrinsic,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
        let resource_repr = M::prepare(&<M::PreVariant<AV::VariantSeed> as PreVariant<M>>::get(
            &keys.intrinsic,
            storage_type_map,
        ));
        storage_type_map.write(
            &keys.extrinsic,
            reuse,
            ((resource_repr, device, queue, format)).coerce(),
        );
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        <M::PreVariant<AV::VariantSeed> as PreVariant<M>>::render(
            &keys.intrinsic,
            storage_type_map,
            render_pass,
        );
    }
}

impl<M, AV> Variant<PrepareRenderMobjectCategory, M> for AV
where
    M: Mobject<MobjectCategory = PrepareRenderMobjectCategory> + Prepare + Render,
    AV: AbstractVariant<M>,
    M::PreVariant<AV::VariantSeed>: PreVariant<M>,
    AV::StoreType: StoreType,
    for<'s> M::MobjectRef<'s>: Coerce<<AV::StoreType as StoreType>::KeyInput<'s>>,
    for<'s> (
        M::ResourceRepr,
        &'s wgpu::Device,
        &'s wgpu::Queue,
        wgpu::TextureFormat,
    ): Coerce<<AV::StoreType as StoreType>::Input<'s>>,
    for<'s> &'s <<AV::StoreType as StoreType>::Slot as Slot>::Value: Coerce<M::ResourceRef<'s>>,
{
    type Keys = Derivation<
        StorageKey<AV::StoreType>,
        <M::PreVariant<AV::VariantSeed> as PreVariant<M>>::Keys,
    >;

    fn allocate(
        mobject_ref: M::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        Derivation {
            extrinsic: slot_key_generator_map.allocate(mobject_ref.coerce_ref()),
            intrinsic: <M::PreVariant<AV::VariantSeed> as PreVariant<M>>::allocate(
                mobject_ref,
                slot_key_generator_map,
            ),
        }
    }

    fn get<'s>(keys: &Self::Keys, storage_type_map: &'s StorageTypeMap) -> M::ResourceRef<'s> {
        storage_type_map.read(&keys.extrinsic).coerce()
    }

    fn prepare(
        mobject_ref: M::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        <M::PreVariant<AV::VariantSeed> as PreVariant<M>>::prepare(
            mobject_ref,
            &keys.intrinsic,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
        let resource_repr = M::prepare(&<M::PreVariant<AV::VariantSeed> as PreVariant<M>>::get(
            &keys.intrinsic,
            storage_type_map,
        ));
        storage_type_map.write(
            &keys.extrinsic,
            reuse,
            ((resource_repr, device, queue, format)).coerce(),
        );
    }

    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) {
        M::render(
            &<Self as Variant<PrepareRenderMobjectCategory, M>>::get(keys, storage_type_map),
            render_pass,
        );
    }
}

impl<M> Timeline<M> for StaticTimeline
where
    M: Mobject,
{
    type Observe = Arc<M>;
    type Variant = M::Variant<StaticVariantSeed>;

    fn observe(
        _clock: Clock,
        _clock_span: ClockSpan,
        _timeline: &Self,
        observe: &Self::Observe,
    ) -> Self::Observe {
        observe.clone()
    }

    fn mobject_ref<'m>(observe: &'m Self::Observe) -> M::MobjectRef<'m> {
        M::mobject_ref(observe.as_ref())
    }
}

impl<M, R> Timeline<M> for DynamicTimeline<R>
where
    M: Mobject,
    R: Refresh<M>,
{
    type Observe = M;
    type Variant = M::Variant<DynamicVariantSeed>;

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

    fn mobject_ref<'m>(observe: &'m Self::Observe) -> M::MobjectRef<'m> {
        M::mobject_ref(observe)
    }
}

// impl<M, VS> Variant<M> for VariantLeaf<M, VS>
// where
//     M: Mobject<MobjectCategory = PrepareRenderMobjectCategory> + Prepare + Render,
//     M::AbstractVariantInput<VS>: AbstractVariantInput<M>,
//     Self: StoreType,
//     for<'s> M::MobjectRef<'s>: Coerce<<Self as StoreType>::KeyInput<'s>>,
//     for<'s> (
//         M::ResourceRepr,
//         &'s wgpu::Device,
//         &'s wgpu::Queue,
//         wgpu::TextureFormat,
//     ): Coerce<<Self as StoreType>::Input<'s>>,
//     for<'s> <Self as StoreType>::Output<'s>: Coerce<M::ResourceRef<'s>>,
// {
//     type Keys = Derivation<StorageKey<Self>, <M::AbstractVariantInput<VS> as AbstractVariantInput<M>>::Keys>;

//     fn allocate(
//         mobject_ref: M::MobjectRef<'_>,
//         slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> Self::Keys {
//         Derivation {
//             extrinsic: slot_key_generator_map.allocate(mobject_ref.coerce_ref()),
//             intrinsic: <M::AbstractVariantInput<VS> as AbstractVariantInput<M>>::allocate(
//                 mobject_ref,
//                 slot_key_generator_map,
//             ),
//         }
//     }

//     fn get<'s>(keys: &Self::Keys, storage_type_map: &'s StorageTypeMap) -> M::ResourceRef<'s> {
//         storage_type_map.read(&keys.extrinsic).coerce()
//     }

//     fn prepare(
//         mobject_ref: M::MobjectRef<'_>,
//         keys: &Self::Keys,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         reuse: &mut ResourceReuseResult,
//     ) {
//         <M::AbstractVariantInput<VS> as AbstractVariantInput<M>>::prepare(
//             mobject_ref,
//             &keys.intrinsic,
//             storage_type_map,
//             device,
//             queue,
//             format,
//             reuse,
//         );
//         let resource_repr = M::prepare(&<M::AbstractVariantInput<VS> as AbstractVariantInput<M>>::get(
//             &keys.intrinsic,
//             storage_type_map,
//         ));
//         storage_type_map.write(
//             &keys.extrinsic,
//             reuse,
//             ((resource_repr, device, queue, format)).coerce(),
//         );
//     }

//     fn render(
//         keys: &Self::Keys,
//         storage_type_map: &StorageTypeMap,
//         render_pass: &mut wgpu::RenderPass,
//     ) {
//         M::render(&Self::get(keys, storage_type_map), render_pass);
//     }
// }

// impl<M, PV> Variant<M> for VariantBranch<PV>
// where
//     M: Mobject<MobjectCategory = PrepareRenderMobjectCategory> + Prepare + Render,
//     M::AbstractVariant<DynamicVariantSeed>: StoreType,
//     PV: AbstractVariantInput<M>,
//     for<'s> M::MobjectRef<'s>:
//         Coerce<<M::AbstractVariant<DynamicVariantSeed> as StoreType>::KeyInput<'s>>,
//     for<'s> (
//         M::ResourceRepr,
//         &'s wgpu::Device,
//         &'s wgpu::Queue,
//         wgpu::TextureFormat,
//     ): Coerce<<M::AbstractVariant<DynamicVariantSeed> as StoreType>::Input<'s>>,
//     for<'s> <M::AbstractVariant<DynamicVariantSeed> as StoreType>::Output<'s>:
//         Coerce<M::ResourceRef<'s>>,
// {
//     type Keys = Derivation<StorageKey<M::AbstractVariant<DynamicVariantSeed>>, PV::Keys>;

//     fn allocate(
//         mobject_ref: M::MobjectRef<'_>,
//         slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> Self::Keys {
//         Derivation {
//             extrinsic: slot_key_generator_map.allocate(mobject_ref.coerce_ref()),
//             intrinsic: PV::allocate(mobject_ref, slot_key_generator_map),
//         }
//     }

//     fn get<'s>(keys: &Self::Keys, storage_type_map: &'s StorageTypeMap) -> M::ResourceRef<'s> {
//         storage_type_map.read(&keys.extrinsic).coerce()
//     }

//     fn prepare(
//         mobject_ref: M::MobjectRef<'_>,
//         keys: &Self::Keys,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         reuse: &mut ResourceReuseResult,
//     ) {
//         PV::prepare(
//             mobject_ref,
//             &keys.intrinsic,
//             storage_type_map,
//             device,
//             queue,
//             format,
//             reuse,
//         );
//         let resource_repr = M::prepare(&PV::get(&keys.intrinsic, storage_type_map));
//         storage_type_map.write(
//             &keys.extrinsic,
//             reuse,
//             ((resource_repr, device, queue, format)).coerce(),
//         );
//     }

//     fn render(
//         keys: &Self::Keys,
//         storage_type_map: &StorageTypeMap,
//         render_pass: &mut wgpu::RenderPass,
//     ) {
//         M::render(&Self::get(keys, storage_type_map), render_pass);
//     }
// }

// impl<PV> Variant<MyMobject0> for PV
// where
//     PV: AbstractVariantInput<MyMobject0>,
// {
//     type Keys = <PV as AbstractVariantInput<MyMobject0>>::Keys;

//     fn allocate(
//         mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
//         slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> Self::Keys {
//         <PV as AbstractVariantInput<MyMobject0>>::allocate(mobject_ref, slot_key_generator_map)
//     }

//     fn get<'s>(
//         keys: &Self::Keys,
//         storage_type_map: &'s StorageTypeMap,
//     ) -> <MyMobject0 as Mobject>::ResourceRef<'s> {
//         <PV as AbstractVariantInput<MyMobject0>>::get(keys, storage_type_map)
//     }

//     fn prepare(
//         mobject_ref: <MyMobject0 as Mobject>::MobjectRef<'_>,
//         keys: &Self::Keys,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         reuse: &mut ResourceReuseResult,
//     ) {
//         <PV as AbstractVariantInput<MyMobject0>>::prepare(
//             mobject_ref,
//             keys,
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
//         <PV as AbstractVariantInput<MyMobject0>>::render(keys, storage_type_map, render_pass);
//     }
// }

// impl Timeline<MyMobject1> for StaticTimeline {
//     type Observe = Arc<MyMobject1>;
//     type Variant = <MyMobject1 as Mobject>::AbstractVariant<StaticVariantSeed>;

//     fn observe(
//         _clock: Clock,
//         _clock_span: ClockSpan,
//         _timeline: &Self,
//         observe: &Self::Observe,
//     ) -> Self::Observe {
//         observe.clone()
//     }

//     fn mobject_ref<'m>(observe: &'m Self::Observe) -> <MyMobject1 as Mobject>::MobjectRef<'m> {
//         <MyMobject1 as Mobject>::mobject_ref(observe.as_ref())
//     }
// }

// impl<R> Timeline<MyMobject1> for DynamicTimeline<R>
// where
//     R: Refresh<MyMobject1>,
// {
//     type Observe = MyMobject1;
//     type Variant = <MyMobject1 as Mobject>::AbstractVariant<DynamicVariantSeed>;

//     fn observe(
//         clock: Clock,
//         clock_span: ClockSpan,
//         timeline: &Self,
//         observe: &Self::Observe,
//     ) -> Self::Observe {
//         let mut observe = observe.clone();
//         timeline.refresh.refresh(clock, clock_span, &mut observe);
//         observe
//     }

//     fn mobject_ref<'m>(observe: &'m Self::Observe) -> <MyMobject1 as Mobject>::MobjectRef<'m> {
//         <MyMobject1 as Mobject>::mobject_ref(observe)
//     }
// }

impl<MA, MB> Timeline<MyMobject1> for MyMobject1<MA, MB>
where
    MA: Timeline<MyMobject0>,
    MB: Timeline<MyMobject0>,
{
    type Observe = MyMobject1<MA::Observe, MB::Observe>;
    type Variant = VariantBranch<MyMobject1<MA::Variant, MB::Variant>>;

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
