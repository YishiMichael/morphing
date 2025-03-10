use std::any::Any;
use std::any::TypeId;
use std::cell::RefCell;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use super::config::Config;
use super::stage::World;
use super::storable::SharableSlot;
use super::storable::Slot;
use super::storable::SlotKeyGenerator;
use super::storable::SlotKeyGeneratorTypeMap;
use super::storable::Storable;
use super::storable::StorageKey;
use super::storable::StorageTypeMap;
use super::storable::SwapSlot;
use super::storable::VecSlot;
use super::traits::Construct;
use super::traits::IncreasingRate;
use super::traits::Mobject;
use super::traits::MobjectBuilder;
use super::traits::Rate;
use super::traits::StorableKeyFn;
use super::traits::Update;

// #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
// pub struct StaticAnimationId(pub u64);

// #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
// pub struct DynamicAnimationId(pub u64);

// impl AnimationId {
//     fn new(animation: &serde_traitobject::Arc<dyn Animation>) -> Self {
//         // Hash `Arc<T>` instead of `T`.
//         // Presentation maps inside `storage` are identified only by `T::Presentation` type, without `T`.
//         Self(seahash::hash(
//             &ron::ser::to_string(animation).unwrap().into_bytes(),
//         ))
//     }

//     fn prepare_presentation<P, C, F>(
//         self,
//         storage: &mut iced::widget::shader::Storage,
//         device: &wgpu::Device,
//         presentation_constructor: C,
//         prepare: F,
//     ) where
//         P: 'static + Send,
//         C: FnOnce(&wgpu::Device) -> P,
//         F: FnOnce(&mut P),
//     {
//         prepare(
//             &mut match storage.get_mut::<dashmap::DashMap<Self, P>>() {
//                 Some(presentation_map) => presentation_map,
//                 None => {
//                     storage.store::<dashmap::DashMap<Self, P>>(dashmap::DashMap::new());
//                     storage.get_mut::<dashmap::DashMap<Self, P>>().unwrap()
//                 }
//             }
//             .entry(self)
//             .or_insert_with(|| presentation_constructor(device)),
//         )
//     }

//     fn render_presentation<P, F>(self, storage: &iced::widget::shader::Storage, render: F)
//     where
//         P: 'static,
//         F: FnOnce(&P),
//     {
//         render(
//             &storage
//                 .get::<dashmap::DashMap<Self, P>>()
//                 .unwrap()
//                 .get(&self)
//                 .unwrap(),
//         )
//     }
// }

pub type Time = f32;

pub enum PresentationKey<SKF, MP>
where
    SKF: StorableKeyFn,
    MP: 'static + Send + Sync,
{
    Static(
        Arc<StorageKey<
            (TypeId, SKF::Output),
            <<SwapSlot<SharableSlot<MP>> as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        >>,
    ),
    Dynamic(
        Arc<StorageKey<
            (TypeId, SKF::Output, TypeId, SKF::Output),
            <<SwapSlot<VecSlot<MP>> as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        >>,
    ),
}

impl<SKF, MP> PresentationKey<SKF, MP>
where
    SKF: StorableKeyFn,
    MP: 'static + Send + Sync,
{
    fn read<'mp>(&self, storage_type_map: &'mp StorageTypeMap) -> &'mp MP {
        match self {
            Self::Static(key) => storage_type_map
                .get::<_, SwapSlot<SharableSlot<MP>>>(key)
                .as_ref()
                .unwrap(),
            Self::Dynamic(key) => storage_type_map
                .get::<_, SwapSlot<VecSlot<MP>>>(key)
                .as_ref()
                .unwrap(),
        }
    }
}

trait Animation:
    'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize + Storable
{
    type MobjectPresentation: Send + Sync;
    // type SerializableKeyFn: StorableKeyFn;
    // type SerdeKey: 'static + Eq + Hash + Send + Sync;
    // type MobjectPresentationStorage: PresentationStorage;

    // fn serde_key(&self) -> Self::SerdeKey;
    fn init_presentation(&self, device: &wgpu::Device) -> <Self::Slot as Slot>::Value;
    fn erase_presentation_key<SKF>(
        &self,
        mobject_presentation_key: Arc<
            StorageKey<
                Self::StorableKey<SKF>,
                <<Self::Slot as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
            >,
        >,
    ) -> PresentationKey<SKF, Self::MobjectPresentation>
    where
        SKF: StorableKeyFn;
    fn prepare_presentation(
        &self,
        time: Time,
        time_interval: Range<Time>,
        mobject_presentation: &mut <Self::Slot as Slot>::Value,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    );
    // fn erased(&self) -> Box<dyn Animation<PresentationStorage = Self::PresentationStorage>>;

    // let slot_map = storage
    //     .entry::<AllocatedAnimation<Self::SerdeKey, Self::MobjectPresentationStorage>>()
    //     .or_insert_with(HashMap::new);
    // let serde_key = self.serde_key();
    // let slot_id = slot_map
    //     .entry(serde_key.clone())
    //     .or_insert_with(Self::MobjectPresentationStorage::new)
    //     .allocate();
    // AllocatedAnimation {
    //     serde_key,
    //     slot_id,
    //     animation: self as Box<
    //         dyn Animation<
    //             SerdeKey = Self::SerdeKey,
    //             MobjectPresentationStorage = Self::MobjectPresentationStorage,
    //         >,
    //     >,
    // }
}

// struct StaticAnimationKeyFn<M, MKF>(PhantomData<(M, MKF)>);

// impl<M, MKF> Default for StaticAnimationKeyFn<M, MKF> {
//     fn default() -> Self {
//         Self(PhantomData)
//     }
// }

// impl<M, MKF> KeyFn for StaticAnimationKeyFn<M, MKF>
// where
//     M: Mobject,
//     MKF: MobjectKeyFn,
// {
//     type Input = M;
//     type Output = MKF::Output;

//     fn eval_key(&self, input: &Self::Input) -> Self::Output {
//         MKF::eval_key(input)
//     }
// }

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct StaticAnimation<M> {
    // mobject_serde_key: serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    // slot_id: SI,
    // time_interval: Range<Time>,
    mobject: Arc<M>,
    // phantom: PhantomData<SKF>,
}

impl<M> Storable for StaticAnimation<M>
where
    M: Mobject,
{
    type StorableKey<SKF> = (TypeId, SKF::Output) where SKF: StorableKeyFn;
    type Slot = SwapSlot<SharableSlot<M::MobjectPresentation>>;

    fn key<SKF>(&self) -> Self::StorableKey<SKF>
    where
        SKF: StorableKeyFn,
    {
        (self.mobject.type_id(), SKF::eval_key(&self.mobject))
    }

    // type PresentationStorage = MapStorage<
    //     serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    //     SwapStorage<ReadStorage<M::MobjectPresentation>>,
    // >;

    // fn storage_id_input(
    //     &self,
    // ) -> <Self::PresentationStorage as PresentationStorage>::StorageIdInput {
    //     (
    //         serde_hashkey::to_key_with_ordered_float(self.mobject.as_ref()).unwrap(),
    //         (),
    //     )
    // }
}

