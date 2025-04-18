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
    type MobjectCategory: MobjectCategory<Self>;
    type MobjectRef<'m>: serde::Serialize;
    type ResourceRef<'s>;
    type ResourceRefInput<'s>;
    type PreVariantLeaves<VA>: PreVariant<Self>
    where
        VA: VariantAtom;

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

pub trait Variant<M>
where
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

pub trait MobjectCategory<M>: 'static + Send + Sync
where
    M: Mobject,
{
    type Keys<VN>
    where
        VN: VariantNode<M>;

    fn allocate<VN>(
        mobject_ref: M::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys<VN>
    where
        VN: VariantNode<M>;
    fn get<'s, VN>(
        keys: &Self::Keys<VN>,
        storage_type_map: &'s StorageTypeMap,
    ) -> M::ResourceRef<'s>
    where
        VN: VariantNode<M>;
    fn prepare<VN>(
        mobject_ref: M::MobjectRef<'_>,
        keys: &Self::Keys<VN>,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) where
        VN: VariantNode<M>;
    fn render<VN>(
        keys: &Self::Keys<VN>,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) where
        VN: VariantNode<M>;
}

pub struct NotPrepareNotRenderMobjectCategory;

pub struct NotPrepareRenderMobjectCategory;

pub struct PrepareNotRenderMobjectCategory;

pub struct PrepareRenderMobjectCategory;

