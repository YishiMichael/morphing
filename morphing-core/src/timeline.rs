use core::range::IterRangeFrom;
use core::range::Range;
use core::range::RangeFrom;
use std::any::TypeId;
use std::cell::RefCell;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;

use super::stage::Channel;
use super::stage::ChannelArchitecture;
use super::stage::ChannelAttachment;
use super::stage::ChannelIndex;
use super::stage::LayerIndex;
use super::stage::LayerIndexed;
use super::stage::Node;
use super::stage::World;
use super::stage::WorldIndexed;
use super::storable::DynKey;
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
use super::traits::MobjectPresentation;
use super::traits::Rate;
use super::traits::Update;

// #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
// pub struct StaticTimelineId(pub u64);

// #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
// pub struct DynamicTimelineId(pub u64);

// impl TimelineId {
//     fn new(timeline: &serde_traitobject::Arc<dyn Timeline>) -> Self {
//         // Hash `Arc<T>` instead of `T`.
//         // Presentation maps inside `storage` are identified only by `T::Presentation` type, without `T`.
//         Self(seahash::hash(
//             &ron::ser::to_string(timeline).unwrap().into_bytes(),
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

pub enum PresentationKey<MP>
where
    MP: 'static + Send + Sync,
{
    Static(
        Arc<StorageKey<
            (TypeId, Box<dyn DynKey>),
            <<SwapSlot<SharableSlot<MP>> as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        >>,
    ),
    Dynamic(
        Arc<StorageKey<
            (TypeId, Box<dyn DynKey>, Box<dyn DynKey>),
            <<SwapSlot<VecSlot<MP>> as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        >>,
    ),
}

impl<MP> PresentationKey<MP>
where
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

trait Timeline:
    'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize + Storable
{
    type MobjectPresentation: Send + Sync;
    // type SerdeKey: 'static + Eq + Hash + Send + Sync;
    // type MobjectPresentationStorage: PresentationStorage;

    // fn serde_key(&self) -> Self::SerdeKey;
    fn init_presentation(&self, device: &wgpu::Device) -> <Self::Slot as Slot>::Value;
    fn erase_presentation_key(
        &self,
        mobject_presentation_key: Arc<
            StorageKey<
                Self::StorableKey,
                <<Self::Slot as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
            >,
        >,
    ) -> PresentationKey<Self::MobjectPresentation>;
    fn prepare_presentation(
        &self,
        time: Time,
        time_interval: Range<Time>,
        mobject_presentation: &mut <Self::Slot as Slot>::Value,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    );
    // fn erased(&self) -> Box<dyn Timeline<PresentationStorage = Self::PresentationStorage>>;

    // let slot_map = storage
    //     .entry::<TimelineAllocation<Self::SerdeKey, Self::MobjectPresentationStorage>>()
    //     .or_insert_with(HashMap::new);
    // let serde_key = self.serde_key();
    // let slot_id = slot_map
    //     .entry(serde_key.clone())
    //     .or_insert_with(Self::MobjectPresentationStorage::new)
    //     .allocate();
    // TimelineAllocation {
    //     serde_key,
    //     slot_id,
    //     timeline: self as Box<
    //         dyn Timeline<
    //             SerdeKey = Self::SerdeKey,
    //             MobjectPresentationStorage = Self::MobjectPresentationStorage,
    //         >,
    //     >,
    // }
}

// struct StaticTimelineKeyFn<M, MKF>(PhantomData<(M, MKF)>);

// impl<M, MKF> Default for StaticTimelineKeyFn<M, MKF> {
//     fn default() -> Self {
//         Self(PhantomData)
//     }
// }

// impl<M, MKF> KeyFn for StaticTimelineKeyFn<M, MKF>
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
struct StaticTimeline<TQ>
where
    TQ: TypeQuery,
{
    // mobject_serde_key: serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    // slot_id: SI,
    // time_interval: Range<Time>,
    mobject: Arc<TQ::Mobject>,
}

