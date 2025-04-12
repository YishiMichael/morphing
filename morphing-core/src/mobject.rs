use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;

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
    type ResourceStructure;
    type ResourceRef<'s>;
    type ResourceInputRef<'s>;

    fn static_allocate(
        mobject: &Self,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::StaticKeys;
    fn dynamic_allocate(slot_key_generator_map: &mut SlotKeyGeneratorTypeMap) -> Self::DynamicKeys;
    fn resource_ref<'s>(resource_structure: &'s Self::ResourceStructure) -> Self::ResourceRef<'s>;
    fn static_refresh(
        mobject: &Self,
        static_keys: &Self::StaticKeys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceStructure;
    fn dynamic_refresh(
        mobject: &Self,
        dynamic_keys: &Self::DynamicKeys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceStructure;
    fn static_fetch(
        static_keys: &Self::StaticKeys,
        storage_type_map: &StorageTypeMap,
    ) -> Self::ResourceStructure;
    fn dynamic_fetch(
        dynamic_keys: &Self::DynamicKeys,
        storage_type_map: &StorageTypeMap,
    ) -> Self::ResourceStructure;
    fn render_structure(resource_ref: &Self::ResourceRef<'_>, render_pass: &mut wgpu::RenderPass);
}

pub trait Timeline<M>
where
    M: Mobject,
{
    type Observe: Send + Sync;
    type Keys;

    fn allocate(
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys;
    fn observe(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &Self::Observe,
    ) -> Self::Observe;
    fn prepare(
        observe: &Self::Observe,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> M::ResourceStructure;
    fn render(
        keys: &Self::Keys,
        storage_type_map: &StorageTypeMap,
        render_pass: &mut wgpu::RenderPass,
    );
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

    fn prepare(input: &Self::ResourceInputRef<'_>) -> Self::ResourceRepr;

    fn static_prepare(
        input: &Self::ResourceInputRef<'_>,
        static_key: &StorageKey<StaticStoreType<Self, Self::Resource>>,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Arc<RwLock<Self::Resource>> {
        storage_type_map
            .update_or_insert(
                static_key,
                reuse,
                || {
                    Arc::new(RwLock::new(<Self::Resource as Resource<
                        Self::ResourceRepr,
                    >>::new(
                        Self::prepare(input), device, queue, format
                    )))
                },
                |_resource, _reuse| {},
            )
            .clone()
    }

    fn dynamic_prepare(
        input: &Self::ResourceInputRef<'_>,
        dynamic_key: &StorageKey<DynamicStoreType<Self, Self::Resource>>,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Arc<RwLock<Self::Resource>> {
        storage_type_map
            .update_or_insert(
                dynamic_key,
                reuse,
                || {
                    Arc::new(RwLock::new(<Self::Resource as Resource<
                        Self::ResourceRepr,
                    >>::new(
                        Self::prepare(input), device, queue, format
                    )))
                },
                |resource, reuse| {
                    <Self::Resource as Resource<Self::ResourceRepr>>::update(
                        &mut resource.write().unwrap(),
                        Self::prepare(input),
                        device,
                        queue,
                        format,
                        reuse,
                    )
                },
            )
            .clone()
    }
}

pub trait Render: Mobject {
    fn render(resource: &Self::ResourceRef<'_>, render_pass: &mut wgpu::RenderPass);
}

pub struct Derivation<I, E> {
    pub intrinsic: I,
    pub extrinsic: E,
}

pub struct StaticStoreType<M, R>(M, R);

impl<M, R> StoreType for StaticStoreType<M, R>
where
    M: Mobject,
    R: 'static + Send + Sync,
{
    type KeyInput = M;
    type Slot = SwapSlot<SingletonSlot<Arc<RwLock<R>>>>;
}

pub struct DynamicStoreType<M, R>(M, R);

impl<M, R> StoreType for DynamicStoreType<M, R>
where
    M: Mobject,
    R: 'static + Send + Sync,
{
    type KeyInput = ();
    type Slot = SwapSlot<MultitonSlot<Arc<RwLock<R>>>>;
}

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
    type Keys = M::StaticKeys;

    fn allocate(
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        M::static_allocate(observe.as_ref(), slot_key_generator_map)
    }

    fn observe(
        _clock: Clock,
        _clock_span: ClockSpan,
        _timeline: &Self,
        observe: &Self::Observe,
    ) -> Self::Observe {
        observe.clone()
    }

    fn prepare(
        observe: &Self::Observe,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> M::ResourceStructure {
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
        M::render_structure(
            &M::resource_ref(&M::static_fetch(keys, storage_type_map)),
            render_pass,
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
    type Keys = M::DynamicKeys;

    fn allocate(
        _observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        M::dynamic_allocate(slot_key_generator_map)
    }

    fn observe(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &Self::Observe,
    ) -> Self::Observe {
        timeline.refresh.refresh(clock, clock_span, observe.clone())
    }

    fn prepare(
        observe: &Self::Observe,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> M::ResourceStructure {
        M::dynamic_refresh(
            observe,
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
        M::render_structure(
            &M::resource_ref(&M::dynamic_fetch(keys, storage_type_map)),
            render_pass,
        );
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

impl<T> Prepare for Data<T>
where
    T: 'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    type ResourceRepr = T;
    type Resource = T;

    fn prepare(input: &<Self as Mobject>::ResourceInputRef<'_>) -> Self::ResourceRepr {
        (*input).clone()
    }
}

impl<T> Mobject for Data<T>
where
    T: 'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    type StaticKeys = StorageKey<StaticStoreType<Data<T>, T>>;
    type DynamicKeys = StorageKey<DynamicStoreType<Data<T>, T>>;
    type ResourceStructure = Arc<RwLock<T>>;
    type ResourceRef<'s> = RwLockReadGuard<'s, T>;
    type ResourceInputRef<'s> = &'s T;

    fn static_allocate(
        mobject: &Self,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::StaticKeys {
        slot_key_generator_map.allocate(mobject)
    }

    fn dynamic_allocate(slot_key_generator_map: &mut SlotKeyGeneratorTypeMap) -> Self::DynamicKeys {
        slot_key_generator_map.allocate(&())
    }

    fn resource_ref<'s>(resource_structure: &'s Self::ResourceStructure) -> Self::ResourceRef<'s> {
        resource_structure.read().unwrap()
    }

    fn static_refresh(
        mobject: &Self,
        static_keys: &Self::StaticKeys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceStructure {
        Self::static_prepare(
            &&**mobject,
            static_keys,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        )
    }

    fn dynamic_refresh(
        mobject: &Self,
        dynamic_keys: &Self::DynamicKeys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceStructure {
        Self::dynamic_prepare(
            &&**mobject,
            dynamic_keys,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        )
    }

    fn static_fetch(
        static_keys: &Self::StaticKeys,
        storage_type_map: &StorageTypeMap,
    ) -> Self::ResourceStructure {
        storage_type_map.get_and_unwrap(static_keys).clone()
    }

    fn dynamic_fetch(
        dynamic_keys: &Self::DynamicKeys,
        storage_type_map: &StorageTypeMap,
    ) -> Self::ResourceStructure {
        storage_type_map.get_and_unwrap(dynamic_keys).clone()
    }

    fn render_structure(
        _resource_ref: &Self::ResourceRef<'_>,
        _render_pass: &mut wgpu::RenderPass,
    ) {
    }
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
    type ResourceStructure = MyMobject0<
        <Data<f32> as Mobject>::ResourceStructure,
        <Data<f32> as Mobject>::ResourceStructure,
    >;
    type ResourceRef<'s> = MyMobject0<
        <Data<f32> as Mobject>::ResourceRef<'s>,
        <Data<f32> as Mobject>::ResourceRef<'s>,
    >;
    type ResourceInputRef<'s> = MyMobject0<
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

    fn resource_ref<'s>(resource_structure: &'s Self::ResourceStructure) -> Self::ResourceRef<'s> {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::resource_ref(&resource_structure.ma),
            mb: <Data<f32> as Mobject>::resource_ref(&resource_structure.mb),
        }
    }

    fn static_refresh(
        mobject: &Self,
        static_keys: &Self::StaticKeys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceStructure {
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

    fn dynamic_refresh(
        mobject: &Self,
        dynamic_keys: &Self::DynamicKeys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceStructure {
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

    fn static_fetch(
        static_keys: &Self::StaticKeys,
        storage_type_map: &StorageTypeMap,
    ) -> Self::ResourceStructure {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::static_fetch(&static_keys.ma, storage_type_map),
            mb: <Data<f32> as Mobject>::static_fetch(&static_keys.mb, storage_type_map),
        }
    }

    fn dynamic_fetch(
        dynamic_keys: &Self::DynamicKeys,
        storage_type_map: &StorageTypeMap,
    ) -> Self::ResourceStructure {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::dynamic_fetch(&dynamic_keys.ma, storage_type_map),
            mb: <Data<f32> as Mobject>::dynamic_fetch(&dynamic_keys.mb, storage_type_map),
        }
    }

    fn render_structure(resource_ref: &Self::ResourceRef<'_>, render_pass: &mut wgpu::RenderPass) {
        <Data<f32> as Mobject>::render_structure(&resource_ref.ma, render_pass);
        <Data<f32> as Mobject>::render_structure(&resource_ref.mb, render_pass);
    }
}

impl<MA, MB> Timeline<MyMobject0> for MyMobject0<MA, MB>
where
    MA: Timeline<Data<f32>>,
    MB: Timeline<Data<f32>>,
{
    type Observe = MyMobject0<MA::Observe, MB::Observe>;
    type Keys = MyMobject0<MA::Keys, MB::Keys>;

    fn allocate(
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Keys {
        MyMobject0 {
            ma: MA::allocate(&observe.ma, slot_key_generator_map),
            mb: MB::allocate(&observe.mb, slot_key_generator_map),
        }
    }

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

    fn prepare(
        observe: &Self::Observe,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> <MyMobject0 as Mobject>::ResourceStructure {
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

    fn prepare(input: &Self::ResourceInputRef<'_>) -> Self::ResourceRepr {
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
    type StaticKeys = Derivation<
        MyMobject1<<MyMobject0 as Mobject>::StaticKeys, <MyMobject0 as Mobject>::StaticKeys>,
        StorageKey<StaticStoreType<MyMobject1, <MyMobject1 as Prepare>::Resource>>,
    >;
    type DynamicKeys = Derivation<
        MyMobject1<<MyMobject0 as Mobject>::DynamicKeys, <MyMobject0 as Mobject>::DynamicKeys>,
        StorageKey<DynamicStoreType<MyMobject1, <MyMobject1 as Prepare>::Resource>>,
    >;
    type ResourceStructure = Arc<RwLock<<MyMobject1 as Prepare>::Resource>>;
    type ResourceRef<'s> = RwLockReadGuard<'s, <MyMobject1 as Prepare>::Resource>;
    type ResourceInputRef<'s> = MyMobject1<
        <MyMobject0 as Mobject>::ResourceRef<'s>,
        <MyMobject0 as Mobject>::ResourceRef<'s>,
    >;

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

    fn resource_ref<'s>(resource_structure: &'s Self::ResourceStructure) -> Self::ResourceRef<'s> {
        resource_structure.read().unwrap()
    }

    fn static_refresh(
        mobject: &Self,
        static_keys: &Self::StaticKeys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceStructure {
        Self::static_prepare(
            &MyMobject1 {
                ma: <MyMobject0 as Mobject>::resource_ref(
                    &<MyMobject0 as Mobject>::static_refresh(
                        &mobject.ma,
                        &static_keys.intrinsic.ma,
                        storage_type_map,
                        device,
                        queue,
                        format,
                        reuse,
                    ),
                ),
                mb: <MyMobject0 as Mobject>::resource_ref(
                    &<MyMobject0 as Mobject>::static_refresh(
                        &mobject.mb,
                        &static_keys.intrinsic.mb,
                        storage_type_map,
                        device,
                        queue,
                        format,
                        reuse,
                    ),
                ),
            },
            &static_keys.extrinsic,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        )
    }

    fn dynamic_refresh(
        mobject: &Self,
        dynamic_keys: &Self::DynamicKeys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> Self::ResourceStructure {
        Self::dynamic_prepare(
            &MyMobject1 {
                ma: <MyMobject0 as Mobject>::resource_ref(
                    &<MyMobject0 as Mobject>::dynamic_refresh(
                        &mobject.ma,
                        &dynamic_keys.intrinsic.ma,
                        storage_type_map,
                        device,
                        queue,
                        format,
                        reuse,
                    ),
                ),
                mb: <MyMobject0 as Mobject>::resource_ref(
                    &<MyMobject0 as Mobject>::dynamic_refresh(
                        &mobject.mb,
                        &dynamic_keys.intrinsic.mb,
                        storage_type_map,
                        device,
                        queue,
                        format,
                        reuse,
                    ),
                ),
            },
            &dynamic_keys.extrinsic,
            storage_type_map,
            device,
            queue,
            format,
            reuse,
        )
    }

    fn static_fetch(
        static_keys: &Self::StaticKeys,
        storage_type_map: &StorageTypeMap,
    ) -> Self::ResourceStructure {
        storage_type_map
            .get_and_unwrap(&static_keys.extrinsic)
            .clone()
    }

    fn dynamic_fetch(
        dynamic_keys: &Self::DynamicKeys,
        storage_type_map: &StorageTypeMap,
    ) -> Self::ResourceStructure {
        storage_type_map
            .get_and_unwrap(&dynamic_keys.extrinsic)
            .clone()
    }

    fn render_structure(resource_ref: &Self::ResourceRef<'_>, render_pass: &mut wgpu::RenderPass) {
        Self::render(resource_ref, render_pass);
    }
}

impl<MA, MB> Timeline<MyMobject1> for MyMobject1<MA, MB>
where
    MA: Timeline<MyMobject0>,
    MB: Timeline<MyMobject0>,
{
    type Observe = MyMobject1<MA::Observe, MB::Observe>;
    type Keys = Derivation<
        MyMobject1<MA::Keys, MB::Keys>,
        StorageKey<DynamicStoreType<MyMobject1, <MyMobject1 as Prepare>::Resource>>,
    >;

    fn allocate(
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

    fn prepare(
        observe: &Self::Observe,
        keys: &Self::Keys,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        reuse: &mut ResourceReuseResult,
    ) -> <MyMobject1 as Mobject>::ResourceStructure {
        <MyMobject1 as Prepare>::dynamic_prepare(
            &MyMobject1 {
                ma: <MyMobject0 as Mobject>::resource_ref(&MA::prepare(
                    &observe.ma,
                    &keys.intrinsic.ma,
                    storage_type_map,
                    device,
                    queue,
                    format,
                    reuse,
                )),
                mb: <MyMobject0 as Mobject>::resource_ref(&MB::prepare(
                    &observe.mb,
                    &keys.intrinsic.mb,
                    storage_type_map,
                    device,
                    queue,
                    format,
                    reuse,
                )),
            },
            &keys.extrinsic,
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
        <MyMobject1 as Mobject>::render_structure(
            &<MyMobject1 as Mobject>::resource_ref(
                &storage_type_map.get_and_unwrap(&keys.extrinsic),
            ),
            render_pass,
        );
    }
}