impl<M> Animation for StaticAnimation<M>
where
    M: Mobject,
{
    type MobjectPresentation = M::MobjectPresentation;
    // type SerdeKey = serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>;
    // type MobjectPresentationStorage = ;

    // fn serde_key(&self) -> Self::SerdeKey {
    //     serde_hashkey::to_key_with_ordered_float(self.mobject.as_ref()).unwrap()
    // }
    fn init_presentation(&self, device: &wgpu::Device) -> <Self::Slot as Slot>::Value {
        Arc::new(self.mobject.presentation(device))
    }

    fn erase_presentation_key<SKF>(
        &self,
        mobject_presentation_key: Arc<
            StorageKey<
                Self::StorableKey<SKF>,
                <<Self::Slot as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
            >,
        >,
    ) -> PresentationKey<SKF, Self::MobjectPresentation>
    where
        SKF: StorableKeyFn,
    {
        PresentationKey::Static(mobject_presentation_key)
    }

    fn prepare_presentation(
        &self,
        _time: Time,
        _time_interval: Range<Time>,
        _mobject_presentation: &mut <Self::Slot as Slot>::Value,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) {
    }

    // fn erased(&self) -> Box<dyn Animation<PresentationStorage = Self::PresentationStorage>> {
    //     Box::new(StaticAnimation {
    //         mobject: self.mobject.clone(),
    //     })
    // }

    // fn prepare(
    //     &self,
    //     _time: Time,
    //     _time_interval: Range<Time>,
    //     storage: &mut S,
    //     device: &wgpu::Device,
    //     _queue: &wgpu::Queue,
    //     _format: wgpu::TextureFormat,
    //     // activate: Option<()>,
    // ) {
    //     storage
    //         .static_set(&self.id)
    //         .get_or_insert_with(|| Arc::new(self.mobject.presentation(device)));
    // }

    // fn render(&self, storage: &S, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
    //     storage.static_get(&self.id).map(|mobject_presentation| {
    //         mobject_presentation.render(encoder, target);
    //     });
    // }
}

// struct DynamicAnimationKeyFn<TM, M, U, MKF, UKF>(PhantomData<(TM, M, U, MKF, UKF)>);

// impl<TM, M, U, MKF, UKF> Default for DynamicAnimationKeyFn<TM, M, U, MKF, UKF> {
//     fn default() -> Self {
//         Self(PhantomData)
//     }
// }

// impl<TM, M, U, MKF, UKF> KeyFn for DynamicAnimationKeyFn<TM, M, U, MKF, UKF>
// where
//     TM: TimeMetric,
//     M: Mobject,
//     U: Update<TM, M>,
//     MKF: MobjectKeyFn,
//     UKF: UpdateKeyFn,
// {
//     type Input = (M, U);
//     type Output = (MKF::Output, UKF::Output);

//     fn eval_key(&self, input: &Self::Input) -> Self::Output {
//         (MKF::eval_key(input.0), UKF::eval_key(input.1))
//     }
// }

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct DynamicAnimation<M, TE, U> {
    // mobject_serde_key: serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    // update_serde_key: serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    // slot_id: SI,
    // time_interval: Range<Time>,
    mobject: Arc<M>,
    time_eval: Arc<TE>,
    update: Arc<U>,
    // phantom: PhantomData<SKF>,
}

impl<M, TE, U> Storable for DynamicAnimation<M, TE, U>
where
    M: Mobject,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, M>,
{
    type StorableKey<SKF> = (TypeId, SKF::Output, TypeId, SKF::Output) where SKF: StorableKeyFn;
    type Slot = SwapSlot<VecSlot<M::MobjectPresentation>>;

    fn key<SKF>(&self) -> Self::StorableKey<SKF>
    where
        SKF: StorableKeyFn,
    {
        (
            self.mobject.type_id(),
            SKF::eval_key(&self.mobject),
            self.update.type_id(),
            SKF::eval_key(&self.update),
        )
    }

    // type PresentationStorage = MapStorage<
    //     (
    //         serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    //         serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    //     ),
    //     SwapStorage<ReadWriteStorage<M::MobjectPresentation>>,
    // >;

    // fn storage_id_input(
    //     &self,
    // ) -> <Self::PresentationStorage as PresentationStorage>::StorageIdInput {
    //     (
    //         (
    //             serde_hashkey::to_key_with_ordered_float(self.mobject.as_ref()).unwrap(),
    //             serde_hashkey::to_key_with_ordered_float(self.update.as_ref()).unwrap(),
    //         ),
    //         (),
    //     )
    // }
}

impl<M, TE, U> Animation for DynamicAnimation<M, TE, U>
where
    M: Mobject,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, M>,
{
    // type SerdeKey = (
    //     serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    //     serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    // );
    // type MobjectPresentationStorage = MapStorage<
    //     (
    //         serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    //         serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    //     ),
    //     SwapStorage<ReadWriteStorage<M::MobjectPresentation>>,
    // >;

    // fn serde_key(&self) -> Self::SerdeKey {
    //     (
    //         serde_hashkey::to_key_with_ordered_float(self.mobject.as_ref()).unwrap(),
    //         serde_hashkey::to_key_with_ordered_float(self.update.as_ref()).unwrap(),
    //     )
    // }
    type MobjectPresentation = M::MobjectPresentation;

    fn init_presentation(&self, device: &wgpu::Device) -> <Self::Slot as Slot>::Value {
        self.mobject.presentation(device)
    }

    fn erase_presentation_key<SKF>(
        &self,
        mobject_presentation_key: Arc<
            StorageKey<
                Self::StorableKey<SKF>,
                <<Self::Slot as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
            >,
        >,
    ) -> PresentationKey<SKF, Self::MobjectPresentation>
    where
        SKF: StorableKeyFn,
    {
        PresentationKey::Dynamic(mobject_presentation_key)
    }

    fn prepare_presentation(
        &self,
        time: Time,
        time_interval: Range<Time>,
        mobject_presentation: &mut <Self::Slot as Slot>::Value,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        self.update.prepare_presentation(
            self.time_eval.time_eval(time, time_interval),
            &self.mobject,
            mobject_presentation,
            device,
            queue,
            format,
        );
    }

    // fn erased(&self) -> Box<dyn Animation<PresentationStorage = Self::PresentationStorage>> {
    //     Box::new(DynamicAnimation {
    //         mobject: self.mobject.clone(),
    //         time_eval: self.time_eval.clone(),
    //         update: self.update.clone(),
    //     })
    // }

    // fn render(&self, storage: &S, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
    //     storage.dynamic_get(&self.id).map(|mobject_presentation| {
    //         mobject_presentation.render(encoder, target);
    //     });
    // }
}

// #[derive(Debug)]
// struct AllocatedAnimation<SK, MPS>
// where
//     SK: SerdeKey,
//     MPS: MobjectPresentationStorage,
// {
//     // time_interval: Range<Time>,
//     serde_key: SK,        //T::SerdeKey,
//     slot_id: MPS::SlotId, //<T::MobjectPresentationStorage as MobjectPresentationStorage>::SlotId,
//     animation: Box<dyn Animation<SerdeKey = SK, MobjectPresentationStorage = MPS>>,
// }

// impl<SK, MPS> typemap_rev::TypeMapKey for AllocatedAnimation<SK, MPS>
// where
//     SK: SerdeKey,
//     MPS: MobjectPresentationStorage,
// {
//     type Value = HashMap<SK, MPS>;
// }

// impl<SK, MPS> AllocatedAnimation<SK, MPS>
// where
//     SK: SerdeKey,
//     MPS: MobjectPresentationStorage,
// {
//     fn get_prepare_ref<'a, F>(
//         &'a self,
//         storage: &'a mut typemap_rev::TypeMap,
//         f: F,
//     ) -> MPS::PrepareRef<'a>
//     where
//         F: FnOnce() -> MPS::MobjectPresentation,
//     {
//         storage
//             .get_mut::<Self>()
//             .unwrap()
//             .get_mut(&self.serde_key)
//             .unwrap()
//             .get_prepare_ref(&self.slot_id, f)
//     }