impl<TQ> Storable for StaticTimeline<TQ>
where
    TQ: TypeQuery,
{
    type StorableKey = (TypeId, Box<dyn DynKey>);
    type Slot = SwapSlot<SharableSlot<TQ::MobjectPresentation>>;

    fn key(
        &self,
        storable_key_fn: &fn(&dyn serde_traitobject::Serialize) -> Box<dyn DynKey>,
    ) -> Self::StorableKey {
        (
            TypeId::of::<(TQ::Layer, TQ::Mobject)>(),
            storable_key_fn(self.mobject.as_ref()),
        )
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

impl<TQ> Timeline for StaticTimeline<TQ>
where
    TQ: TypeQuery,
{
    type MobjectPresentation = TQ::MobjectPresentation;
    // type PresentationKey =
    //     PresentationKey<TQ::MobjectPresentation, <TQ::StorableKeyFn as StorableKeyFn>::Output>;
    // type SerdeKey = serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>;
    // type MobjectPresentationStorage = ;

    // fn serde_key(&self) -> Self::SerdeKey {
    //     serde_hashkey::to_key_with_ordered_float(self.mobject.as_ref()).unwrap()
    // }

    fn init_presentation(&self, device: &wgpu::Device) -> <Self::Slot as Slot>::Value {
        Arc::new(TQ::MobjectPresentation::presentation(
            self.mobject.as_ref(),
            device,
        ))
    }

    fn erase_presentation_key(
        &self,
        mobject_presentation_key: Arc<
            StorageKey<
                Self::StorableKey,
                <<Self::Slot as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
            >,
        >,
    ) -> PresentationKey<Self::MobjectPresentation> {
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

    // fn erased(&self) -> Box<dyn Timeline<PresentationStorage = Self::PresentationStorage>> {
    //     Box::new(StaticTimeline {
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

// struct DynamicTimelineKeyFn<TM, M, U, MKF, UKF>(PhantomData<(TM, M, U, MKF, UKF)>);

// impl<TM, M, U, MKF, UKF> Default for DynamicTimelineKeyFn<TM, M, U, MKF, UKF> {
//     fn default() -> Self {
//         Self(PhantomData)
//     }
// }

// impl<TM, M, U, MKF, UKF> KeyFn for DynamicTimelineKeyFn<TM, M, U, MKF, UKF>
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
struct DynamicTimeline<TQ, TE, U>
where
    TQ: TypeQuery,
{
    // mobject_serde_key: serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    // update_serde_key: serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    // slot_id: SI,
    // time_interval: Range<Time>,
    mobject: Arc<TQ::Mobject>,
    time_eval: TE,
    update: U,
}

impl<TQ, TE, U> Storable for DynamicTimeline<TQ, TE, U>
where
    TQ: TypeQuery,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, TQ>,
{
    type StorableKey = (TypeId, Box<dyn DynKey>, Box<dyn DynKey>);
    type Slot = SwapSlot<VecSlot<TQ::MobjectPresentation>>;

    fn key(
        &self,
        storable_key_fn: &fn(&dyn serde_traitobject::Serialize) -> Box<dyn DynKey>,
    ) -> Self::StorableKey {
        (
            TypeId::of::<(TQ::Layer, TQ::Mobject, U)>(),
            storable_key_fn(self.mobject.as_ref()),
            storable_key_fn(&self.update),
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

impl<TQ, TE, U> Timeline for DynamicTimeline<TQ, TE, U>
where
    TQ: TypeQuery,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, TQ>,
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
    type MobjectPresentation = TQ::MobjectPresentation;

    fn init_presentation(&self, device: &wgpu::Device) -> <Self::Slot as Slot>::Value {
        TQ::MobjectPresentation::presentation(self.mobject.as_ref(), device)
    }

    fn erase_presentation_key(
        &self,
        mobject_presentation_key: Arc<
            StorageKey<
                Self::StorableKey,
                <<Self::Slot as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
            >,
        >,
    ) -> PresentationKey<Self::MobjectPresentation> {
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
            self.mobject.as_ref(),
            mobject_presentation,
            device,
            queue,
            format,
        );
    }

    // fn erased(&self) -> Box<dyn Timeline<PresentationStorage = Self::PresentationStorage>> {
    //     Box::new(DynamicTimeline {
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
// struct TimelineAllocation<SK, MPS>
// where
//     SK: SerdeKey,
//     MPS: MobjectPresentationStorage,
// {
//     // time_interval: Range<Time>,
//     serde_key: SK,        //T::SerdeKey,
//     slot_id: MPS::SlotId, //<T::MobjectPresentationStorage as MobjectPresentationStorage>::SlotId,
//     timeline: Box<dyn Timeline<SerdeKey = SK, MobjectPresentationStorage = MPS>>,
// }

// impl<SK, MPS> typemap_rev::TypeMapKey for TimelineAllocation<SK, MPS>
// where
//     SK: SerdeKey,
//     MPS: MobjectPresentationStorage,
// {
//     type Value = HashMap<SK, MPS>;
// }

// impl<SK, MPS> TimelineAllocation<SK, MPS>
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
//     T: Timeline,
// {
//     timeline: T,
// }

pub trait TimelineErasure: serde_traitobject::Deserialize + serde_traitobject::Serialize {
    type MobjectPresentation;

    fn allocation(
        self: Box<Self>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Box<dyn TimelineAllocationErasure<MobjectPresentation = Self::MobjectPresentation>>;
}

pub trait TimelineAllocationErasure {
    type MobjectPresentation: Send + Sync;

    fn prepare(
        &self,
        time: Time,
        time_interval: Range<f32>,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> PresentationKey<Self::MobjectPresentation>;
}

struct TimelineAllocation<T>
where
    T: Timeline,
{
    storage_key: Arc<
        StorageKey<
            T::StorableKey,
            <<T::Slot as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        >,
    >,
    timeline: Box<T>,
}

impl<T> TimelineErasure for T
where
    T: Timeline,
{
    type MobjectPresentation = T::MobjectPresentation;

    fn allocation(
        self: Box<Self>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Box<dyn TimelineAllocationErasure<MobjectPresentation = Self::MobjectPresentation>> {
        Box::new(TimelineAllocation {
            storage_key: Arc::new(slot_key_generator_type_map.allocate(&self)),
            timeline: self,
        })
    }
}

impl<T> TimelineAllocationErasure for TimelineAllocation<T>
where
    T: Timeline,
{
    // fn fetch_presentation(
    //     &self,
    //     storage_type_map: &StorageTypeMap,
    // ) -> PresentationCell<Self::MobjectPresentation> {
    //     self.storable_primitive()
    //         .fetch_presentation(storage_type_map.get_ref(self).as_ref().unwrap())
    // }

    type MobjectPresentation = T::MobjectPresentation;

    fn prepare(
        &self,
        time: Time,
        time_interval: Range<f32>,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> PresentationKey<Self::MobjectPresentation> {
        let mobject_presentation = storage_type_map
            .get_or_insert_with::<_, T::Slot, _>(&self.storage_key, || {
                self.timeline.init_presentation(device)
            });
        self.timeline.prepare_presentation(
            time,
            time_interval,
            mobject_presentation,
            device,
            queue,
            format,
        );
        self.timeline
            .erase_presentation_key(self.storage_key.clone())
    }
}

// impl<V> Node<V> {
//     pub(crate) fn map_ref<F, FO>(&self, f: F) -> Node<FO>
//     where
//         F: FnMut(&V) -> FO,
//     {
//         match self {
//             Self::None => Self::None,
//             Self::Singleton(v) => Self::Singleton(f(v)),
//             Self::Multiton(vs) => Self::Multiton(vs.iter().map(f).collect()),
//         }
//     }

//     pub(crate) fn map<F, FO>(self, f: F) -> Node<FO>
//     where
//         F: FnMut(V) -> FO,
//     {
//         match self {
//             Self::None => Self::None,
//             Self::Singleton(v) => Self::Singleton(f(v)),
//             Self::Multiton(vs) => Self::Multiton(vs.into_iter().map(f).collect()),
//         }
//     }
// }

// trait Structure {}

// pub trait Node<SKF>
// where
//     SKF: StorableKeyFn,
// {
//     // type StorableKeyFn: StorableKeyFn;
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
//     ) -> ChannelAllocated<Self::MobjectPresentation, SKF>;
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
//     ) -> Vec<PresentationKey<Self::MobjectPresentation, SKF>>;
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
//             Box<dyn TimelineErasure<SKF, MobjectPresentation = MP>>,
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
//                     .map(|(entry_time_interval, timeline)| {
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
//                             timeline,
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
//     ) -> ChannelAllocated<Self::MobjectPresentation, SKF> {
//         ChannelAllocated(
//             self.0
//                 .into_inner()
//                 .into_iter()
//                 .map(|(time_interval, timeline)| {
//                     (
//                         time_interval,
//                         timeline.allocate(slot_key_generator_type_map),
//                     )
//                 })
//                 .collect(),
//         )
//     }
// }

// struct ChannelAllocated<SKF, MP>(
//     Vec<(
//         Range<Time>,
//         Box<dyn TimelineAllocationErasure<SKF, MobjectPresentation = MP>>,
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
//     ) -> Vec<PresentationKey<Self::MobjectPresentation, SKF>> {
//         self.0
//             .iter()
//             .filter_map(|(time_interval, timeline)| {
//                 time_interval.contains(&time).then(|| {
//                     timeline.prepare(
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
//     fn push<T>(&self, time_interval: Range<Rc<Time>>, timeline: T)
//     where
//         T: Timeline<MobjectPresentation = MP>,
//     {
//         if !Rc::ptr_eq(&time_interval.start, &time_interval.end) {
//             self.channel.0.borrow_mut().push((
//                 *time_interval.start..*time_interval.end,
//                 Box::new(timeline),
//             ))
//         }
//     }

//     fn start<M, TS>(&self, mobject: Arc<M>, timeline_state: TS) -> Alive<SKF, W, M, TS>
//     where
//         M: Mobject<MobjectPresentation = MP>,
//         TS: TimelineState<SKF, W, M>,
//     {
//         Alive {
//             attachment: self,
//             // index: usize,
//             spawn_time: self.context.timer.time(),
//             mobject,
//             timeline_state: Some(timeline_state),
//         }
//     }

//     pub fn config(&self) -> &Config {
//         self.context.config
//     }

//     #[must_use]
//     pub fn spawn<M>(&self, mobject: M) -> Alive<SKF, W, M, CollapsedTimelineState>
//     where
//         M: Mobject<MobjectPresentation = MP>,
//     {
//         self.start(Arc::new(mobject), CollapsedTimelineState)
//     }
// }

// #[derive(Deserialize, Serialize)]
// pub struct PreallocatedTimelineEntry<SKF, MP>
// where
//     MP: 'static,
//     SKF: 'static,
// {
//     time_interval: Range<Time>,
//     timeline: serde_traitobject::Box<
//         dyn PreallocatedTimeline<MobjectPresentation = MP, StorableKeyFn = SKF>,
//     >,
// }

// pub struct TimelineAllocationEntry<SKF, MP> {
//     time_interval: Range<Time>,
//     timeline: Box<dyn TimelineAllocation<MobjectPresentation = MP, StorableKeyFn = SKF>>,
// }

// impl<SKF, MP> PreallocatedTimelineEntry<SKF, MP> {
//     fn allocate(
//         self,
//         slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> TimelineAllocationEntry<SKF, MP> {
//         TimelineAllocationEntry {
//             time_interval: self.time_interval,
//             timeline: self
//                 .timeline
//                 .into_box()
//                 .allocate(slot_key_generator_type_map),
//         }
//     }
// }

// struct TimelineEntriesSink<'v>(Option<&'v mut Vec<WithTimeInterval<Box<dyn TimelineAllocation>>>>);

// impl Extend<Box<dyn TimelineAllocation>> for TimelineEntriesSink<'_> {
//     fn extend<I>(&mut self, iter: I)
//     where
//         I: IntoIterator<Item = Box<dyn TimelineAllocation>>,
//     {
//         if let Some(timeline_entries) = self.0.as_mut() {
//             timeline_entries.extend(iter)
//         }
//     }
// }

// trait TimelineState {
//     fn flush_timelines(
//         &mut self,
//         time_interval: Range<Time>,
//     ) -> Vec<WithTimeInterval<Box<dyn TimelineErasure>>>;
// }

pub struct Timer {
    alive_id_generator: RefCell<IterRangeFrom<usize>>,
    time: RefCell<Rc<Time>>,
}

impl Timer {
    fn new() -> Self {
        Self {
            alive_id_generator: RefCell::new(RangeFrom { start: 0 }.into_iter()),
            time: RefCell::new(Rc::new(0.0)),
        }
    }

    pub(crate) fn generate_alive_id(&self) -> usize {
        self.alive_id_generator.borrow_mut().next().unwrap()
    }

    pub(crate) fn time(&self) -> Rc<Time> {
        self.time.borrow().clone()
    }

    pub fn wait(&self, time: Time) {
        self.time.replace_with(|rc_time| Rc::new(**rc_time + time));
    }
}

// struct Context<'c, SKF, W> {
//     config: &'c Config,
//     timer: Timer,
//     world: W,
//     phantom: PhantomData<SKF>,
// }

// impl<'c, SKF, W> Context<'c, SKF, W>
// where
//     SKF: StorableKeyFn,
//     W: World,
// {
// fn new(config: &'c Config) -> Self {
//     Self {
//         config,
//         timer: Timer::new(),
//         world: W::new(),
//         phantom: PhantomData,
//     }
// }

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

// fn collect(self) -> Vec<PreallocatedTimelineEntry<MP>> {
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
// }

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
//     timeline_entries_stack: RefCell<nonempty::NonEmpty<Vec<PreallocatedTimelineEntry<MP>>>>,
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
//             timeline_entries_stack: RefCell::new(nonempty::NonEmpty::singleton(Vec::new())),
//         }
//     }

//     pub fn grow_stack(&self) {
//         self.timeline_entries_stack.borrow_mut().push(Vec::new());
//     }

//     pub fn shrink_stack<TE>(&self, time_interval: Range<Time>, time_eval: &TE)
//     where
//         TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
//     {
//         let child_time_interval = 0.0..self.timer_stack.time();
//         let mut timeline_entries_stack = self.timeline_entries_stack.borrow_mut();
//         let timeline_entries = timeline_entries_stack.pop().unwrap();
//         timeline_entries_stack
//             .last_mut()
//             .extend(
//                 timeline_entries
//                     .into_iter()
//                     .map(|timeline_entry| PreallocatedTimelineEntry {
//                         time_interval: time_interval.start
//                             + (time_interval.end - time_interval.start)
//                                 * *time_eval.time_eval(
//                                     timeline_entry.time_interval.start,
//                                     child_time_interval.clone(),
//                                 )
//                             ..time_interval.start
//                                 + (time_interval.end - time_interval.start)
//                                     * *time_eval.time_eval(
//                                         timeline_entry.time_interval.end,
//                                         child_time_interval.clone(),
//                                     ),
//                         timeline: timeline_entry.timeline,
//                     }),
//             );
//     }

//     pub fn collect(self) -> Vec<PreallocatedTimelineEntry<MP>> {
//         let timeline_entries_stack = self.timeline_entries_stack.into_inner();
//         assert!(timeline_entries_stack.tail.is_empty());
//         timeline_entries_stack.head
//     }

//     fn world(&self) -> Rc<W> {
//         self.world.upgrade().unwrap()
//     }

//     // fn layer(&self) -> Rc<L> {
//     //     self.layer.upgrade().unwrap()
//     // }

//     fn push<T>(&self, time_interval: Range<Time>, timeline: T)
//     where
//         T: Timeline<MobjectPresentation = MP>,
//         Box<T>: Storable,
//     {
//         if time_interval.start < time_interval.end {
//             self.timeline_entries_stack
//                 .borrow_mut()
//                 .last_mut()
//                 .push(PreallocatedTimelineEntry {
//                     time_interval,
//                     timeline: serde_traitobject::Box::new(timeline),
//                 });
//         }
//     }

//     fn start<TS>(&self, timeline_state: TS) -> Alive<W, TS>
//     where
//         TS: TimelineState<MobjectPresentation = MP>,
//     {
//         Alive {
//             attachment: self,
//             // index: usize,
//             spawn_time: self.timer_stack.time(),
//             timeline_state: Some(timeline_state),
//         }
//     }

//     #[must_use]
//     pub fn spawn<M>(&self, mobject: M) -> Alive<W, M, CollapsedTimelineState>
//     where
//         M: Mobject<MobjectPresentation = MP>,
//     {
//         self.start(CollapsedTimelineState {
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
//     type StorableKeyFn: StorableKeyFn;

//     fn new(config: &Config, timer_stack: &TimerStack) -> Rc<Self>;
//     fn grow_stack(&self);
//     fn shrink_stack<TE>(&self, time_interval: Range<Time>, time_eval: &TE)
//     where
//         TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>;
//     fn collect(self) -> Vec<Box<dyn LayerPreallocated>>;
// }

// pub trait WorldEntry {
//     type StorableKeyFn: StorableKeyFn;

//     fn new(config: &Config, timer_stack: &TimerStack) -> Self;
//     fn merge<TE>(&mut self, world_entry: Self, time_interval: Range<Time>, time_eval: &TE)
//     where
//         TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>;
//     fn collect(self) -> Vec<Box<dyn LayerEntry>>;
// }

pub trait TypeQuery:
    'static + Clone + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
{
    type World: WorldIndexed<Self::LayerIndex, Layer = Self::Layer>;
    type LayerIndex: LayerIndex;
    type Layer: LayerIndexed<Self::ChannelIndex, Channel = Self::Channel>;
    type ChannelIndex: ChannelIndex;
    type Channel: Channel<MobjectPresentation = Self::MobjectPresentation>;
    type Mobject: Mobject;
    type MobjectPresentation: MobjectPresentation<Self::Mobject>;
    type Attachment: ChannelAttachment<
        Self::World,
        Self::LayerIndex,
        Self::Layer,
        Self::ChannelIndex,
        Self::Channel,
        Self::MobjectPresentation,
    >;

    // fn mobject(&self) -> &Arc<Self::Mobject>;
    // fn update<F>(self, f: F) -> Self
    // where
    //     F: FnOnce(&mut Self::Mobject);
    // fn attachment(&self) -> &Self::Attachment;
}

#[derive(Deserialize, Serialize)]
pub struct TypeQueried<W, LI, L, CI, C, M, MP, A>(PhantomData<fn() -> (W, LI, L, CI, C, M, MP, A)>);

impl<W, LI, L, CI, C, M, MP, A> Default for TypeQueried<W, LI, L, CI, C, M, MP, A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<W, LI, L, CI, C, M, MP, A> Clone for TypeQueried<W, LI, L, CI, C, M, MP, A> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

impl<W, LI, L, CI, C, M, MP, A> Debug for TypeQueried<W, LI, L, CI, C, M, MP, A> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl<W, LI, L, CI, C, M, MP, A> TypeQuery for TypeQueried<W, LI, L, CI, C, M, MP, A>
where
    W: WorldIndexed<LI, Layer = L>,
    LI: LayerIndex,
    L: LayerIndexed<CI, Channel = C>,
    CI: ChannelIndex,
    C: Channel<MobjectPresentation = MP>,
    M: Mobject,
    MP: MobjectPresentation<M>,
    A: ChannelAttachment<W, LI, L, CI, C, MP>,
{
    type World = W;
    type LayerIndex = LI;
    type Layer = L;
    type ChannelIndex = CI;
    type Channel = C;
    type Mobject = M;
    type MobjectPresentation = MP;
    type Attachment = A;

    // fn mobject(&self) -> &Arc<Self::Mobject> {
    //     &self.mobject
    // }

    // fn update<F>(self, f: F) -> Self
    // where
    //     F: FnOnce(&mut Self::Mobject),
    // {
    //     let mut mobject = Arc::unwrap_or_clone(self.mobject);
    //     f(&mut mobject);
    //     Self {
    //         mobject: Arc::new(mobject),
    //         phantom: PhantomData,
    //     }
    // }
}

// pub trait CompatibleAttachment<TQ>:
//     ChannelAttachment<
//     TQ::World,
//     TQ::LayerIndex,
//     TQ::Layer,
//     TQ::ChannelIndex,
//     TQ::Channel,
//     TQ::MobjectPresentation,
//     TQ::StorableKeyFn,
// >
// where
//     TQ: TypeQuery,
// {

//     // pub fn spawn_mobject<M>(&self, mobject: Box<M>) -> Alive<MobjectLocate<>>
// }

pub struct AttachedMobject<'a, TQ>
where
    TQ: TypeQuery,
{
    mobject: Arc<TQ::Mobject>,
    attachment: &'a TQ::Attachment,
}

impl<'a, TQ> AttachedMobject<'a, TQ>
where
    TQ: TypeQuery,
{
    pub(crate) fn new(mobject: Arc<TQ::Mobject>, attachment: &'a TQ::Attachment) -> Self {
        Self {
            mobject,
            attachment,
        }
    }

    pub(crate) fn launch<TS>(self, timeline_state: TS) -> Alive<'a, TQ, TS>
    where
        TS: TimelineState<TQ>,
    {
        Alive(Some(AliveInner {
            alive_id: self.attachment.timer().generate_alive_id(),
            spawn_time: self.attachment.timer().time(),
            attached_mobject: self,
            timeline_state,
        }))
    }

    // pub(crate) fn alive_context(&self) -> &'ac AC {
    //     &self.alive_context
    // }

    // pub(crate) fn archive_state(&self) -> &AS {
    //     self.archive_state.as_ref().unwrap()
    // }

    // fn record<T>(&self, alive_id: usize, time_interval: Range<Rc<Time>>, timeline: T)
    // where
    //     T: Timeline<
    //         MobjectPresentation = TQ::MobjectPresentation,
    //         StorableKeyFn = TQ::StorableKeyFn,
    //     >,
    // {
    //     if !Rc::ptr_eq(&time_interval.start, &time_interval.end) {
    //         self.attachment.channel_architecture().push(
    //             alive_id,
    //             Node::Singleton((
    //                 Range {
    //                     start: *time_interval.start,
    //                     end: *time_interval.end,
    //                 },
    //                 Box::new(timeline),
    //             )),
    //         );
    //     }
    // }

    // fn record_multiple<TE>(
    //     &self,
    //     alive_id: usize,
    //     time_interval: Range<Rc<Time>>,
    //     time_eval: &TE,
    //     world_time: Time,
    //     world_architecture: <TQ::World as World>::Architecture<TQ::StorableKeyFn>,
    // ) where
    //     TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    // {
    //     if !Rc::ptr_eq(&time_interval.start, &time_interval.end) {
    //         TQ::World::merge(
    //             self.attachment.world_architecture(),
    //             TQ::World::archive(world_architecture),
    //             alive_id,
    //             time_eval,
    //             Range {
    //                 start: *time_interval.start,
    //                 end: *time_interval.end,
    //             },
    //             Range {
    //                 start: 0.0,
    //                 end: world_time,
    //             },
    //         );
    //     }
    // }

    fn record<F>(&self, time_interval: Range<Rc<Time>>, f: F)
    where
        F: FnOnce(Arc<TQ::Mobject>, &TQ::Attachment, Range<Time>),
    {
        if !Rc::ptr_eq(&time_interval.start, &time_interval.end) {
            f(
                self.mobject.clone(),
                self.attachment,
                Range {
                    start: *time_interval.start,
                    end: *time_interval.end,
                },
            )
        }
    }

    // fn refresh_mobject<F>(&mut self, f: F) -> Alive<'a, TQ, TS>
    // where
    //     TS: Clone,
    //     F: FnOnce(&mut TQ::Mobject),
    // {
    //     let new_timeline_state = self.pair.as_ref().unwrap().1.clone();
    //     self.attachment
    //         .launch(self.terminate().update(f), new_timeline_state)
    // }

    // fn refresh_timeline_state<TSO>(&mut self, new_timeline_state: TSO) -> Alive<'a, TQ, TSO>
    // where
    //     TSO: TimelineState<TQ>,
    // {
    //     self.attachment.launch(self.terminate(), new_timeline_state)
    // }
}

struct AliveInner<'a, TQ, TS>
where
    TQ: TypeQuery,
    TS: TimelineState<TQ>,
{
    alive_id: usize,
    spawn_time: Rc<Time>,
    attached_mobject: AttachedMobject<'a, TQ>,
    timeline_state: TS,
}

pub struct Alive<'a, TQ, TS>(Option<AliveInner<'a, TQ, TS>>)
where
    TQ: TypeQuery,
    TS: TimelineState<TQ>;

impl<'a, TQ, TS> Alive<'a, TQ, TS>
where
    TQ: TypeQuery,
    TS: TimelineState<TQ>,
{
    fn terminate(&mut self) -> AttachedMobject<TS::OutputTypeQuery> {
        // let mut recorder = self.alive_recorder.recorder.borrow_mut();
        // let entry = recorder.get_mut(self.index).unwrap();
        let inner = self.0.take().unwrap();
        inner.timeline_state.transit(
            inner.alive_id,
            Range {
                start: inner.spawn_time.clone(),
                end: inner.attached_mobject.attachment.timer().time(),
            },
            inner.attached_mobject,
        )
    }
}

impl<TQ, TS> Drop for Alive<'_, TQ, TS>
where
    TQ: TypeQuery,
    TS: TimelineState<TQ>,
{
    fn drop(&mut self) {
        if self.0.is_some() {
            self.terminate();
        }
    }
}

pub trait TimelineState<TQ>
where
    TQ: TypeQuery,
{
    type OutputTypeQuery: TypeQuery;
    // type OutputLayerIndex;
    // type OutputChannelIndex;
    // type OutputMobject;

    fn transit(
        self,
        alive_id: usize,
        time_interval: Range<Rc<Time>>,
        attached_mobject: AttachedMobject<TQ>,
    ) -> AttachedMobject<Self::OutputTypeQuery>;
}

#[derive(Clone)] // TODO: get rid of it
pub struct CollapsedTimelineState;

impl<TQ> TimelineState<TQ> for CollapsedTimelineState
where
    TQ: TypeQuery,
{
    // type MobjectPresentation = M::MobjectPresentation;
    type OutputTypeQuery = TQ;

    fn transit(
        self,
        alive_id: usize,
        time_interval: Range<Rc<Time>>,
        attached_mobject: AttachedMobject<TQ>,
    ) -> AttachedMobject<Self::OutputTypeQuery> {
        attached_mobject.record(time_interval, |mobject, attachment, time_interval| {
            attachment.channel_architecture().push(
                alive_id,
                Node::Singleton((time_interval, Box::new(StaticTimeline::<TQ> { mobject }))),
            );
        });
        attached_mobject
    }
}

pub struct IndeterminedTimelineState<TE> {
    time_eval: TE,
}

impl<TQ, TE> TimelineState<TQ> for IndeterminedTimelineState<TE>
where
    TQ: TypeQuery,
    TE: TimeEval,
{
    // type MobjectPresentation = M::MobjectPresentation;
    type OutputTypeQuery = TQ;

    fn transit(
        self,
        alive_id: usize,
        time_interval: Range<Rc<Time>>,
        attached_mobject: AttachedMobject<TQ>,
    ) -> AttachedMobject<Self::OutputTypeQuery> {
        attached_mobject.record(time_interval, |mobject, attachment, time_interval| {
            attachment.channel_architecture().push(
                alive_id,
                Node::Singleton((time_interval, Box::new(StaticTimeline::<TQ> { mobject }))),
            );
        });
        attached_mobject
    }
}

pub struct UpdateTimelineState<TE, U> {
    // mobject: Arc<M>,
    time_eval: TE,
    update: U,
}

impl<TQ, TE, U> TimelineState<TQ> for UpdateTimelineState<TE, U>
where
    TQ: TypeQuery,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, TQ>,
{
    type OutputTypeQuery = TQ;

    fn transit(
        self,
        alive_id: usize,
        time_interval: Range<Rc<Time>>,
        attached_mobject: AttachedMobject<TQ>,
    ) -> AttachedMobject<Self::OutputTypeQuery> {
        attached_mobject.record(time_interval, |mobject, attachment, time_interval| {
            attachment.channel_architecture().push(
                alive_id,
                Node::Singleton((
                    time_interval,
                    Box::new(DynamicTimeline {
                        mobject,
                        time_eval: self.time_eval,
                        update: self.update,
                    }),
                )),
            );
        });
        attached_mobject
    }

    // type OutputTimelineState = M, CollapsedTimelineState;

    // fn into_next(
    //     self,
    //     supervisor: &Supervisor,
    //     time_interval: Range<Rc<Time>>,
    //     mut timeline_entries_sink: TimelineEntriesSink,
    // ) -> Self::OutputTimelineState {
    //     timeline_entries_sink.extend_one(supervisor.new_dynamic_timeline_entry(
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
    //     CollapsedTimelineState {
    //         mobject: Arc::new(mobject),
    //     }
    // }
}

pub struct ConstructTimelineState<TE, C> {
    time_eval: TE,
    construct: C,
}

impl<TQ, TE, C> TimelineState<TQ> for ConstructTimelineState<TE, C>
where
    TQ: TypeQuery,
    TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    C: Construct<TQ>,
{
    type OutputTypeQuery = C::OutputTypeQuery;

    fn transit(
        self,
        alive_id: usize,
        time_interval: Range<Rc<Time>>,
        attached_mobject: AttachedMobject<TQ>,
    ) -> AttachedMobject<Self::OutputTypeQuery> {
        let timer = Timer::new();
        let world_architecture = TQ::World::architecture();
        let output_mobject = {
            let world_attachment =
                World::attachment(&world_architecture, attachment.config(), &timer);
            self.construct
                .construct(
                    &world_attachment,
                    attachment.config(),
                    &timer,
                    mobject_query
                        .mobject()
                        .clone()
                        .spawn(WorldIndexed::index::<TQ::LayerIndex>(&world_architecture)),
                )
                .terminate()
        };
        attachment.record_multiple(
            alive_id,
            time_interval,
            &self.time_eval,
            *timer.time(),
            world_architecture,
        );
        // let world = world.as_ref();
        // world.grow_stack();
        // let output_mobject = self
        //     .construct
        //     .take()
        //     .unwrap()
        //     .construct(
        //         world,
        //         attachment.start(CollapsedTimelineState {
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
    //         timeline_entries,
    //         ..
    //     } in renderables.as_mut_slice()
    //     {
    //         rescale_time_interval(time_interval);
    //         for TimelineEntry { time_interval, .. } in timeline_entries.as_mut_slice() {
    //             rescale_time_interval(time_interval);
    //         }
    //     }
    //     global_archive.borrow_mut().1.extend(renderables.drain(..));
    // }

    // type OutputTimelineState = CollapsedTimelineState<C::OutputMobject>;

    // fn into_next(
    //     self,
    //     supervisor: &Supervisor,
    //     time_interval: Range<Time>,
    //     mut timeline_entries_sink: TimelineEntriesSink,
    // ) -> Self::OutputTimelineState {
    //     let child_supervisor = Supervisor::new(supervisor.config);
    //     let output_timeline_state = child_supervisor
    //         .end(&self.construct.construct(
    //             child_supervisor.start(AliveContent {
    //                 spawn_time: child_supervisor.time.borrow().clone(),
    //                 timeline_state: CollapsedTimelineState {
    //                     mobject: self.mobject,
    //                 },
    //             }),
    //             &child_supervisor,
    //         ))
    //         .timeline_state;
    //     let children_time_interval = child_supervisor.time_interval();
    //     timeline_entries_sink.extend(child_supervisor.iter_timeline_entries().map(
    //         |mut timeline_entry| {
    //             timeline_entry.time_interval = time_interval.start
    //                 + (time_interval.end - time_interval.start)
    //                     * *self.time_eval.time_eval(
    //                         timeline_entry.time_interval.start,
    //                         children_time_interval.clone(),
    //                     )
    //                 ..time_interval.start
    //                     + (time_interval.end - time_interval.start)
    //                         * *self.time_eval.time_eval(
    //                             timeline_entry.time_interval.end,
    //                             children_time_interval.clone(),
    //                         );
    //             timeline_entry
    //         },
    //     ));
    //     output_timeline_state
    // }

    // fn into_next(
    //     self,
    //     time_interval: Range<f32>,
    //     _mobject: Arc<M>,
    //     _storage: &S,
    // ) -> Vec<TimelineEntry> {
    //     let mut timeline_entries = self.children_timeline_entries;
    //     timeline_entries.iter_mut().for_each(|timeline_entry| {
    //         timeline_entry.rescale_time_interval(
    //             &self.time_eval,
    //             &self.children_time_interval,
    //             &time_interval,
    //         )
    //     });
    //     timeline_entries
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
//     TS: TimelineState,
// {
// }

// pub struct Supervisor<'c> {
//     config: &'c Config,
//     // storage: &'s Storage,
//     time: RefCell<Arc<Time>>,
//     timeline_slots: RefCell<Vec<Result<Vec<Box<dyn TimelineAllocation>>, Arc<dyn Any>>>>,
// }

// impl<L, MB> IntoArchiveState<AliveRenderable<'_, '_, LayerRenderableState<L>>> for MB
// where
//     W: World,
//     MB: MobjectBuilder<L>,
// {
//     type ArchiveState = CollapsedTimelineState<MB::Instantiation>;

//     fn into_archive_state(
//         self,
//         alive_context: &AliveRenderable<'_, '_, LayerRenderableState<L>>,
//     ) -> Self::ArchiveState {
//         CollapsedTimelineState {
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
    'static + Clone + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
{
    type OutputTimeMetric: TimeMetric;

    fn time_eval(&self, time: Time, time_interval: Range<Time>) -> Self::OutputTimeMetric;
}

pub trait IncreasingTimeEval: TimeEval {}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
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

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DenormalizedTimeEval;

impl TimeEval for DenormalizedTimeEval {
    type OutputTimeMetric = DenormalizedTimeMetric;

    fn time_eval(&self, time: Time, time_interval: Range<Time>) -> Self::OutputTimeMetric {
        DenormalizedTimeMetric(time - time_interval.start)
    }
}

impl IncreasingTimeEval for DenormalizedTimeEval {}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct RateComposeTimeEval<RA, TE> {
    rate: RA,
    time_eval: TE,
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

pub trait ApplyUpdate<TM, TQ>: Sized
where
    TM: TimeMetric,
    TQ: TypeQuery,
{
    type Output<U>
    where
        U: Update<TM, TQ>;

    fn apply_update<U>(self, update: U) -> Self::Output<U>
    where
        U: Update<TM, TQ>;
}

pub trait ApplyConstruct<TQ>: Sized
where
    TQ: TypeQuery,
{
    type Output<C>
    where
        C: Construct<TQ>;

    fn apply_construct<C>(self, construct: C) -> Self::Output<C>
    where
        C: Construct<TQ>;
}

impl<'a, TQ> Quantize for Alive<'a, TQ, CollapsedTimelineState>
where
    TQ: TypeQuery,
{
    type Output<TE> =
        Alive<'a, TQ, IndeterminedTimelineState<TE>>
    where
        TE: TimeEval;

    #[must_use]
    fn quantize<TE>(mut self, time_eval: TE) -> Self::Output<TE>
    where
        TE: TimeEval,
    {
        self.refresh_timeline_state(IndeterminedTimelineState { time_eval })
    }
}

impl<'a, TQ, TE, U> Collapse for Alive<'a, TQ, UpdateTimelineState<TE, U>>
where
    TQ: TypeQuery,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, TQ>,
{
    type Output = Alive<'a, TQ, CollapsedTimelineState>;

    #[must_use]
    fn collapse(mut self) -> Self::Output {
        self.refresh_timeline_state(CollapsedTimelineState)
    }
}

impl<'a, TQ, TE, C> Collapse for Alive<'a, TQ, ConstructTimelineState<TE, C>>
where
    TQ: TypeQuery,
    TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    C: Construct<TQ>,
{
    type Output = Alive<'a, TQ, CollapsedTimelineState>;

    #[must_use]
    fn collapse(mut self) -> Self::Output {
        self.refresh_timeline_state(CollapsedTimelineState)
    }
}

impl<'a, TQ, TE> ApplyRate<TE::OutputTimeMetric> for Alive<'a, TQ, IndeterminedTimelineState<TE>>
where
    TQ: TypeQuery,
    TE: TimeEval,
{
    type Output<RA> =
        Alive<'a, TQ, IndeterminedTimelineState<RateComposeTimeEval<RA, TE>>>
    where
        RA: Rate<TE::OutputTimeMetric>;

    #[must_use]
    fn apply_rate<RA>(mut self, rate: RA) -> Self::Output<RA>
    where
        RA: Rate<TE::OutputTimeMetric>,
    {
        self.refresh_timeline_state(IndeterminedTimelineState {
            time_eval: RateComposeTimeEval {
                rate,
                time_eval: self.time_eval.clone(),
            },
        })
    }
}

impl<'a, TQ, TE> ApplyUpdate<TE::OutputTimeMetric, TQ>
    for Alive<'a, TQ, IndeterminedTimelineState<TE>>
where
    TQ: TypeQuery,
    TE: TimeEval,
{
    type Output<U> =
        Alive<'a, TQ, UpdateTimelineState<TE, U>>
    where
        U: Update<TE::OutputTimeMetric, TQ>;

    #[must_use]
    fn apply_update<U>(mut self, update: U) -> Self::Output<U>
    where
        U: Update<TE::OutputTimeMetric, TQ>,
    {
        self.refresh_timeline_state(|timeline_state| UpdateTimelineState {
            // mobject: timeline_state.mobject,
            time_eval: timeline_state.time_eval.clone(),
            update,
        })
    }
}

impl<'a, TQ> ApplyUpdate<NormalizedTimeMetric, TQ> for Alive<'a, TQ, CollapsedTimelineState>
where
    TQ: TypeQuery,
{
    type Output<U> =
        Alive<'a, TQ, CollapsedTimelineState>
    where
        U: Update<NormalizedTimeMetric, TQ>;

    #[must_use]
    fn apply_update<U>(mut self, update: U) -> Self::Output<U>
    where
        U: Update<NormalizedTimeMetric, TQ>,
    {
        self.refresh_mobject(|mobject| {
            update.update(NormalizedTimeMetric(1.0), mobject);
            // let mut mobject = Arc::unwrap_or_clone(timeline_state.mobject);
            // update.update(NormalizedTimeMetric(1.0), &mut mobject);
            // CollapsedTimelineState {
            //     mobject: Arc::new(mobject),
            // }
        })
    }
}

impl<'a, TQ, TE> ApplyConstruct<TQ> for Alive<'a, TQ, IndeterminedTimelineState<TE>>
where
    TQ: TypeQuery,
    TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
{
    type Output<C> =
        Alive<'a, TQ, ConstructTimelineState<TE, C>>
    where
        C: Construct<TQ>;

    #[must_use]
    fn apply_construct<C>(mut self, construct: C) -> Self::Output<C>
    where
        C: Construct<TQ>,
    {
        self.refresh_timeline_state(|alive_context, timeline_state| {
            let child_root = AliveRoot::new(alive_context.alive_context().config());
            let child_renderable = child_root.start(LayerRenderableState {
                layer: alive_context.archive_state().layer.clone(),
            });
            let child_timeline = child_renderable.start(CollapsedTimelineState {
                mobject: timeline_state.mobject.clone(),
            });
            let mobject = construct
                .construct(&child_root, &child_renderable, child_timeline)
                .archive_state()
                .mobject
                .clone();
            drop(child_renderable);
            ConstructTimelineState {
                mobject,
                time_eval: timeline_state.time_eval.clone(),
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
//     //     self.0.get::<StaticTimeline<M>>()
//     // }
// }
