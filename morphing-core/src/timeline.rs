use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::Range;
use std::rc::Rc;
use std::rc::Weak;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use serde::Deserialize;
use serde::Serialize;

use super::config::Config;
use super::storable::Allocated;
use super::storable::MutexSlot;
use super::storable::SharableSlot;
use super::storable::Slot;
use super::storable::SlotKeyGeneratorTypeMap;
use super::storable::Storable;
use super::storable::StorageTypeMap;
use super::storable::SwapSlot;
use super::traits::Construct;
use super::traits::IncreasingRate;
use super::traits::Mobject;
use super::traits::MobjectBuilder;
use super::traits::Rate;
use super::traits::Render;
use super::traits::SerializableKeyFn;
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

pub enum PresentationCell<P> {
    Sharable(Arc<P>),
    Mutex(Arc<Mutex<P>>),
}

impl<P> AsRef<P> for PresentationCell<P> {
    fn as_ref(&self) -> &P {
        match self {
            Self::Sharable(presentation) => &presentation,
            Self::Mutex(presentation) => &presentation.lock().unwrap(),
        }
    }
} // TODO

trait Timeline: 'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
where
    Box<Self>: Storable,
{
    type MobjectPresentation;
    // type SerdeKey: 'static + Eq + Hash + Send + Sync;
    // type MobjectPresentationStorage: PresentationStorage;

    // fn serde_key(&self) -> Self::SerdeKey;
    fn init_presentation(
        &self,
        device: &wgpu::Device,
    ) -> <<Box<Self> as Storable>::Slot as Slot>::Value;
    fn fetch_presentation(
        &self,
        mobject_presentation: &<<Box<Self> as Storable>::Slot as Slot>::Value,
    ) -> PresentationCell<Self::MobjectPresentation>;
    fn prepare_presentation(
        &self,
        time: Time,
        time_interval: Range<Time>,
        mobject_presentation: &mut <<Box<Self> as Storable>::Slot as Slot>::Value,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    );
    // fn erased(&self) -> Box<dyn Timeline<PresentationStorage = Self::PresentationStorage>>;

    // let slot_map = storage
    //     .entry::<AllocatedTimeline<Self::SerdeKey, Self::MobjectPresentationStorage>>()
    //     .or_insert_with(HashMap::new);
    // let serde_key = self.serde_key();
    // let slot_id = slot_map
    //     .entry(serde_key.clone())
    //     .or_insert_with(Self::MobjectPresentationStorage::new)
    //     .allocate();
    // AllocatedTimeline {
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
struct StaticTimeline<M, SKF> {
    // mobject_serde_key: serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    // slot_id: SI,
    // time_interval: Range<Time>,
    mobject: Arc<M>,
    phantom: PhantomData<SKF>,
}

impl<M, SKF> Storable for Box<StaticTimeline<M, SKF>>
where
    M: Mobject,
    SKF: SerializableKeyFn,
{
    type StorableKey = SKF::Output;
    type Slot = SwapSlot<SharableSlot<M::MobjectPresentation>>;

    fn key(&self) -> Self::StorableKey {
        SKF::eval_key(&self.mobject)
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

impl<M, SKF> Timeline for StaticTimeline<M, SKF>
where
    M: Mobject,
    SKF: SerializableKeyFn,
{
    // type SerdeKey = serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>;
    // type MobjectPresentationStorage = ;

    // fn serde_key(&self) -> Self::SerdeKey {
    //     serde_hashkey::to_key_with_ordered_float(self.mobject.as_ref()).unwrap()
    // }

    type MobjectPresentation = M::MobjectPresentation;

    fn init_presentation(
        &self,
        device: &wgpu::Device,
    ) -> <<Box<Self> as Storable>::Slot as Slot>::Value {
        Arc::new(self.mobject.presentation(device))
    }

    fn fetch_presentation(
        &self,
        mobject_presentation: &<<Box<Self> as Storable>::Slot as Slot>::Value,
    ) -> PresentationCell<Self::MobjectPresentation> {
        PresentationCell::Sharable(mobject_presentation.clone())
    }

    fn prepare_presentation(
        &self,
        _time: Time,
        _time_interval: Range<Time>,
        _mobject_presentation: &mut <<Box<Self> as Storable>::Slot as Slot>::Value,
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
struct DynamicTimeline<M, TE, U, SKF> {
    // mobject_serde_key: serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    // update_serde_key: serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    // slot_id: SI,
    // time_interval: Range<Time>,
    mobject: Arc<M>,
    time_eval: Arc<TE>,
    update: Arc<U>,
    phantom: PhantomData<SKF>,
}

impl<M, TE, U, SKF> Storable for Box<DynamicTimeline<M, TE, U, SKF>>
where
    M: Mobject,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, M>,
    SKF: SerializableKeyFn,
{
    type StorableKey = (SKF::Output, SKF::Output);
    type Slot = SwapSlot<MutexSlot<M::MobjectPresentation>>;

    fn key(&self) -> Self::StorableKey {
        (SKF::eval_key(&self.mobject), SKF::eval_key(&self.update))
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

impl<M, TE, U, SKF> Timeline for DynamicTimeline<M, TE, U, SKF>
where
    M: Mobject,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, M>,
    SKF: SerializableKeyFn,
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

    fn init_presentation(
        &self,
        device: &wgpu::Device,
    ) -> <<Box<Self> as Storable>::Slot as Slot>::Value {
        Arc::new(Mutex::new(self.mobject.presentation(device)))
    }

    fn fetch_presentation(
        &self,
        mobject_presentation: &<<Box<Self> as Storable>::Slot as Slot>::Value,
    ) -> PresentationCell<Self::MobjectPresentation> {
        PresentationCell::Mutex(mobject_presentation.clone())
    }

    fn prepare_presentation(
        &self,
        time: Time,
        time_interval: Range<Time>,
        mobject_presentation: &mut <<Box<Self> as Storable>::Slot as Slot>::Value,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        self.update.prepare_presentation(
            self.time_eval.time_eval(time, time_interval),
            &self.mobject,
            mobject_presentation.as_ref().lock().as_mut().unwrap(),
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
// struct AllocatedTimeline<SK, MPS>
// where
//     SK: SerdeKey,
//     MPS: MobjectPresentationStorage,
// {
//     // time_interval: Range<Time>,
//     serde_key: SK,        //T::SerdeKey,
//     slot_id: MPS::SlotId, //<T::MobjectPresentationStorage as MobjectPresentationStorage>::SlotId,
//     timeline: Box<dyn Timeline<SerdeKey = SK, MobjectPresentationStorage = MPS>>,
// }

// impl<SK, MPS> typemap_rev::TypeMapKey for AllocatedTimeline<SK, MPS>
// where
//     SK: SerdeKey,
//     MPS: MobjectPresentationStorage,
// {
//     type Value = HashMap<SK, MPS>;
// }

// impl<SK, MPS> AllocatedTimeline<SK, MPS>
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

pub trait PreallocatedTimeline:
    serde_traitobject::Deserialize + serde_traitobject::Serialize
{
    type MobjectPresentation;

    fn allocate(
        self: Box<Self>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Box<dyn AllocatedTimeline<MobjectPresentation = Self::MobjectPresentation>>;
}

impl<T> PreallocatedTimeline for T
where
    T: Timeline,
    Box<T>: Storable,
{
    type MobjectPresentation = T::MobjectPresentation;

    fn allocate(
        self: Box<Self>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Box<dyn AllocatedTimeline<MobjectPresentation = Self::MobjectPresentation>> {
        Box::new(slot_key_generator_type_map.allocate(self))
    }
}

pub trait AllocatedTimeline {
    type MobjectPresentation;

    // fn fetch_presentation(
    //     &self,
    //     storage_type_map: &StorageTypeMap,
    // ) -> PresentationCell<Self::MobjectPresentation>;
    fn prepare(
        &self,
        time: Time,
        time_interval: Range<f32>,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> PresentationCell<Self::MobjectPresentation>;
}

impl<T> AllocatedTimeline for Allocated<Box<T>>
where
    T: Timeline,
    Box<T>: Storable,
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
    ) -> PresentationCell<Self::MobjectPresentation> {
        let mobject_presentation = storage_type_map
            .get_or_insert_with(self, |storable| storable.init_presentation(device));
        self.storable().prepare_presentation(
            time,
            time_interval,
            mobject_presentation,
            device,
            queue,
            format,
        );
        self.storable().fetch_presentation(mobject_presentation)
    }
}

#[derive(Deserialize, Serialize)]
pub struct PreallocatedTimelineEntry<MP>
where
    MP: 'static,
{
    time_interval: Range<Time>,
    timeline: serde_traitobject::Box<dyn PreallocatedTimeline<MobjectPresentation = MP>>,
}

pub struct AllocatedTimelineEntry<MP> {
    time_interval: Range<Time>,
    timeline: Box<dyn AllocatedTimeline<MobjectPresentation = MP>>,
}

impl<MP> PreallocatedTimelineEntry<MP> {
    fn allocate(
        self,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> AllocatedTimelineEntry<MP> {
        AllocatedTimelineEntry {
            time_interval: self.time_interval,
            timeline: self
                .timeline
                .into_box()
                .allocate(slot_key_generator_type_map),
        }
    }
}

// struct TimelineEntriesSink<'v>(Option<&'v mut Vec<WithTimeInterval<Box<dyn AllocatedTimeline>>>>);

// impl Extend<Box<dyn AllocatedTimeline>> for TimelineEntriesSink<'_> {
//     fn extend<I>(&mut self, iter: I)
//     where
//         I: IntoIterator<Item = Box<dyn AllocatedTimeline>>,
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
//     ) -> Vec<WithTimeInterval<Box<dyn TimelineErased>>>;
// }

#[derive(Default)]
pub struct Timer {
    time: RefCell<Time>,
}

impl Timer {
    fn time(&self) -> Time {
        *self.time.borrow()
    }

    pub fn wait(&self, time: Time) {
        self.time.replace_with(|time_ref| *time_ref + time);
    }
}

#[derive(Default)]
struct TimerStack(RefCell<nonempty::NonEmpty<Timer>>);

impl TimerStack {
    fn time(&self) -> Time {
        self.0.borrow().last().time()
    }

    fn grow_stack(&mut self) {
        self.0.borrow_mut().push(Timer::default());
    }

    fn shrink_stack(&mut self) {
        self.0.borrow_mut().push(Timer {
            time: RefCell::new(0.0),
        });
    }
}

pub struct LayerField<'lf, W, MP>
where
    MP: 'static,
{
    config: &'lf Config,
    timer_stack: &'lf TimerStack,
    world: Weak<W>,
    // layer: Weak<L>,
    // timer: &'lf Timer,
    // depth: &'lf RefCell<usize>,
    timeline_entries_stack: RefCell<nonempty::NonEmpty<Vec<PreallocatedTimelineEntry<MP>>>>,
}

impl<'lf, W, MP> LayerField<'lf, W, MP>
where
    W: World,
    MP: 'static,
{
    pub fn new(config: &'lf Config, timer_stack: &'lf TimerStack, world: Weak<W>) -> Self {
        Self {
            config,
            timer_stack,
            world,
            timeline_entries_stack: RefCell::new(nonempty::NonEmpty::singleton(Vec::new())),
        }
    }

    pub fn grow_stack(&self) {
        self.timeline_entries_stack.borrow_mut().push(Vec::new());
    }

    pub fn shrink_stack<TE>(&self, time_interval: Range<Time>, time_eval: &TE)
    where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    {
        let child_time_interval = 0.0..self.timer_stack.time();
        let mut timeline_entries_stack = self.timeline_entries_stack.borrow_mut();
        let timeline_entries = timeline_entries_stack.pop().unwrap();
        timeline_entries_stack
            .last_mut()
            .extend(
                timeline_entries
                    .into_iter()
                    .map(|timeline_entry| PreallocatedTimelineEntry {
                        time_interval: time_interval.start
                            + (time_interval.end - time_interval.start)
                                * *time_eval.time_eval(
                                    timeline_entry.time_interval.start,
                                    child_time_interval.clone(),
                                )
                            ..time_interval.start
                                + (time_interval.end - time_interval.start)
                                    * *time_eval.time_eval(
                                        timeline_entry.time_interval.end,
                                        child_time_interval.clone(),
                                    ),
                        timeline: timeline_entry.timeline,
                    }),
            );
    }

    pub fn collect(self) -> Vec<PreallocatedTimelineEntry<MP>> {
        let timeline_entries_stack = self.timeline_entries_stack.into_inner();
        assert!(timeline_entries_stack.tail.is_empty());
        timeline_entries_stack.head
    }

    fn world(&self) -> Rc<W> {
        self.world.upgrade().unwrap()
    }

    // fn layer(&self) -> Rc<L> {
    //     self.layer.upgrade().unwrap()
    // }

    fn push<T>(&self, time_interval: Range<Time>, timeline: T)
    where
        T: Timeline<MobjectPresentation = MP>,
        Box<T>: Storable,
    {
        if time_interval.start < time_interval.end {
            self.timeline_entries_stack
                .borrow_mut()
                .last_mut()
                .push(PreallocatedTimelineEntry {
                    time_interval,
                    timeline: serde_traitobject::Box::new(timeline),
                });
        }
    }

    fn start<TS>(&self, timeline_state: TS) -> Alive<W, TS>
    where
        TS: TimelineState<MobjectPresentation = MP>,
    {
        Alive {
            layer_field: self,
            // index: usize,
            spawn_time: self.timer_stack.time(),
            timeline_state: Some(timeline_state),
        }
    }

    #[must_use]
    pub fn spawn<M>(&self, mobject: M) -> Alive<W, CollapsedTimelineState<M>>
    where
        M: Mobject<MobjectPresentation = MP>,
    {
        self.start(CollapsedTimelineState {
            mobject: Arc::new(mobject),
        })
    }
}

pub trait Layer {
    type LayerPreallocated: LayerPreallocated;

    fn new<W>(config: &Config, timer_stack: &TimerStack, world: Weak<W>) -> Self
    where
        W: World;
    fn grow_stack(&self);
    fn shrink_stack<TE>(&self, time_interval: Range<Time>, time_eval: &TE)
    where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>;
    fn collect(self) -> Self::LayerPreallocated;
}

pub trait LayerPreallocated: serde_traitobject::Deserialize + serde_traitobject::Serialize {
    // type LayerAllocated: LayerAllocated;

    fn allocate(
        self,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Box<dyn LayerAllocated>;
}

pub trait LayerAllocated {
    // type LayerRender: Render;

    fn prepare(
        &self,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Box<dyn Render>;
    // fn render()
}

pub trait World {
    type SerializableKeyFn: SerializableKeyFn;

    fn new(config: &Config, timer_stack: &TimerStack) -> Rc<Self>;
    fn grow_stack(&self);
    fn shrink_stack<TE>(&self, time_interval: Range<Time>, time_eval: &TE)
    where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>;
    fn collect(self) -> Vec<Box<dyn LayerPreallocated>>;
}

// test code
struct MyMobjectPresentation0;
struct MyMobjectPresentation1;

struct MyLayer<'w, W>
where
    W: World,
{
    field_0: LayerField<'w, W, MyMobjectPresentation0>,
    field_1: LayerField<'w, W, MyMobjectPresentation1>,
}

#[derive(Deserialize, Serialize)]
struct MyLayerPreallocated {
    field_0: Vec<PreallocatedTimelineEntry<MyMobjectPresentation0>>,
    field_1: Vec<PreallocatedTimelineEntry<MyMobjectPresentation1>>,
}

struct MyLayerAllocated {
    field_0: Vec<AllocatedTimelineEntry<MyMobjectPresentation0>>,
    field_1: Vec<AllocatedTimelineEntry<MyMobjectPresentation1>>,
}

struct MyLayerRender {
    field_0: Vec<PresentationCell<MyMobjectPresentation0>>,
    field_1: Vec<PresentationCell<MyMobjectPresentation1>>,
}

impl Layer for MyLayer<'_> {
    type LayerPreallocated = MyLayerPreallocated;

    fn new<W>(config: &Config, timer_stack: &TimerStack, world: Weak<W>) -> Self
    where
        W: World,
    {
        Self {
            field_0: LayerField::new(config, timer_stack, world.clone()),
            field_1: LayerField::new(config, timer_stack, world.clone()),
        }
    }

    fn grow_stack(&self) {
        self.field_0.grow_stack();
        self.field_1.grow_stack();
    }

    fn shrink_stack<TE>(&self, time_interval: Range<Time>, time_eval: &TE)
    where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    {
        self.field_0.shrink_stack(time_interval.clone(), time_eval);
        self.field_1.shrink_stack(time_interval.clone(), time_eval);
    }

    fn collect(self) -> Self::LayerPreallocated {
        MyLayerPreallocated {
            field_0: self.field_0.collect(),
            field_1: self.field_1.collect(),
        }
    }
}

impl LayerPreallocated for MyLayerPreallocated {
    fn allocate(
        self,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Box<dyn LayerAllocated> {
        Box::new(MyLayerAllocated {
            field_0: self
                .field_0
                .into_iter()
                .map(|timeline_entry| timeline_entry.allocate(slot_key_generator_type_map))
                .collect(),
            field_1: self
                .field_1
                .into_iter()
                .map(|timeline_entry| timeline_entry.allocate(slot_key_generator_type_map))
                .collect(),
        })
    }
}

impl LayerAllocated for MyLayerAllocated {
    fn prepare(
        &self,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Box<dyn Render> {
        Box::new(MyLayerRender {
            field_0: self
                .field_0
                .iter()
                .filter_map(|timeline_entry| {
                    timeline_entry.time_interval.contains(&time).then(|| {
                        timeline_entry.timeline.prepare(
                            time,
                            timeline_entry.time_interval.clone(),
                            storage_type_map,
                            device,
                            queue,
                            format,
                        )
                    })
                })
                .collect(),
            field_1: self
                .field_1
                .iter()
                .filter_map(|timeline_entry| {
                    timeline_entry.time_interval.contains(&time).then(|| {
                        timeline_entry.timeline.prepare(
                            time,
                            timeline_entry.time_interval.clone(),
                            storage_type_map,
                            device,
                            queue,
                            format,
                        )
                    })
                })
                .collect(),
        })
    }
}

impl Render for MyLayerRender {
    fn render(&self, _encoder: &mut wgpu::CommandEncoder, _target: &wgpu::TextureView) {}
}

struct MyWorld<'w> {
    layer_0: MyLayer<'w>,
    layer_1: MyLayer<'w>,
}

#[derive(Debug)]
struct MySerializableKeyFn;

impl SerializableKeyFn for MySerializableKeyFn {
    type Output = ();

    fn eval_key<S>(_serializable: &S) -> Self::Output
    where
        S: serde::Serialize,
    {
        ()
    }
}

impl World for MyWorld<'_> {
    type SerializableKeyFn = MySerializableKeyFn;

    fn new(config: &Config, timer_stack: &TimerStack) -> Rc<Self> {
        Rc::new_cyclic(|world| Self {
            layer_0: MyLayer::new(config, timer_stack, world.clone()),
            layer_1: MyLayer::new(config, timer_stack, world.clone()),
        })
    }

    fn grow_stack(&self) {
        self.layer_0.grow_stack();
        self.layer_1.grow_stack();
    }
    fn shrink_stack<TE>(&self, time_interval: Range<Time>, time_eval: &TE)
    where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    {
        self.layer_0.shrink_stack(time_interval.clone(), time_eval);
        self.layer_1.shrink_stack(time_interval.clone(), time_eval);
    }

    fn collect(self) -> Vec<Box<dyn LayerPreallocated>> {
        Vec::from([
            Box::new(self.layer_0.collect()) as Box<dyn LayerPreallocated>,
            Box::new(self.layer_1.collect()) as Box<dyn LayerPreallocated>,
        ])
    }
}
// end test code

pub struct Alive<'lf, W, TS>
where
    W: World,
    TS: TimelineState,
{
    layer_field: &'lf LayerField<'lf, W, TS::MobjectPresentation>,
    spawn_time: Time,
    timeline_state: Option<TS>,
}

impl<'lf, W, TS> Alive<'lf, W, TS>
where
    W: World,
    TS: TimelineState,
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
        let archive_time = self.layer_field.timer_stack.time();
        let mut timeline_state = self.timeline_state.take().unwrap();
        timeline_state.archive(spawn_time..archive_time, self.layer_field)
    }

    pub(crate) fn map<F, FO>(&mut self, f: F) -> Alive<'lf, W, FO>
    where
        F: FnOnce(Arc<TS::OutputMobject>) -> FO,
        FO: TimelineState,
    {
        self.layer_field.start(f(self.end()))
    }
}

impl<W, TS> Drop for Alive<'_, W, TS>
where
    W: World,
    TS: TimelineState,
{
    fn drop(&mut self) {
        if self.timeline_state.is_some() {
            self.end();
        }
    }
}

pub trait TimelineState {
    type MobjectPresentation: 'static;
    type OutputMobject;

    fn archive<W>(
        &mut self,
        time_interval: Range<Time>,
        layer_field: &LayerField<W, Self::MobjectPresentation>,
    ) -> Arc<Self::OutputMobject>
    where
        W: World;
}

pub struct CollapsedTimelineState<M> {
    mobject: Arc<M>,
}

impl<M> TimelineState for CollapsedTimelineState<M>
where
    M: Mobject,
{
    type MobjectPresentation = M::MobjectPresentation;
    type OutputMobject = M;

    fn archive<W>(
        &mut self,
        time_interval: Range<Time>,
        layer_field: &LayerField<W, Self::MobjectPresentation>,
    ) -> Arc<Self::OutputMobject>
    where
        W: World,
    {
        layer_field.push(
            time_interval,
            StaticTimeline {
                mobject: self.mobject.clone(),
                phantom: PhantomData::<W::SerializableKeyFn>,
            },
        );
        self.mobject.clone()
    }
}

pub struct IndeterminedTimelineState<M, TE> {
    mobject: Arc<M>,
    time_eval: Arc<TE>,
}

impl<M, TE> TimelineState for IndeterminedTimelineState<M, TE>
where
    M: Mobject,
    TE: TimeEval,
{
    type MobjectPresentation = M::MobjectPresentation;
    type OutputMobject = M;

    fn archive<W>(
        &mut self,
        time_interval: Range<Time>,
        layer_field: &LayerField<W, Self::MobjectPresentation>,
    ) -> Arc<Self::OutputMobject>
    where
        W: World,
    {
        layer_field.push(
            time_interval,
            StaticTimeline {
                mobject: self.mobject.clone(),
                phantom: PhantomData::<W::SerializableKeyFn>,
            },
        );
        self.mobject.clone()
    }
}

pub struct UpdateTimelineState<M, TE, U> {
    mobject: Arc<M>,
    time_eval: Arc<TE>,
    update: Arc<U>,
}

impl<M, TE, U> TimelineState for UpdateTimelineState<M, TE, U>
where
    M: Mobject,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, M>,
{
    type MobjectPresentation = M::MobjectPresentation;
    type OutputMobject = M;

    fn archive<W>(
        &mut self,
        time_interval: Range<Time>,
        layer_field: &LayerField<W, Self::MobjectPresentation>,
    ) -> Arc<Self::OutputMobject>
    where
        W: World,
    {
        layer_field.push(
            time_interval,
            DynamicTimeline {
                mobject: self.mobject.clone(),
                time_eval: self.time_eval.clone(),
                update: self.update.clone(),
                phantom: PhantomData::<W::SerializableKeyFn>,
            },
        );
        self.mobject.clone()
    }

    // type OutputTimelineState = CollapsedTimelineState<M>;

    // fn into_next(
    //     self,
    //     supervisor: &Supervisor,
    //     time_interval: Range<Time>,
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

pub struct ConstructTimelineState<M, TE, C> {
    mobject: Arc<M>,
    time_eval: Arc<TE>,
    // root_archive: (Range<Time>, Vec<RenderableEntry>),
    // time_eval: Arc<TE>,
    construct: Option<C>,
}

impl<M, TE, C> TimelineState for ConstructTimelineState<M, TE, C>
where
    M: Mobject,
    TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    C: Construct<M>,
{
    type MobjectPresentation = M::MobjectPresentation;
    type OutputMobject = C::OutputMobject;

    fn archive<W>(
        &mut self,
        time_interval: Range<Time>,
        layer_field: &LayerField<W, Self::MobjectPresentation>,
    ) -> Arc<Self::OutputMobject>
    where
        W: World,
    {
        let world = layer_field.world();
        let world = world.as_ref();
        world.grow_stack();
        let output_mobject = self
            .construct
            .take()
            .unwrap()
            .construct(
                world,
                layer_field.start(CollapsedTimelineState {
                    mobject: self.mobject.clone(),
                }),
            )
            .end()
            .mobject;
        world.shrink_stack(time_interval, self.time_eval.as_ref());
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
//     timeline_slots: RefCell<Vec<Result<Vec<Box<dyn AllocatedTimeline>>, Arc<dyn Any>>>>,
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

pub trait ApplyConstruct<M>: Sized
where
    M: Mobject,
{
    type Output<C>
    where
        C: Construct<M>;

    fn apply_construct<C>(self, construct: C) -> Self::Output<C>
    where
        C: Construct<M>;
}

impl<'lf, W, M> Quantize for Alive<'lf, W, CollapsedTimelineState<M>>
where
    W: World,
    M: Mobject,
{
    type Output<TE> =
        Alive<'lf, W, IndeterminedTimelineState<M, TE>>
    where
        TE: TimeEval;

    #[must_use]
    fn quantize<TE>(mut self, time_eval: TE) -> Self::Output<TE>
    where
        TE: TimeEval,
    {
        self.map(|timeline_state| IndeterminedTimelineState {
            mobject: timeline_state.mobject.clone(),
            time_eval: Arc::new(time_eval),
        })
    }
}

impl<'lf, W, M, TE, U> Collapse for Alive<'lf, W, UpdateTimelineState<M, TE, U>>
where
    W: World,
    M: Mobject,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, M>,
{
    type Output = Alive<'lf, W, CollapsedTimelineState<M>>;

    #[must_use]
    fn collapse(mut self) -> Self::Output {
        self.map(|timeline_state| CollapsedTimelineState {
            mobject: timeline_state.mobject.clone(),
        })
    }
}

impl<'lf, W, M, TE, C> Collapse for Alive<'lf, W, ConstructTimelineState<M, TE, C>>
where
    W: World,
    M: Mobject,
    TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    C: Construct<M>,
{
    type Output = Alive<'lf, W, CollapsedTimelineState<M>>;

    #[must_use]
    fn collapse(mut self) -> Self::Output {
        self.map(|timeline_state| CollapsedTimelineState {
            mobject: timeline_state.mobject.clone(),
        })
    }
}

impl<'lf, W, M, TE> ApplyRate<TE::OutputTimeMetric>
    for Alive<'lf, W, IndeterminedTimelineState<M, TE>>
where
    W: World,
    M: Mobject,
    TE: TimeEval,
{
    type Output<RA> =
        Alive<'lf, W, IndeterminedTimelineState<M, RateComposeTimeEval<RA, TE>>>
    where
        RA: Rate<TE::OutputTimeMetric>;

    #[must_use]
    fn apply_rate<RA>(mut self, rate: RA) -> Self::Output<RA>
    where
        RA: Rate<TE::OutputTimeMetric>,
    {
        self.map(|timeline_state| IndeterminedTimelineState {
            mobject: timeline_state.mobject.clone(),
            time_eval: Arc::new(RateComposeTimeEval {
                rate,
                time_eval: timeline_state.time_eval.clone(),
            }),
        })
    }
}

impl<'lf, W, M, TE> ApplyUpdate<TE::OutputTimeMetric, M>
    for Alive<'lf, W, IndeterminedTimelineState<M, TE>>
where
    W: World,
    M: Mobject,
    TE: TimeEval,
{
    type Output<U> =
        Alive<'lf, W, UpdateTimelineState<M, TE, U>>
    where
        U: Update<TE::OutputTimeMetric, M>;

    #[must_use]
    fn apply_update<U>(mut self, update: U) -> Self::Output<U>
    where
        U: Update<TE::OutputTimeMetric, M>,
    {
        self.map(|timeline_state| UpdateTimelineState {
            mobject: timeline_state.mobject,
            time_eval: timeline_state.time_eval,
            update: Arc::new(update),
        })
    }
}

impl<'lf, W, M> ApplyUpdate<NormalizedTimeMetric, M> for Alive<'lf, W, CollapsedTimelineState<M>>
where
    W: World,
    M: Mobject,
{
    type Output<U> =
        Alive<'lf, W, CollapsedTimelineState<M>>
    where
        U: Update<NormalizedTimeMetric, M>;

    #[must_use]
    fn apply_update<U>(mut self, update: U) -> Self::Output<U>
    where
        U: Update<NormalizedTimeMetric, M>,
    {
        self.map(|timeline_state| {
            let mut mobject = Arc::unwrap_or_clone(timeline_state.mobject);
            update.update(NormalizedTimeMetric(1.0), &mut mobject);
            CollapsedTimelineState {
                mobject: Arc::new(mobject),
            }
        })
    }
}

impl<'lf, W, M, TE> ApplyConstruct<L, M> for Alive<'lf, W, IndeterminedTimelineState<M, TE>>
where
    W: World,
    M: Mobject,
    TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
{
    type Output<C> =
        Alive<'lf, W, ConstructTimelineState<C::OutputMobject, TE>>
    where
        C: Construct<L, M>;

    #[must_use]
    fn apply_construct<C>(mut self, construct: C) -> Self::Output<C>
    where
        C: Construct<L, M>,
    {
        self.map(|alive_context, timeline_state| {
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