//     fn get_render_ref<'a>(&'a self, storage: &'a mut typemap_rev::TypeMap) -> MPS::RenderRef<'a> {
//         storage
//             .get::<Self>()
//             .unwrap()
//             .get(&self.serde_key)
//             .unwrap()
//             .get_render_ref(&self.slot_id)
//     }
// }

// #[derive(Deserialize, Serialize)]
// struct Preallocated<T>
// where
//     A: Animation,
// {
//     animation: T,
// }

pub trait AnimationErasure<SKF>:
    serde_traitobject::Deserialize + serde_traitobject::Serialize
{
    type MobjectPresentation;
    // type SerializableKeyFn;

    fn allocate(
        self: Box<Self>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Box<dyn AnimationAllocatedErasure<SKF, MobjectPresentation = Self::MobjectPresentation>>;
}

pub trait AnimationAllocatedErasure<SKF>
where
    SKF: StorableKeyFn,
{
    type MobjectPresentation: Send + Sync;

    fn prepare(
        &self,
        time: Time,
        time_interval: Range<f32>,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> PresentationKey<SKF, Self::MobjectPresentation>;
}

struct AnimationAllocated<SKF, A>
where
    SKF: StorableKeyFn,
    A: Animation,
{
    storage_key: Arc<
        StorageKey<
            A::StorableKey<SKF>,
            <<A::Slot as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        >,
    >,
    animation: Box<A>,
}

impl<SKF, A> AnimationErasure<SKF> for A
where
    SKF: StorableKeyFn,
    A: Animation,
{
    type MobjectPresentation = A::MobjectPresentation;
    // type SerializableKeyFn = T::SerializableKeyFn;

    fn allocate(
        self: Box<Self>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Box<dyn AnimationAllocatedErasure<SKF, MobjectPresentation = Self::MobjectPresentation>>
    {
        Box::new(AnimationAllocated {
            storage_key: Arc::new(slot_key_generator_type_map.allocate(&self)),
            animation: self,
        })
    }
}

impl<SKF, A> AnimationAllocatedErasure<SKF> for AnimationAllocated<SKF, A>
where
    SKF: StorableKeyFn,
    A: Animation,
{
    // fn fetch_presentation(
    //     &self,
    //     storage_type_map: &StorageTypeMap,
    // ) -> PresentationCell<Self::MobjectPresentation> {
    //     self.storable_primitive()
    //         .fetch_presentation(storage_type_map.get_ref(self).as_ref().unwrap())
    // }

    type MobjectPresentation = A::MobjectPresentation;
    // type SerializableKeyFn = T::SerializableKeyFn;

    fn prepare(
        &self,
        time: Time,
        time_interval: Range<f32>,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> PresentationKey<SKF, Self::MobjectPresentation> {
        let mobject_presentation = storage_type_map
            .get_or_insert_with::<_, A::Slot, _>(&self.storage_key, || {
                self.animation.init_presentation(device)
            });
        self.animation.prepare_presentation(
            time,
            time_interval,
            mobject_presentation,
            device,
            queue,
            format,
        );
        self.animation
            .erase_presentation_key(self.storage_key.clone())
    }
}

pub(crate) enum Node<V> {
    None,
    Singleton(V),
    Multiton(Vec<V>),
}

impl<V> Node<V> {
    pub(crate) fn map_ref<F, FO>(&self, f: F) -> Node<FO>
    where
        F: FnMut(&V) -> FO,
    {
        match self {
            Self::None => Self::None,
            Self::Singleton(v) => Self::Singleton(f(v)),
            Self::Multiton(vs) => Self::Multiton(vs.iter().map(f).collect()),
        }
    }

    pub(crate) fn map<F, FO>(self, f: F) -> Node<FO>
    where
        F: FnMut(V) -> FO,
    {
        match self {
            Self::None => Self::None,
            Self::Singleton(v) => Self::Singleton(f(v)),
            Self::Multiton(vs) => Self::Multiton(vs.into_iter().map(f).collect()),
        }
    }
}

impl<V> IntoIterator for Node<V> {
    type Item = V;
    type IntoIter = <Vec<V> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::None => vec![],
            Self::Singleton(v) => vec![v],
            Self::Multiton(vs) => vs,
        }
        .into_iter()
    }
}

// trait Structure {}

// pub trait Node<SKF>
// where
//     SKF: StorableKeyFn,
// {
//     // type SerializableKeyFn: SerializableKeyFn;
//     type Attachment<'s, W>
//     where
//         Self: 's,
//         W: 's + WorldErasure<SKF>;

//     fn new() -> Self;
//     fn merge<TE>(
//         &self,
//         child: Self,
//         time_interval: &Range<Time>,
//         time_eval: &TE,
//         child_time_interval: &Range<Time>,
//     ) where
//         TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>;
//     fn attachment<'s, W>(&'s self, context: &'s Context<SKF, W>) -> Self::Attachment<'s, W>
//     where
//         W: WorldErasure<SKF>;
// }

// pub trait ChannelErasure<SKF>: Node<SKF>
// where
//     SKF: StorableKeyFn,
// {
//     type MobjectPresentation;

//     fn allocate(
//         self: Self,
//         slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> ChannelAllocated<SKF, Self::MobjectPresentation>;
// }

// pub trait ChannelAllocatedErasure<SKF>
// where
//     SKF: StorableKeyFn,
// {
//     type MobjectPresentation: Send + Sync;

//     fn prepare(
//         &self,
//         time: Time,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> Vec<PresentationKey<SKF, Self::MobjectPresentation>>;
// }

// pub trait LayerErasure<SKF>: Node<SKF>
// where
//     SKF: StorableKeyFn,
// {
//     type Allocated: LayerAllocatedErasure<SKF>;

//     fn allocate(
//         self: Self,
//         slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> Self::Allocated;
// }

// pub trait LayerAllocatedErasure<SKF> {
//     type Render: Render;

//     fn prepare(
//         &self,
//         time: Time,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> Self::Render;
// }

// pub trait WorldErasure<SKF>: Node<SKF>
// where
//     SKF: StorableKeyFn,
// {
//     type Allocated: WorldAllocatedErasure<SKF>;

//     fn allocate(
//         self: Self,
//         slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> Self::Allocated;
// }

// pub trait WorldAllocatedErasure<SKF> {
//     fn prepare(
//         &self,
//         time: Time,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> Vec<Box<dyn Render>>;
// }

// struct Channel<SKF, MP>(
//     RefCell<
//         Vec<(
//             Range<Time>,
//             Box<dyn AnimationErasure<SKF, MobjectPresentation = MP>>,
//         )>,
//     >,
// );

// struct ChannelAttachment<'c, SKF, W, MP>
// where
//     SKF: StorableKeyFn,
//     W: WorldErasure<SKF>,
// {
//     context: &'c Context<'c, SKF, W>,
//     channel: &'c Channel<SKF, MP>,
// }

// impl<SKF, MP> Node<SKF> for Channel<SKF, MP>
// where
//     SKF: StorableKeyFn,
// {
//     type Attachment<'c, W> = ChannelAttachment<'c, SKF, W, MP>
//     where
//         Self: 'c, W: 'c + WorldErasure<SKF>;

//     fn new() -> Self {
//         Self(RefCell::new(Vec::new()))
//     }

//     fn merge<TE>(
//         &self,
//         child: Self,
//         time_interval: &Range<Time>,
//         time_eval: &TE,
//         child_time_interval: &Range<Time>,
//     ) where
//         TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
//     {
//         self.0
//             .borrow_mut()
//             .extend(
//                 child
//                     .0
//                     .into_inner()
//                     .into_iter()
//                     .map(|(entry_time_interval, animation)| {
//                         (
//                             time_interval.start
//                                 + (time_interval.end - time_interval.start)
//                                     * *time_eval.time_eval(
//                                         entry_time_interval.start,
//                                         child_time_interval.clone(),
//                                     )
//                                 ..time_interval.start
//                                     + (time_interval.end - time_interval.start)
//                                         * *time_eval.time_eval(
//                                             entry_time_interval.end,
//                                             child_time_interval.clone(),
//                                         ),
//                             animation,
//                         )
//                     }),
//             );
//     }

//     fn attachment<'c, W>(&'c self, context: &'c Context<'c, SKF, W>) -> Self::Attachment<'c, W>
//     where
//         W: WorldErasure<SKF>,
//     {
//         ChannelAttachment {
//             context,
//             channel: self,
//         }
//     }
// }

// impl<SKF, MP> ChannelErasure<SKF> for Channel<SKF, MP>
// where
//     SKF: StorableKeyFn,
// {
//     type MobjectPresentation = MP;

//     fn allocate(
//         self: Self,
//         slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> ChannelAllocated<SKF, Self::MobjectPresentation> {
//         ChannelAllocated(
//             self.0
//                 .into_inner()
//                 .into_iter()
//                 .map(|(time_interval, animation)| {
//                     (
//                         time_interval,
//                         animation.allocate(slot_key_generator_type_map),
//                     )
//                 })
//                 .collect(),
//         )
//     }
// }

// struct ChannelAllocated<SKF, MP>(
//     Vec<(
//         Range<Time>,
//         Box<dyn AnimationAllocatedErasure<SKF, MobjectPresentation = MP>>,
//     )>,
// );

// impl<SKF, MP> ChannelAllocatedErasure<SKF> for ChannelAllocated<SKF, MP>
// where
//     SKF: StorableKeyFn,
//     MP: Send + Sync,
// {
//     type MobjectPresentation = MP;

//     fn prepare(
//         &self,
//         time: Time,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> Vec<PresentationKey<SKF, Self::MobjectPresentation>> {
//         self.0
//             .iter()
//             .filter_map(|(time_interval, animation)| {
//                 time_interval.contains(&time).then(|| {
//                     animation.prepare(
//                         time,
//                         time_interval.clone(),
//                         storage_type_map,
//                         device,
//                         queue,
//                         format,
//                     )
//                 })
//             })
//             .collect()
//     }
// }

// impl<SKF, W, MP> ChannelAttachment<'_, SKF, W, MP>
// where
//     SKF: StorableKeyFn,
//     W: WorldErasure<SKF>,
// {
//     fn push<A>(&self, time_interval: Range<Rc<Time>>, animation: A)
//     where
//         A: Animation<MobjectPresentation = MP>,
//     {
//         if !Rc::ptr_eq(&time_interval.start, &time_interval.end) {
//             self.channel.0.borrow_mut().push((
//                 *time_interval.start..*time_interval.end,
//                 Box::new(animation),
//             ))
//         }
//     }

//     fn start<M, TS>(&self, mobject: Arc<M>, animation_state: TS) -> Alive<SKF, W, M, TS>
//     where
//         M: Mobject<MobjectPresentation = MP>,
//         TS: AnimationState<SKF, W, M>,
//     {
//         Alive {
//             channel_attachment: self,
//             // index: usize,
//             spawn_time: self.context.timer.time(),
//             mobject,
//             animation_state: Some(animation_state),
//         }
//     }

//     pub fn config(&self) -> &Config {
//         self.context.config
//     }

//     #[must_use]
//     pub fn spawn<M>(&self, mobject: M) -> Alive<SKF, W, M, CollapsedAnimationState>
//     where
//         M: Mobject<MobjectPresentation = MP>,
//     {
//         self.start(Arc::new(mobject), CollapsedAnimationState)
//     }
// }

// #[derive(Deserialize, Serialize)]
// pub struct PreallocatedAnimationEntry<SKF, MP>
// where
//     MP: 'static,
//     SKF: 'static,
// {
//     time_interval: Range<Time>,
//     animation: serde_traitobject::Box<
//         dyn PreallocatedAnimation<MobjectPresentation = MP, SerializableKeyFn = SKF>,
//     >,
// }

// pub struct AllocatedAnimationEntry<SKF, MP> {
//     time_interval: Range<Time>,
//     animation: Box<dyn AllocatedAnimation<MobjectPresentation = MP, SerializableKeyFn = SKF>>,
// }

// impl<SKF, MP> PreallocatedAnimationEntry<SKF, MP> {
//     fn allocate(
//         self,
//         slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> AllocatedAnimationEntry<SKF, MP> {
//         AllocatedAnimationEntry {
//             time_interval: self.time_interval,
//             animation: self
//                 .animation
//                 .into_box()
//                 .allocate(slot_key_generator_type_map),
//         }
//     }
// }

// struct AnimationEntriesSink<'v>(Option<&'v mut Vec<WithTimeInterval<Box<dyn AllocatedAnimation>>>>);

// impl Extend<Box<dyn AllocatedAnimation>> for AnimationEntriesSink<'_> {
//     fn extend<I>(&mut self, iter: I)
//     where
//         I: IntoIterator<Item = Box<dyn AllocatedAnimation>>,
//     {
//         if let Some(animation_entries) = self.0.as_mut() {
//             animation_entries.extend(iter)
//         }
//     }
// }

// trait AnimationState {
//     fn flush_animations(
//         &mut self,
//         time_interval: Range<Time>,
//     ) -> Vec<WithTimeInterval<Box<dyn AnimationErasure>>>;
// }

pub struct Timer {
    time: RefCell<Rc<Time>>,
}

impl Timer {
    fn new() -> Self {
        Self {
            time: RefCell::new(Rc::new(0.0)),
        }
    }

    fn time(&self) -> Rc<Time> {
        self.time.borrow().clone()
    }

    pub fn wait(&self, time: Time) {
        self.time.replace_with(|rc_time| Rc::new(**rc_time + time));
    }
}

struct Context<'c, SKF, W> {
    config: &'c Config,
    timer: Timer,
    world: W,
    phantom: PhantomData<SKF>,
}

impl<'c, SKF, W> Context<'c, SKF, W>
where
    SKF: StorableKeyFn,
    W: World,
{
    fn new(config: &'c Config) -> Self {
        Self {
            config,
            timer: Timer::new(),
            world: W::new(),
            phantom: PhantomData,
        }
    }

    // fn balance<F>(&self, f: F) where F: FnOnce(&Config, &Timer, &W) {
    //     let timer = Timer::new();
    //     let world = W::new();
    //     f(&self.config, &timer, &world);
    //     self.world.merge(world, time_interval, time_eval);
    // }

    // fn grow_stack(&self) -> &(Timer, W) {
    //     self.timer_world_stack.borrow_mut().push((
    //         Timer::new(),
    //         W::new(),
    //     ));
    //     self.timer_world_stack.borrow()
    // }

    // fn shrink_stack<TE>(&self, time_interval: &Range<Time>, time_eval: &TE)
    // where
    //     TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    // {
    //     let mut timer_world_stack = self.timer_world_stack.borrow_mut();
    //     let (timer, world) = timer_world_stack.pop().unwrap();
    //     timer_world_stack.last_mut().1.shrink_stack(time_interval, time_eval, &0.0..timer.time());
    // }

    // fn collect(self) -> Vec<PreallocatedAnimationEntry<MP>> {
    //     let timer_world_stack = self.timer_world_stack.into_inner();
    //     assert!(timer_world_stack.tail.is_empty());
    //     timer_world_stack.head
    // }

    // fn timer_world_ref(&self) -> &(Timer, W) {
    //     self.timer_world_stack.borrow().last()
    // }

    // fn timer_world_mut(&self) -> &mut (Timer, W) {
    //     self.timer_world_stack.borrow_mut().last_mut()
    // }
}

// #[derive(Default)]
// struct TimerStack(RefCell<nonempty::NonEmpty<Timer>>);

// impl TimerStack {
//     fn time(&self) -> Time {
//         self.0.borrow().last().time()
//     }

//     fn grow_stack(&mut self) {
//         self.0.borrow_mut().push(Timer::default());
//     }

//     fn shrink_stack(&mut self) {
//         self.0.borrow_mut().push(Timer {
//             time: RefCell::new(Rc::new(0.0)),
//         });
//     }
// }

// struct Channel(Vec<>)

// pub struct LayerField<'lf, W, MP>
// where
//     MP: 'static,
// {
//     config: &'lf Config,
//     timer_stack: &'lf TimerStack,
//     world: Weak<W>,
//     // layer: Weak<L>,
//     // timer: &'lf Timer,
//     // depth: &'lf RefCell<usize>,
//     animation_entries_stack: RefCell<nonempty::NonEmpty<Vec<PreallocatedAnimationEntry<MP>>>>,
// }

// impl<'lf, W, MP> LayerField<'lf, W, MP>
// where
//     SKF: StorableKeyFn,
//     W: WorldErasure<SKF>,
//     MP: 'static,
// {
//     pub fn new(config: &'lf Config, timer_stack: &'lf TimerStack, world: Weak<W>) -> Self {
//         Self {
//             config,
//             timer_stack,
//             world,
//             animation_entries_stack: RefCell::new(nonempty::NonEmpty::singleton(Vec::new())),
//         }
//     }

//     pub fn grow_stack(&self) {
//         self.animation_entries_stack.borrow_mut().push(Vec::new());
//     }

//     pub fn shrink_stack<TE>(&self, time_interval: Range<Time>, time_eval: &TE)
//     where
//         TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
//     {
//         let child_time_interval = 0.0..self.timer_stack.time();
//         let mut animation_entries_stack = self.animation_entries_stack.borrow_mut();
//         let animation_entries = animation_entries_stack.pop().unwrap();
//         animation_entries_stack
//             .last_mut()
//             .extend(
//                 animation_entries
//                     .into_iter()
//                     .map(|animation_entry| PreallocatedAnimationEntry {
//                         time_interval: time_interval.start
//                             + (time_interval.end - time_interval.start)
//                                 * *time_eval.time_eval(
//                                     animation_entry.time_interval.start,
//                                     child_time_interval.clone(),
//                                 )
//                             ..time_interval.start
//                                 + (time_interval.end - time_interval.start)
//                                     * *time_eval.time_eval(
//                                         animation_entry.time_interval.end,
//                                         child_time_interval.clone(),
//                                     ),
//                         animation: animation_entry.animation,
//                     }),
//             );
//     }

//     pub fn collect(self) -> Vec<PreallocatedAnimationEntry<MP>> {
//         let animation_entries_stack = self.animation_entries_stack.into_inner();
//         assert!(animation_entries_stack.tail.is_empty());
//         animation_entries_stack.head
//     }

//     fn world(&self) -> Rc<W> {
//         self.world.upgrade().unwrap()
//     }

//     // fn layer(&self) -> Rc<L> {
//     //     self.layer.upgrade().unwrap()
//     // }

//     fn push<T>(&self, time_interval: Range<Time>, animation: T)
//     where
//         A: Animation<MobjectPresentation = MP>,
//         Box<T>: Storable,
//     {
//         if time_interval.start < time_interval.end {
//             self.animation_entries_stack
//                 .borrow_mut()
//                 .last_mut()
//                 .push(PreallocatedAnimationEntry {
//                     time_interval,
//                     animation: serde_traitobject::Box::new(animation),
//                 });
//         }
//     }

//     fn start<TS>(&self, animation_state: TS) -> Alive<W, TS>
//     where
//         TS: AnimationState<MobjectPresentation = MP>,
//     {
//         Alive {
//             channel_attachment: self,
//             // index: usize,
//             spawn_time: self.timer_stack.time(),
//             animation_state: Some(animation_state),
//         }
//     }

//     #[must_use]
//     pub fn spawn<M>(&self, mobject: M) -> Alive<W, M, CollapsedAnimationState>
//     where
//         M: Mobject<MobjectPresentation = MP>,
//     {
//         self.start(CollapsedAnimationState {
//             mobject: Arc::new(mobject),
//         })
//     }
// }

// pub trait LayerEntry: serde_traitobject::Deserialize + serde_traitobject::Serialize {
//     fn new(config: &Config, timer_stack: &TimerStack) -> Self;
//     fn merge<TE>(&mut self, laye_entry: Self, time_interval: Range<Time>, time_eval: &TE)
//     where
//         TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>;
//     fn prepare(
//         &self,
//         time: Time,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> Box<dyn Render>;
// }

// pub trait World {
//     type SerializableKeyFn: SerializableKeyFn;

//     fn new(config: &Config, timer_stack: &TimerStack) -> Rc<Self>;
//     fn grow_stack(&self);
//     fn shrink_stack<TE>(&self, time_interval: Range<Time>, time_eval: &TE)
//     where
//         TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>;
//     fn collect(self) -> Vec<Box<dyn LayerPreallocated>>;
// }

// pub trait WorldEntry {
//     type SerializableKeyFn: StorableKeyFn;

//     fn new(config: &Config, timer_stack: &TimerStack) -> Self;
//     fn merge<TE>(&mut self, world_entry: Self, time_interval: Range<Time>, time_eval: &TE)
//     where
//         TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>;
//     fn collect(self) -> Vec<Box<dyn LayerEntry>>;
// }

pub struct Alive<'a, M, TS, C, SKF>
where
    SKF: StorableKeyFn,
    W: WorldErasure<SKF>,
    M: Mobject,
    TS: AnimationState<SKF, W, M>,
{
    channel_attachment: &'a ChannelAttachment<'a, SKF, W, M::MobjectPresentation>,
    spawn_time: Rc<Time>,
    mobject: Arc<M>,
    animation_state: Option<TS>,
}

impl<'a, SKF, W, M, TS> Alive<'a, SKF, W, M, TS>
where
    SKF: StorableKeyFn,
    W: WorldErasure<SKF>,
    M: Mobject,
    TS: AnimationState<SKF, W, M>,
{
    // pub(crate) fn alive_context(&self) -> &'ac AC {
    //     &self.alive_context
    // }

    // pub(crate) fn archive_state(&self) -> &AS {
    //     self.archive_state.as_ref().unwrap()
    // }

    fn end(&mut self) -> Arc<TS::OutputMobject> {
        // let mut recorder = self.alive_recorder.recorder.borrow_mut();
        // let entry = recorder.get_mut(self.index).unwrap();
        let spawn_time = self.spawn_time;
        let archive_time = self.channel_attachment.timer_stack.time();
        let mut animation_state = self.animation_state.take().unwrap();
        animation_state.archive(
            spawn_time..archive_time,
            self.channel_attachment,
            self.mobject.clone(),
        )
    }

    pub(crate) fn map<F, FO>(&mut self, f: F) -> Alive<'a, SKF, W, TS::OutputMobject, FO>
    where
        F: FnOnce(Arc<TS::OutputMobject>) -> FO,
        FO: AnimationState<SKF, W, TS::OutputMobject>,
    {
        self.channel_attachment.start(f(self.end()))
    }
}

impl<SKF, W, M, TS> Drop for Alive<'_, SKF, W, M, TS>
where
    SKF: StorableKeyFn,
    W: WorldErasure<SKF>,
    M: Mobject,
    TS: AnimationState<SKF, W, M>,
{
    fn drop(&mut self) {
        if self.animation_state.is_some() {
            self.end();
        }
    }
}

pub trait AnimationState<M, C, SKF>
where
    M: Mobject,
    SKF: StorableKeyFn,
{
    type OutputMobject: Mobject;

    fn archive(
        &mut self,
        time_interval: Range<Rc<Time>>,
        channel_attachment: &ChannelAttachment<SKF, W, M::MobjectPresentation>,
        mobject: Arc<M>,
    ) -> Arc<Self::OutputMobject>;
}

pub struct CollapsedAnimationState;

impl<SKF, W, M> AnimationState<SKF, W, M> for CollapsedAnimationState
where
    SKF: StorableKeyFn,
    W: WorldErasure<SKF>,
    M: Mobject,
{
    // type MobjectPresentation = M::MobjectPresentation;
    type OutputMobject = M;

    fn archive(
        &mut self,
        time_interval: Range<Rc<Time>>,
        channel_attachment: &ChannelAttachment<SKF, W, M::MobjectPresentation>,
        mobject: Arc<M>,
    ) -> Arc<Self::OutputMobject> {
        channel_attachment.push(
            time_interval,
            StaticAnimation {
                mobject: mobject.clone(),
            },
        );
        mobject
    }
}

pub struct IndeterminedAnimationState<TE> {
    time_eval: Arc<TE>,
}

impl<SKF, W, M, TE> AnimationState<SKF, W, M> for IndeterminedAnimationState<TE>
where
    SKF: StorableKeyFn,
    W: WorldErasure<SKF>,
    M: Mobject,
    TE: TimeEval,
{
    // type MobjectPresentation = M::MobjectPresentation;
    type OutputMobject = M;

    fn archive(
        &mut self,
        time_interval: Range<Rc<Time>>,
        channel_attachment: &ChannelAttachment<SKF, W, M::MobjectPresentation>,
        mobject: Arc<M>,
    ) -> Arc<Self::OutputMobject> {
        channel_attachment.push(
            time_interval,
            StaticAnimation {
                mobject: mobject.clone(),
            },
        );
        mobject
    }
}

pub struct UpdateAnimationState<TE, U> {
    // mobject: Arc<M>,
    time_eval: Arc<TE>,
    update: Arc<U>,
}

impl<SKF, W, M, TE, U> AnimationState<SKF, W, M> for UpdateAnimationState<TE, U>
where
    SKF: StorableKeyFn,
    W: WorldErasure<SKF>,
    M: Mobject,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, M>,
{
    // type MobjectPresentation = M::MobjectPresentation;
    type OutputMobject = M;

    fn archive(
        &mut self,
        time_interval: Range<Rc<Time>>,
        channel_attachment: &ChannelAttachment<SKF, W, M::MobjectPresentation>,
        mobject: Arc<M>,
    ) -> Arc<Self::OutputMobject> {
        channel_attachment.push(
            time_interval,
            DynamicAnimation {
                mobject: mobject.clone(),
                time_eval: self.time_eval.clone(),
                update: self.update.clone(),
            },
        );
        mobject
    }

    // type OutputAnimationState = M, CollapsedAnimationState;

    // fn into_next(
    //     self,
    //     supervisor: &Supervisor,
    //     time_interval: Range<Rc<Time>>,
    //     mut animation_entries_sink: AnimationEntriesSink,
    // ) -> Self::OutputAnimationState {
    //     animation_entries_sink.extend_one(supervisor.new_dynamic_animation_entry(
    //         time_interval.clone(),
    //         self.mobject.clone(),
    //         self.time_eval.clone(),
    //         self.update.clone(),
    //     ));
    //     let mut mobject = Arc::unwrap_or_clone(self.mobject);
    //     self.update.update(
    //         self.time_eval.time_eval(time_interval.end, time_interval),
    //         &mut mobject,
    //     );
    //     CollapsedAnimationState {
    //         mobject: Arc::new(mobject),
    //     }
    // }
}

pub struct ConstructAnimationState<TE, C> {
    // mobject: Arc<M>,
    time_eval: Arc<TE>,
    // root_archive: (Range<Time>, Vec<RenderableEntry>),
    // time_eval: Arc<TE>,
    construct: Option<C>,
}

impl<SKF, W, M, TE, C> AnimationState<SKF, W, M> for ConstructAnimationState<TE, C>
where
    SKF: StorableKeyFn,
    W: WorldErasure<SKF>,
    M: Mobject,
    TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    C: Construct<SKF, W, M>,
{
    // type MobjectPresentation = M::MobjectPresentation;
    type OutputMobject = C::OutputMobject;

    fn archive(
        &mut self,
        time_interval: Range<Rc<Time>>,
        channel_attachment: &ChannelAttachment<SKF, W, M::MobjectPresentation>,
        mobject: Arc<M>,
    ) -> Arc<Self::OutputMobject> {
        // TODO: move beside `push` as `extend`
        let child_context = Context::<SKF, W>::new(channel_attachment.context.config);
        let world_attachment = child_context.world.attachment(&child_context);
        let output_mobject = self
            .construct
            .take()
            .unwrap()
            .construct(&world_attachment, mobject)
            .end();
        if !Rc::ptr_eq(&time_interval.start, &time_interval.end) {
            channel_attachment.context.world.merge(
                child_context.world,
                &(*time_interval.start..*time_interval.end),
                self.time_eval.as_ref(),
                &(0.0..*child_context.timer.time()),
            );
        }

        // let world = world.as_ref();
        // world.grow_stack();
        // let output_mobject = self
        //     .construct
        //     .take()
        //     .unwrap()
        //     .construct(
        //         world,
        //         channel_attachment.start(CollapsedAnimationState {
        //             mobject: self.mobject.clone(),
        //         }),
        //     )
        //     .end()
        //     .mobject;
        // world.shrink_stack(time_interval, self.time_eval.as_ref());
        output_mobject
    }

    // fn archive(
    //     &mut self,
    //     time_interval: Range<Time>,
    //     _local_archive: Self::LocalArchive,
    //     global_archive: &Self::GlobalArchive,
    // ) {
    //     let (child_time_interval, renderables) = &mut self.root_archive;
    //     let rescale_time_interval = |entry_time_interval: &mut Range<Time>| {
    //         *entry_time_interval = time_interval.start
    //             + (time_interval.end - time_interval.start)
    //                 * *self
    //                     .time_eval
    //                     .time_eval(entry_time_interval.start, child_time_interval.clone())
    //             ..time_interval.start
    //                 + (time_interval.end - time_interval.start)
    //                     * *self
    //                         .time_eval
    //                         .time_eval(entry_time_interval.end, child_time_interval.clone())
    //     };
    //     for RenderableEntry {
    //         time_interval,
    //         animation_entries,
    //         ..
    //     } in renderables.as_mut_slice()
    //     {
    //         rescale_time_interval(time_interval);
    //         for AnimationEntry { time_interval, .. } in animation_entries.as_mut_slice() {
    //             rescale_time_interval(time_interval);
    //         }
    //     }
    //     global_archive.borrow_mut().1.extend(renderables.drain(..));
    // }

    // type OutputAnimationState = CollapsedAnimationState<C::OutputMobject>;

    // fn into_next(
    //     self,
    //     supervisor: &Supervisor,
    //     time_interval: Range<Time>,
    //     mut animation_entries_sink: AnimationEntriesSink,
    // ) -> Self::OutputAnimationState {
    //     let child_supervisor = Supervisor::new(supervisor.config);
    //     let output_animation_state = child_supervisor
    //         .end(&self.construct.construct(
    //             child_supervisor.start(AliveContent {
    //                 spawn_time: child_supervisor.time.borrow().clone(),
    //                 animation_state: CollapsedAnimationState {
    //                     mobject: self.mobject,
    //                 },
    //             }),
    //             &child_supervisor,
    //         ))
    //         .animation_state;
    //     let children_time_interval = child_supervisor.time_interval();
    //     animation_entries_sink.extend(child_supervisor.iter_animation_entries().map(
    //         |mut animation_entry| {
    //             animation_entry.time_interval = time_interval.start
    //                 + (time_interval.end - time_interval.start)
    //                     * *self.time_eval.time_eval(
    //                         animation_entry.time_interval.start,
    //                         children_time_interval.clone(),
    //                     )
    //                 ..time_interval.start
    //                     + (time_interval.end - time_interval.start)
    //                         * *self.time_eval.time_eval(
    //                             animation_entry.time_interval.end,
    //                             children_time_interval.clone(),
    //                         );
    //             animation_entry
    //         },
    //     ));
    //     output_animation_state
    // }

    // fn into_next(
    //     self,
    //     time_interval: Range<f32>,
    //     _mobject: Arc<M>,
    //     _storage: &S,
    // ) -> Vec<AnimationEntry> {
    //     let mut animation_entries = self.children_animation_entries;
    //     animation_entries.iter_mut().for_each(|animation_entry| {
    //         animation_entry.rescale_time_interval(
    //             &self.time_eval,
    //             &self.children_time_interval,
    //             &time_interval,
    //         )
    //     });
    //     animation_entries
    // }
}

// fn new_render_pass<'ce>(
//     encoder: &'ce mut wgpu::CommandEncoder,
//     target: &'ce wgpu::TextureView,
//     // clip_bounds: &iced::Rectangle<u32>,
//     load: wgpu::LoadOp<wgpu::Color>,
// ) -> wgpu::RenderPass<'ce> {
//     encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//         label: None,
//         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//             view: target,
//             resolve_target: None,
//             ops: wgpu::Operations {
//                 load,
//                 store: wgpu::StoreOp::Store,
//             },
//         })],
//         depth_stencil_attachment: None,
//         timestamp_writes: None,
//         occlusion_query_set: None,
//     })
// }

// struct WithSpawnTime<D> {
//     spawn_time: Rc<Time>,
//     data: D,
// }

// impl<D> WithSpawnTime<D> {
//     fn map<F, FO>(self, f: F) -> WithSpawnTime<FO>
//     where
//         F: FnOnce(D) -> FO,
//     {
//         WithSpawnTime {
//             spawn_time: self.spawn_time,
//             data: f(self.data),
//         }
//     }
// }

// impl<L> AliveContent<World<'_>> for WithSpawnTime<Rc<L>>
// where
//     W: World,
// {
//     type Next = ();
//     type Input = L;
//     type Output = Vec<Rc<dyn Layer>>;

//     fn new(input: Self::Input, context: &World<'_>) -> Self {
//         let layer = Rc::new(input);
//         context.layers.borrow_mut().push(layer);
//         Self {
//             spawn_time: context.time.borrow().clone(),
//             data: layer,
//         }
//     }

//     fn iterate(self, context: &World<'_>) -> (Self::Output, Self::Next) {
//         (context.layers.borrow().clone(), ())
//     }
// }

// impl<L, TS> AliveContent<L> for WithSpawnTime<TS>
// where
//     W: World,
//     TS: AnimationState,
// {
// }

// pub struct Supervisor<'c> {
//     config: &'c Config,
//     // storage: &'s Storage,
//     time: RefCell<Arc<Time>>,
//     animation_slots: RefCell<Vec<Result<Vec<Box<dyn AllocatedAnimation>>, Arc<dyn Any>>>>,
// }

// impl<L, MB> IntoArchiveState<AliveRenderable<'_, '_, LayerRenderableState<L>>> for MB
// where
//     W: World,
//     MB: MobjectBuilder<L>,
// {
//     type ArchiveState = CollapsedAnimationState<MB::Instantiation>;

//     fn into_archive_state(
//         self,
//         alive_context: &AliveRenderable<'_, '_, LayerRenderableState<L>>,
//     ) -> Self::ArchiveState {
//         CollapsedAnimationState {
//             mobject: Arc::new(self.instantiate(
//                 &alive_context.archive_state().layer,
//                 alive_context.alive_context().config(),
//             )),
//         }
//     }
// }

// pub type Alive<'a2, 'a1, 'a0, RAS, TAS> = Alive<'a2, AliveRenderable<'a1, 'a0, RAS>, TAS>;

pub trait TimeMetric: 'static {}

pub struct NormalizedTimeMetric(f32);

impl Deref for NormalizedTimeMetric {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TimeMetric for NormalizedTimeMetric {}

pub struct DenormalizedTimeMetric(f32);

impl Deref for DenormalizedTimeMetric {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TimeMetric for DenormalizedTimeMetric {}

pub trait TimeEval:
    'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
{
    type OutputTimeMetric: TimeMetric;

    fn time_eval(&self, time: Time, time_interval: Range<Time>) -> Self::OutputTimeMetric;
}

pub trait IncreasingTimeEval: TimeEval {}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct NormalizedTimeEval;

impl TimeEval for NormalizedTimeEval {
    type OutputTimeMetric = NormalizedTimeMetric;

    fn time_eval(&self, time: Time, time_interval: Range<Time>) -> Self::OutputTimeMetric {
        NormalizedTimeMetric(
            (time - time_interval.start) / (time_interval.end - time_interval.start),
        )
    }
}

impl IncreasingTimeEval for NormalizedTimeEval {}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DenormalizedTimeEval;

impl TimeEval for DenormalizedTimeEval {
    type OutputTimeMetric = DenormalizedTimeMetric;

    fn time_eval(&self, time: Time, time_interval: Range<Time>) -> Self::OutputTimeMetric {
        DenormalizedTimeMetric(time - time_interval.start)
    }
}

impl IncreasingTimeEval for DenormalizedTimeEval {}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RateComposeTimeEval<RA, TE> {
    rate: RA,
    time_eval: Arc<TE>,
}

impl<RA, TE> TimeEval for RateComposeTimeEval<RA, TE>
where
    RA: Rate<TE::OutputTimeMetric>,
    TE: TimeEval,
{
    type OutputTimeMetric = RA::OutputTimeMetric;

    fn time_eval(&self, time: Time, time_interval: Range<Time>) -> Self::OutputTimeMetric {
        self.rate
            .eval(self.time_eval.time_eval(time, time_interval))
    }
}

impl<RA, TE> IncreasingTimeEval for RateComposeTimeEval<RA, TE>
where
    RA: IncreasingRate<TE::OutputTimeMetric>,
    TE: IncreasingTimeEval,
{
}

pub trait Quantize: Sized {
    type Output<TE>
    where
        TE: TimeEval;

    fn quantize<TE>(self, time_metric: TE) -> Self::Output<TE>
    where
        TE: TimeEval;
}

pub trait Collapse: Sized {
    type Output;

    fn collapse(self) -> Self::Output;
}

pub trait ApplyRate<TM>: Sized
where
    TM: TimeMetric,
{
    type Output<RA>
    where
        RA: Rate<TM>;

    fn apply_rate<RA>(self, rate: RA) -> Self::Output<RA>
    where
        RA: Rate<TM>;
}

pub trait ApplyUpdate<TM, M>: Sized
where
    TM: TimeMetric,
    M: Mobject,
{
    type Output<U>
    where
        U: Update<TM, M>;

    fn apply_update<U>(self, update: U) -> Self::Output<U>
    where
        U: Update<TM, M>;
}

pub trait ApplyConstruct<SKF, W, M>: Sized
where
    SKF: StorableKeyFn,
    W: WorldErasure<SKF>,
    M: Mobject,
{
    type Output<C>
    where
        C: Construct<SKF, W, M>;

    fn apply_construct<C>(self, construct: C) -> Self::Output<C>
    where
        C: Construct<SKF, W, M>;
}

impl<'a, SKF, W, M> Quantize for Alive<'a, SKF, W, M, CollapsedAnimationState>
where
    SKF: StorableKeyFn,
    W: WorldErasure<SKF>,
    M: Mobject,
{
    type Output<TE> =
        Alive<'a, SKF, W, M, IndeterminedAnimationState<TE>>
    where
        TE: TimeEval;

    #[must_use]
    fn quantize<TE>(mut self, time_eval: TE) -> Self::Output<TE>
    where
        TE: TimeEval,
    {
        self.map(|animation_state| IndeterminedAnimationState {
            mobject: animation_state.mobject.clone(),
            time_eval: Arc::new(time_eval),
        })
    }
}

impl<'a, SKF, W, M, TE, U> Collapse for Alive<'a, SKF, W, M, UpdateAnimationState<TE, U>>
where
    SKF: StorableKeyFn,
    W: WorldErasure<SKF>,
    M: Mobject,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, M>,
{
    type Output = Alive<'a, SKF, W, M, CollapsedAnimationState>;

    #[must_use]
    fn collapse(mut self) -> Self::Output {
        self.map(|animation_state| CollapsedAnimationState {
            mobject: animation_state.mobject.clone(),
        })
    }
}

impl<'a, SKF, W, M, TE, C> Collapse for Alive<'a, SKF, W, ConstructAnimationState<M, TE, C>>
where
    SKF: StorableKeyFn,
    W: WorldErasure<SKF>,
    M: Mobject,
    TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    C: Construct<M>,
{
    type Output = Alive<'a, SKF, W, M, CollapsedAnimationState>;

    #[must_use]
    fn collapse(mut self) -> Self::Output {
        self.map(|animation_state| CollapsedAnimationState {
            mobject: animation_state.mobject.clone(),
        })
    }
}

impl<'a, SKF, W, M, TE> ApplyRate<TE::OutputTimeMetric>
    for Alive<'a, SKF, W, M, IndeterminedAnimationState<TE>>
where
    SKF: StorableKeyFn,
    W: WorldErasure<SKF>,
    M: Mobject,
    TE: TimeEval,
{
    type Output<RA> =
        Alive<'a, SKF, W, M, IndeterminedAnimationState<RateComposeTimeEval<RA, TE>>>
    where
        RA: Rate<TE::OutputTimeMetric>;

    #[must_use]
    fn apply_rate<RA>(mut self, rate: RA) -> Self::Output<RA>
    where
        RA: Rate<TE::OutputTimeMetric>,
    {
        self.map(|animation_state| IndeterminedAnimationState {
            mobject: animation_state.mobject.clone(),
            time_eval: Arc::new(RateComposeTimeEval {
                rate,
                time_eval: animation_state.time_eval.clone(),
            }),
        })
    }
}

impl<'a, SKF, W, M, TE> ApplyUpdate<TE::OutputTimeMetric, M>
    for Alive<'a, SKF, W, M, IndeterminedAnimationState<TE>>
where
    SKF: StorableKeyFn,
    W: WorldErasure<SKF>,
    M: Mobject,
    TE: TimeEval,
{
    type Output<U> =
        Alive<'a, SKF, W, M, UpdateAnimationState<TE, U>>
    where
        U: Update<TE::OutputTimeMetric, M>;

    #[must_use]
    fn apply_update<U>(mut self, update: U) -> Self::Output<U>
    where
        U: Update<TE::OutputTimeMetric, M>,
    {
        self.map(|animation_state| UpdateAnimationState {
            mobject: animation_state.mobject,
            time_eval: animation_state.time_eval,
            update: Arc::new(update),
        })
    }
}

impl<'a, SKF, W, M> ApplyUpdate<NormalizedTimeMetric, M>
    for Alive<'a, SKF, W, M, CollapsedAnimationState>
where
    SKF: StorableKeyFn,
    W: WorldErasure<SKF>,
    M: Mobject,
{
    type Output<U> =
        Alive<'a, SKF, W, M, CollapsedAnimationState>
    where
        U: Update<NormalizedTimeMetric, M>;

    #[must_use]
    fn apply_update<U>(mut self, update: U) -> Self::Output<U>
    where
        U: Update<NormalizedTimeMetric, M>,
    {
        self.map(|animation_state| {
            let mut mobject = Arc::unwrap_or_clone(animation_state.mobject);
            update.update(NormalizedTimeMetric(1.0), &mut mobject);
            CollapsedAnimationState {
                mobject: Arc::new(mobject),
            }
        })
    }
}

impl<'a, SKF, W, M, TE> ApplyConstruct<L, M>
    for Alive<'a, SKF, W, M, IndeterminedAnimationState<TE>>
where
    SKF: StorableKeyFn,
    W: WorldErasure<SKF>,
    M: Mobject,
    TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
{
    type Output<C> =
        Alive<'a, W, ConstructAnimationState<C::OutputMobject, TE>>
    where
        C: Construct<L, M>;

    #[must_use]
    fn apply_construct<C>(mut self, construct: C) -> Self::Output<C>
    where
        C: Construct<L, M>,
    {
        self.map(|alive_context, animation_state| {
            let child_root = AliveRoot::new(alive_context.alive_context().config());
            let child_renderable = child_root.start(LayerRenderableState {
                layer: alive_context.archive_state().layer.clone(),
            });
            let child_animation = child_renderable.start(CollapsedAnimationState {
                mobject: animation_state.mobject.clone(),
            });
            let mobject = construct
                .construct(&child_root, &child_renderable, child_animation)
                .archive_state()
                .mobject
                .clone();
            drop(child_renderable);
            ConstructAnimationState {
                mobject,
                time_eval: animation_state.time_eval.clone(),
                root_archive: child_root.into_archive(),
            }
        })
    }
}

pub trait QuantizeExt: Quantize {
    fn animate(self) -> Self::Output<NormalizedTimeEval>;
    fn animating(self) -> Self::Output<DenormalizedTimeEval>;
}

impl<TS> QuantizeExt for TS
where
    TS: Quantize,
{
    #[must_use]
    fn animate(self) -> Self::Output<NormalizedTimeEval> {
        self.quantize(NormalizedTimeEval)
    }

    #[must_use]
    fn animating(self) -> Self::Output<DenormalizedTimeEval> {
        self.quantize(DenormalizedTimeEval)
    }
}

// pub(crate) struct Storage(typemap_rev::TypeMap);

// impl Storage {
//     // fn get_static<M>(&self) -> &M::MobjectPresentation
//     // where
//     //     M: Mobject,
//     // {
//     //     self.0.get::<StaticAnimation<M>>()
//     // }
// }