impl<M> MobjectCategory<M> for NotPrepareNotRenderMobjectCategory
where
    M: Mobject,
    for<'s> M::ResourceRef<'s>: From<M::ResourceRefInput<'s>>,
{
    type Keys<VN> = <VN::PreVariant as PreVariant<M>>::Keys where VN: VariantNode<M>;

    fn allocate<VN>(
        mobject_ref: M::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys<VN>
    where
        VN: VariantNode<M>,
    {
        VN::PreVariant::allocate(mobject_ref, slot_key_generator_map)
    }

    fn get<'s, VN>(
        keys: &Self::Keys<VN>,
        storage_type_map: &'s StorageTypeMap,
    ) -> M::ResourceRef<'s>
    where
        VN: VariantNode<M>,
    {
        VN::PreVariant::get(keys, storage_type_map).into()
    }

    fn prepare<VN>(
        mobject_ref: M::MobjectRef<'_>,
        keys: &Self::Keys<VN>,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) where
        VN: VariantNode<M>,
    {
        VN::PreVariant::prepare(
            mobject_ref,
            keys,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
    }

    fn render<VN>(
        keys: &Self::Keys<VN>,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) where
        VN: VariantNode<M>,
    {
        VN::PreVariant::render(keys, storage_type_map, render_pass);
    }
}

impl<M> MobjectCategory<M> for NotPrepareRenderMobjectCategory
where
    M: Mobject + Render,
    for<'s> M::ResourceRef<'s>: From<M::ResourceRefInput<'s>>,
{
    type Keys<VN> = <VN::PreVariant as PreVariant<M>>::Keys where VN: VariantNode<M> ;

    fn allocate<VN>(
        mobject_ref: M::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys<VN>
    where
        VN: VariantNode<M>,
    {
        VN::PreVariant::allocate(mobject_ref, slot_key_generator_map)
    }

    fn get<'s, VN>(
        keys: &Self::Keys<VN>,
        storage_type_map: &'s StorageTypeMap,
    ) -> M::ResourceRef<'s>
    where
        VN: VariantNode<M>,
    {
        VN::PreVariant::get(keys, storage_type_map).into()
    }

    fn prepare<VN>(
        mobject_ref: M::MobjectRef<'_>,
        keys: &Self::Keys<VN>,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) where
        VN: VariantNode<M>,
    {
        VN::PreVariant::prepare(
            mobject_ref,
            keys,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
    }

    fn render<VN>(
        keys: &Self::Keys<VN>,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) where
        VN: VariantNode<M>,
    {
        M::render(&Self::get::<VN>(keys, storage_type_map), render_pass);
    }
}

impl<M> MobjectCategory<M> for PrepareNotRenderMobjectCategory
where
    M: Mobject + Prepare,
    for<'s> M::ResourceRef<'s>: From<&'s M::Resource>,
{
    type Keys<VN> = Derivation<
        StorageKey<VariantLeaf<M, VN::VariantAtom>>,
        <VN::PreVariant as PreVariant<M>>::Keys,
    > where VN: VariantNode<M>;

    fn allocate<VN>(
        mobject_ref: M::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys<VN>
    where
        VN: VariantNode<M>,
    {
        Derivation {
            extrinsic: slot_key_generator_map.allocate(&mobject_ref),
            intrinsic: VN::PreVariant::allocate(mobject_ref, slot_key_generator_map),
        }
    }

    fn get<'s, VN>(
        keys: &Self::Keys<VN>,
        storage_type_map: &'s StorageTypeMap,
    ) -> M::ResourceRef<'s>
    where
        VN: VariantNode<M>,
    {
        storage_type_map.read(&keys.extrinsic).into()
    }

    fn prepare<VN>(
        mobject_ref: M::MobjectRef<'_>,
        keys: &Self::Keys<VN>,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) where
        VN: VariantNode<M>,
    {
        VN::PreVariant::prepare(
            mobject_ref,
            &keys.intrinsic,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
        let resource_repr = M::prepare(&VN::PreVariant::get(&keys.intrinsic, storage_type_map));
        storage_type_map.write(
            &keys.extrinsic,
            reuse,
            (resource_repr, device, queue, format),
        );
    }

    fn render<VN>(
        keys: &Self::Keys<VN>,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) where
        VN: VariantNode<M>,
    {
        VN::PreVariant::render(&keys.intrinsic, storage_type_map, render_pass);
    }
}

impl<M> MobjectCategory<M> for PrepareRenderMobjectCategory
where
    M: Mobject + Prepare + Render,
    for<'s> M::ResourceRef<'s>: From<&'s M::Resource>,
{
    type Keys<VN> = Derivation<
        StorageKey<VariantLeaf<M, VN::VariantAtom>>,
        <VN::PreVariant as PreVariant<M>>::Keys,
    > where
        VN: VariantNode<M>;

    fn allocate<VN>(
        mobject_ref: M::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys<VN>
    where
        VN: VariantNode<M>,
    {
        Derivation {
            extrinsic: slot_key_generator_map.allocate(&mobject_ref),
            intrinsic: VN::PreVariant::allocate(mobject_ref, slot_key_generator_map),
        }
    }

    fn get<'s, VN>(
        keys: &Self::Keys<VN>,
        storage_type_map: &'s StorageTypeMap,
    ) -> M::ResourceRef<'s>
    where
        VN: VariantNode<M>,
    {
        storage_type_map.read(&keys.extrinsic).into()
    }

    fn prepare<VN>(
        mobject_ref: M::MobjectRef<'_>,
        keys: &Self::Keys<VN>,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) where
        VN: VariantNode<M>,
    {
        VN::PreVariant::prepare(
            mobject_ref,
            &keys.intrinsic,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        );
        let resource_repr = M::prepare(&VN::PreVariant::get(&keys.intrinsic, storage_type_map));
        storage_type_map.write(
            &keys.extrinsic,
            reuse,
            (resource_repr, device, queue, format),
        );
    }

    fn render<VN>(
        keys: &Self::Keys<VN>,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    ) where
        VN: VariantNode<M>,
    {
        M::render(&Self::get::<VN>(keys, storage_type_map), render_pass);
    }
}

pub trait VariantAtom: 'static + Send + Sync {
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

pub struct StaticVariantAtom;

pub struct DynamicVariantAtom;

impl VariantAtom for StaticVariantAtom {
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

impl VariantAtom for DynamicVariantAtom {
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

pub trait VariantNode<M>
where
    M: Mobject,
{
    type PreVariant: PreVariant<M>;
    type VariantAtom: VariantAtom;
}

pub struct VariantLeaf<M, VA>(M, VA);

pub struct VariantBranch<PV>(PV);

impl<M, VA> VariantNode<M> for VariantLeaf<M, VA>
where
    M: Mobject,
    VA: VariantAtom,
{
    type PreVariant = M::PreVariantLeaves<VA>;
    type VariantAtom = VA;
}

impl<M, PV> VariantNode<M> for VariantBranch<PV>
where
    M: Mobject,
    PV: PreVariant<M>,
{
    type PreVariant = PV;
    type VariantAtom = DynamicVariantAtom;
}

impl<M, VN> Variant<M> for VN
where
    M: Mobject,
    VN: VariantNode<M>,
{
    type Keys = <M::MobjectCategory as MobjectCategory<M>>::Keys<VN>;

    fn allocate(
        mobject_ref: M::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        M::MobjectCategory::allocate::<VN>(mobject_ref, slot_key_generator_map)
    }

    fn get<'s>(keys: &Self::Keys, storage_type_map: &'s StorageTypeMap) -> M::ResourceRef<'s> {
        M::MobjectCategory::get::<VN>(keys, storage_type_map)
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
        M::MobjectCategory::prepare::<VN>(
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
        M::MobjectCategory::render::<VN>(keys, storage_type_map, render_pass);
    }
}

impl<M, VA> StoreType for VariantLeaf<M, VA>
where
    M: Prepare,
    VA: VariantAtom,
{
    type KeyInput<'s> = M::MobjectRef<'s>;
    type Input<'s> = (
        M::ResourceRepr,
        &'s wgpu::Device,
        &'s wgpu::Queue,
        wgpu::TextureFormat,
    );
    type Slot = VA::Slot<M::Resource>;

    fn key<'s>(key_input: &'s Self::KeyInput<'_>) -> &'s dyn serde_traitobject::Serialize {
        VA::key(key_input)
    }

    fn insert(input: Self::Input<'_>) -> <Self::Slot as Slot>::Value {
        let (resource_repr, device, queue, format) = input;
        VA::insert::<M::ResourceRepr, M::Resource>(resource_repr, device, queue, format)
    }

    fn update(
        input: Self::Input<'_>,
        value: &mut <Self::Slot as Slot>::Value,
        reuse: &mut ResourceReuseResult,
    ) {
        let (resource_repr, device, queue, format) = input;
        VA::update::<M::ResourceRepr, M::Resource>(
            value,
            resource_repr,
            device,
            queue,
            format,
            reuse,
        )
    }
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

impl<M> Timeline<M> for StaticTimeline
where
    M: Mobject,
{
    type Observe = Arc<M>;
    type Variant = VariantLeaf<M, StaticVariantAtom>;

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
    type Variant = VariantLeaf<M, DynamicVariantAtom>;

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
    type MobjectRef<'m> = &'m Data<T>;
    type ResourceRef<'s> = &'s T;
    type ResourceRefInput<'s> = &'s T;
    type PreVariantLeaves<VA> = VariantLeaf<Self, VA>
    where
        VA: VariantAtom;

    fn mobject_ref<'m>(mobject: &'m Self) -> Self::MobjectRef<'m> {
        mobject
    }
}

impl<T> Resource<T> for Data<T>
where
    T: DataTrait,
{
    fn new(
        resource_repr: T,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> Self {
        Data(resource_repr)
    }

    fn update(
        resource: &mut Self,
        resource_repr: T,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        _reuse: &mut ResourceReuseResult,
    ) {
        resource.0 = resource_repr;
    }
}

impl<T> Prepare for Data<T>
where
    T: DataTrait,
{
    type ResourceRepr = T;
    type Resource = Data<T>;

    fn prepare(input: &<Self as Mobject>::ResourceRefInput<'_>) -> Self::ResourceRepr {
        (*input).clone()
    }
}

impl<T, VA> PreVariant<Data<T>> for VariantLeaf<Data<T>, VA>
where
    T: DataTrait,
    VA: VariantAtom,
{
    type Keys = StorageKey<Self>;

    fn allocate(
        mobject_ref: <Data<T> as Mobject>::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        slot_key_generator_map.allocate(&mobject_ref)
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <Data<T> as Mobject>::ResourceRefInput<'s> {
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
        let resource_repr = <Data<T> as Prepare>::prepare(&&**mobject_ref);
        storage_type_map.write(keys, reuse, (resource_repr, device, queue, format));
    }

    fn render(
        _keys: &Self::Keys,
        _storage_type_map: &StorageTypeMap,
        _render_pass: &mut wgpu::RenderPass,
    ) {
    }
}

// vec

impl<M> Mobject for Vec<M>
where
    M: Mobject,
{
    type MobjectCategory = NotPrepareNotRenderMobjectCategory;
    type MobjectRef<'m> = Vec<M::MobjectRef<'m>>;
    type ResourceRef<'s> = Vec<M::ResourceRef<'s>>;
    type ResourceRefInput<'s> = Vec<M::ResourceRef<'s>>;
    type PreVariantLeaves<VA> =
        Vec<VariantLeaf<M, VA>>
    where
        VA: VariantAtom;

    fn mobject_ref<'m>(mobject: &'m Self) -> Self::MobjectRef<'m> {
        mobject
            .iter()
            .map(|mobject| M::mobject_ref(mobject))
            .collect()
    }
}

impl<M, V> PreVariant<Vec<M>> for Vec<V>
where
    M: Mobject,
    V: Variant<M>,
{
    type Keys = Vec<V::Keys>;

    fn allocate(
        mobject_ref: <Vec<M> as Mobject>::MobjectRef<'_>,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        mobject_ref
            .into_iter()
            .map(|mobject_ref| V::allocate(mobject_ref, slot_key_generator_map))
            .collect()
    }

    fn get<'s>(
        keys: &Self::Keys,
        storage_type_map: &'s StorageTypeMap,
    ) -> <Vec<M> as Mobject>::ResourceRefInput<'s> {
        keys.iter()
            .map(|keys| V::get(keys, storage_type_map))
            .collect()
    }

    fn prepare(
        mobject_ref: <Vec<M> as Mobject>::MobjectRef<'_>,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) {
        mobject_ref
            .into_iter()
            .zip(keys.iter())
            .for_each(|(mobject_ref, keys)| {
                V::prepare(
                    mobject_ref,
                    keys,
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
        keys.iter()
            .for_each(|keys| V::render(keys, storage_type_map, render_pass));
    }
}

impl<M, T> Timeline<Vec<M>> for Vec<T>
where
    M: Mobject,
    T: Timeline<M>,
{
    type Observe = Vec<T::Observe>;
    type Variant = VariantBranch<Vec<T::Variant>>;

    fn observe(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &Self::Observe,
    ) -> Self::Observe {
        timeline
            .iter()
            .zip(observe.iter())
            .map(|(timeline, observe)| T::observe(clock, clock_span, timeline, observe))
            .collect()
    }

    fn mobject_ref<'m>(observe: &'m Self::Observe) -> <Vec<M> as Mobject>::MobjectRef<'m> {
        observe
            .iter()
            .map(|observe| T::mobject_ref(observe))
            .collect()
    }
}

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
    type PreVariantLeaves<VA> =
        MyMobject0<VariantLeaf<Data<f32>, VA>, VariantLeaf<Data<f32>, VA>>
    where
        VA: VariantAtom;

    fn mobject_ref<'m>(mobject: &'m Self) -> Self::MobjectRef<'m> {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::mobject_ref(&mobject.ma),
            mb: <Data<f32> as Mobject>::mobject_ref(&mobject.mb),
        }
    }
}

impl<MA, MB> PreVariant<MyMobject0> for MyMobject0<MA, MB>
where
    MA: Variant<Data<f32>>,
    MB: Variant<Data<f32>>,
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
    type PreVariantLeaves<VA> =
        MyMobject1<VariantLeaf<MyMobject0, VA>, VariantLeaf<MyMobject0, VA>>
    where
        VA: VariantAtom;

    fn mobject_ref<'m>(mobject: &'m Self) -> Self::MobjectRef<'m> {
        MyMobject1 {
            ma: <MyMobject0 as Mobject>::mobject_ref(&mobject.ma),
            mb: <MyMobject0 as Mobject>::mobject_ref(&mobject.mb),
        }
    }
}

impl<MA, MB> PreVariant<MyMobject1> for MyMobject1<MA, MB>
where
    MA: Variant<MyMobject0>,
    MB: Variant<MyMobject0>,
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
