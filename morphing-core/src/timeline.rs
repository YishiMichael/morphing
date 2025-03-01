use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::Range;
use std::sync::Arc;

use super::alive::Alive;
use super::alive::AliveContext;
use super::alive::AliveRootContext;
use super::alive::ArchiveState;
use super::alive::IntoArchiveState;
use super::alive::Time;
use super::renderable::AliveRenderable;
use super::renderable::LayerRenderableState;
use super::renderable::RenderableErased;
use super::storage::Allocated;
use super::storage::MapStorage;
use super::storage::PresentationStorage;
use super::storage::ReadStorage;
use super::storage::ReadWriteStorage;
use super::storage::StorablePrimitive;
use super::storage::StorageTypeMap;
use super::storage::SwapStorage;
use super::traits::Construct;
use super::traits::IncreasingRate;
use super::traits::Layer;
use super::traits::Mobject;
use super::traits::MobjectBuilder;
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

trait Timeline:
    'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize + StorablePrimitive
// + typemap_rev::TypeMapKey<Value = HashMap<Self::SerdeKey, Self::MobjectPresentationStorage>>
{
    // type SerdeKey: 'static + Eq + Hash + Send + Sync;
    // type MobjectPresentationStorage: PresentationStorage;

    // fn serde_key(&self) -> Self::SerdeKey;
    fn init_presentation(
        &self,
        device: &wgpu::Device,
    ) -> <Self::PresentationStorage as PresentationStorage>::Target;
    fn prepare(
        &self,
        time: Time,
        time_interval: Range<Time>,
        prepare_ref: &mut <Self::PresentationStorage as PresentationStorage>::Target,
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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct StaticTimeline<M> {
    // mobject_serde_key: serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    // slot_id: SI,
    // time_interval: Range<Time>,
    mobject: Arc<M>,
}

impl<M> StorablePrimitive for StaticTimeline<M>
where
    M: Mobject,
{
    type PresentationStorage = MapStorage<
        serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
        SwapStorage<ReadStorage<M::MobjectPresentation>>,
    >;

    fn storage_id_input(
        &self,
    ) -> <Self::PresentationStorage as PresentationStorage>::StorageIdInput {
        (
            serde_hashkey::to_key_with_ordered_float(self.mobject.as_ref()).unwrap(),
            (),
        )
    }
}

impl<M> Timeline for StaticTimeline<M>
where
    M: Mobject,
{
    // type SerdeKey = serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>;
    // type MobjectPresentationStorage = ;

    // fn serde_key(&self) -> Self::SerdeKey {
    //     serde_hashkey::to_key_with_ordered_float(self.mobject.as_ref()).unwrap()
    // }

    fn init_presentation(
        &self,
        device: &wgpu::Device,
    ) -> <Self::PresentationStorage as PresentationStorage>::Target {
        Arc::new(self.mobject.presentation(device))
    }

    fn prepare(
        &self,
        _time: Time,
        _time_interval: Range<Time>,
        _prepare_ref: &mut <Self::PresentationStorage as PresentationStorage>::Target,
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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct DynamicTimeline<M, TE, U> {
    // mobject_serde_key: serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    // update_serde_key: serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
    // slot_id: SI,
    // time_interval: Range<Time>,
    mobject: Arc<M>,
    time_eval: Arc<TE>,
    update: Arc<U>,
}

impl<M, TE, U> StorablePrimitive for DynamicTimeline<M, TE, U>
where
    M: Mobject,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, M>,
{
    type PresentationStorage = MapStorage<
        (
            serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
            serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>,
        ),
        SwapStorage<ReadWriteStorage<M::MobjectPresentation>>,
    >;

    fn storage_id_input(
        &self,
    ) -> <Self::PresentationStorage as PresentationStorage>::StorageIdInput {
        (
            (
                serde_hashkey::to_key_with_ordered_float(self.mobject.as_ref()).unwrap(),
                serde_hashkey::to_key_with_ordered_float(self.update.as_ref()).unwrap(),
            ),
            (),
        )
    }
}

impl<M, TE, U> Timeline for DynamicTimeline<M, TE, U>
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

    fn init_presentation(
        &self,
        device: &wgpu::Device,
    ) -> <Self::PresentationStorage as PresentationStorage>::Target {
        self.mobject.presentation(device)
    }

    fn prepare(
        &self,
        time: Time,
        time_interval: Range<Time>,
        prepare_ref: &mut <Self::PresentationStorage as PresentationStorage>::Target,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        self.update.update_presentation(
            self.time_eval.time_eval(time, time_interval),
            &self.mobject,
            prepare_ref,
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

pub trait TimelineErased {
    fn allocate(self, storage_type_map: &mut StorageTypeMap) -> Box<dyn AllocatedTimelineErased>;
}

impl<T> TimelineErased for T
where
    T: Timeline,
{
    fn allocate(self, storage_type_map: &mut StorageTypeMap) -> Box<dyn AllocatedTimelineErased> {
        Box::new(storage_type_map.allocate(self))
    }
}

trait AllocatedTimelineErased {
    fn prepare(
        &self,
        time: Time,
        time_interval: Range<f32>,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    );
}

impl<T> AllocatedTimelineErased for Allocated<T>
where
    T: Timeline,
{
    fn prepare(
        &self,
        time: Time,
        time_interval: Range<f32>,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        self.storable_primitive().prepare(
            time,
            time_interval,
            storage_type_map
                .get_mut(self)
                .get_or_insert_with(|| self.storable_primitive().init_presentation(device)),
            device,
            queue,
            format,
        );
    }
}

// struct TimelineEntriesSink<'v>(Option<&'v mut Vec<WithTimeInterval<Box<dyn AllocatedTimeline>>>>);

// impl Extend<Box<dyn AllocatedTimelineErased>> for TimelineEntriesSink<'_> {
//     fn extend<I>(&mut self, iter: I)
//     where
//         I: IntoIterator<Item = Box<dyn AllocatedTimelineErased>>,
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

pub struct CollapsedTimelineState<M> {
    mobject: Arc<M>,
}

impl<M> ArchiveState for CollapsedTimelineState<M>
where
    M: Mobject,
{
    type LocalArchive = ();
    type GlobalArchive = RefCell<(
        Vec<(Range<Time>, Box<dyn TimelineErased>)>,
        Vec<(
            Range<Time>,
            Box<dyn RenderableErased>,
            Vec<(Range<Time>, Box<dyn TimelineErased>)>,
        )>,
    )>;

    fn archive(
        &mut self,
        time_interval: Range<Time>,
        _local_archive: Self::LocalArchive,
        global_archive: &Self::GlobalArchive,
    ) {
        global_archive.borrow_mut().0.push((
            time_interval,
            Box::new(StaticTimeline {
                mobject: self.mobject.clone(),
            }),
        ));
    }
}

pub struct IndeterminedTimelineState<M, TE> {
    mobject: Arc<M>,
    time_eval: Arc<TE>,
}

impl<M, TE> ArchiveState for IndeterminedTimelineState<M, TE>
where
    M: Mobject,
    TE: TimeEval,
{
    type LocalArchive = ();
    type GlobalArchive = RefCell<(
        Vec<(Range<Time>, Box<dyn TimelineErased>)>,
        Vec<(
            Range<Time>,
            Box<dyn RenderableErased>,
            Vec<(Range<Time>, Box<dyn TimelineErased>)>,
        )>,
    )>;

    fn archive(
        &mut self,
        time_interval: Range<Time>,
        _local_archive: Self::LocalArchive,
        global_archive: &Self::GlobalArchive,
    ) {
        global_archive.borrow_mut().0.push((
            time_interval,
            Box::new(StaticTimeline {
                mobject: self.mobject.clone(),
            }),
        ));
    }
}

pub struct UpdateTimelineState<M, TE, U> {
    mobject: Arc<M>,
    time_eval: Arc<TE>,
    update: Arc<U>,
}

impl<M, TE, U> ArchiveState for UpdateTimelineState<M, TE, U>
where
    M: Mobject,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, M>,
{
    type LocalArchive = ();
    type GlobalArchive = RefCell<(
        Vec<(Range<Time>, Box<dyn TimelineErased>)>,
        Vec<(
            Range<Time>,
            Box<dyn RenderableErased>,
            Vec<(Range<Time>, Box<dyn TimelineErased>)>,
        )>,
    )>;

    fn archive(
        &mut self,
        time_interval: Range<Time>,
        _local_archive: Self::LocalArchive,
        global_archive: &Self::GlobalArchive,
    ) {
        global_archive.borrow_mut().0.push((
            time_interval,
            Box::new(DynamicTimeline {
                mobject: self.mobject.clone(),
                time_eval: self.time_eval.clone(),
                update: self.update.clone(),
            }),
        ));
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

pub struct ConstructTimelineState<M, TE> {
    mobject: Arc<M>,
    time_eval: Arc<TE>,
    root_archive: (
        Range<Time>,
        Vec<(
            Range<Time>,
            Box<dyn RenderableErased>,
            Vec<(Range<Time>, Box<dyn TimelineErased>)>,
        )>,
    ),
    // time_eval: Arc<TE>,
    // construct: C,
}

impl<M, TE> ArchiveState for ConstructTimelineState<M, TE>
where
    M: Mobject,
    TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    // C: Construct<M>,
{
    type LocalArchive = ();
    type GlobalArchive = RefCell<(
        Vec<(Range<Time>, Box<dyn TimelineErased>)>,
        Vec<(
            Range<Time>,
            Box<dyn RenderableErased>,
            Vec<(Range<Time>, Box<dyn TimelineErased>)>,
        )>,
    )>;

    fn archive(
        &mut self,
        time_interval: Range<Time>,
        _local_archive: Self::LocalArchive,
        global_archive: &Self::GlobalArchive,
    ) {
        let (child_time_interval, renderables) = &mut self.root_archive;
        let rescale_time_interval = |entry_time_interval: &mut Range<Time>| {
            *entry_time_interval = time_interval.start
                + (time_interval.end - time_interval.start)
                    * *self
                        .time_eval
                        .time_eval(entry_time_interval.start, child_time_interval.clone())
                ..time_interval.start
                    + (time_interval.end - time_interval.start)
                        * *self
                            .time_eval
                            .time_eval(entry_time_interval.end, child_time_interval.clone())
        };
        for (renderable_time_interval, _, timelines) in renderables.as_mut_slice() {
            rescale_time_interval(renderable_time_interval);
            for (timeline_time_interval, _) in timelines.as_mut_slice() {
                rescale_time_interval(timeline_time_interval);
            }
        }
        global_archive.borrow_mut().1.extend(renderables.drain(..));
    }

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
//     L: Layer,
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
//     L: Layer,
//     TS: TimelineState,
// {
// }

// impl<'_> AliveConte

// pub struct Supervisor<'c> {
//     config: &'c Config,
//     // storage: &'s Storage,
//     time: RefCell<Arc<Time>>,
//     timeline_slots: RefCell<Vec<Result<Vec<Box<dyn AllocatedTimelineErased>>, Arc<dyn Any>>>>,
// }

// pub type TimelineStateArchive = Vec<(Range<Time>, Box<dyn TimelineErased>)>;

impl<L, MB> IntoArchiveState<AliveRenderable<'_, '_, '_, LayerRenderableState<L>>> for MB
where
    L: Layer,
    MB: MobjectBuilder<L>,
{
    type ArchiveState = CollapsedTimelineState<MB::Instantiation>;

    fn into_archive_state(
        self,
        alive_context: &AliveRenderable<'_, '_, '_, LayerRenderableState<L>>,
    ) -> Self::ArchiveState {
        CollapsedTimelineState {
            mobject: Arc::new(self.instantiate(
                &alive_context.archive_state().layer,
                alive_context.alive_context().alive_context().config(),
            )),
        }
    }
}

pub type AliveTimeline<'a3, 'a2, 'a1, 'a0, RAS, TAS> =
    Alive<'a3, AliveRenderable<'a2, 'a1, 'a0, RAS>, TAS>;

// impl<L> Alive<'_, '_, '_, LayerRenderableState<L>>
// where
//     L: Layer,
// {
//     #[must_use]
//     pub fn spawn<MB>(
//         &self,
//         mobject_builder: MB,
//     ) -> Alive<'_, '_, '_, CollapsedTimelineState<MB::Instantiation>>
//     where
//         MB: MobjectBuilder<L>,
//     {
//         self.alive_recorder().start(CollapsedTimelineState {
//             mobject: Arc::new(
//                 mobject_builder.instantiate(&self.archive().layer, self.alive_recorder().config()),
//             ),
//         })
//     }
// }

// impl<TS> Drop for Alive<'_, '_, '_, TS>
// where
//     TS: TimelineState,
// {
//     fn drop(&mut self) {
//         if self.weak.strong_count() != 0 {
//             self.supervisor.end(self);
//         }
//     }
// }

pub(crate) trait TimeMetric: 'static {}

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

trait TimeEval: 'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize {
    type OutputTimeMetric: TimeMetric;

    fn time_eval(&self, time: Time, time_interval: Range<Time>) -> Self::OutputTimeMetric;
}

trait IncreasingTimeEval: TimeEval {}

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

pub trait ApplyConstruct<L, M>: Sized
where
    L: Layer,
    M: Mobject,
{
    type Output<C>
    where
        C: Construct<L, M>;

    fn apply_construct<C>(self, construct: C) -> Self::Output<C>
    where
        C: Construct<L, M>;
}

impl<'a3, 'a2, 'a1, 'a0, L, M> Quantize
    for AliveTimeline<'a3, 'a2, 'a1, 'a0, LayerRenderableState<L>, CollapsedTimelineState<M>>
where
    L: Layer,
    M: Mobject,
{
    type Output<TE> =
        AliveTimeline<'a3, 'a2, 'a1, 'a0, LayerRenderableState<L>, IndeterminedTimelineState<M, TE>>
    where
        TE: TimeEval;

    #[must_use]
    fn quantize<TE>(mut self, time_eval: TE) -> Self::Output<TE>
    where
        TE: TimeEval,
    {
        self.map(|_, timeline_state| IndeterminedTimelineState {
            mobject: timeline_state.mobject.clone(),
            time_eval: Arc::new(time_eval),
        })
        // let timeline_state = self.end();
        // self.alive_manager().start(IndeterminedTimelineState {
        //     mobject: timeline_state.mobject.clone(),
        //     time_eval: Arc::new(time_eval),
        // })
    }
}

impl<'a3, 'a2, 'a1, 'a0, L, M, TE, U> Collapse
    for AliveTimeline<'a3, 'a2, 'a1, 'a0, LayerRenderableState<L>, UpdateTimelineState<M, TE, U>>
where
    L: Layer,
    M: Mobject,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, M>,
{
    type Output =
        AliveTimeline<'a3, 'a2, 'a1, 'a0, LayerRenderableState<L>, CollapsedTimelineState<M>>;

    #[must_use]
    fn collapse(mut self) -> Self::Output {
        self.map(|_, timeline_state| CollapsedTimelineState {
            mobject: timeline_state.mobject.clone(),
        })
    }
}

impl<'a3, 'a2, 'a1, 'a0, L, M, TE> Collapse
    for AliveTimeline<'a3, 'a2, 'a1, 'a0, LayerRenderableState<L>, ConstructTimelineState<M, TE>>
where
    L: Layer,
    M: Mobject,
    TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    // C: Construct<M>,
{
    type Output =
        AliveTimeline<'a3, 'a2, 'a1, 'a0, LayerRenderableState<L>, CollapsedTimelineState<M>>;

    #[must_use]
    fn collapse(mut self) -> Self::Output {
        self.map(|_, timeline_state| CollapsedTimelineState {
            mobject: timeline_state.mobject.clone(),
        })
    }
}

impl<'a3, 'a2, 'a1, 'a0, L, M, TE> ApplyRate<TE::OutputTimeMetric>
    for AliveTimeline<'a3, 'a2, 'a1, 'a0, LayerRenderableState<L>, IndeterminedTimelineState<M, TE>>
where
    L: Layer,
    M: Mobject,
    TE: TimeEval,
{
    type Output<RA> =
        AliveTimeline<'a3, 'a2, 'a1, 'a0, LayerRenderableState<L>, IndeterminedTimelineState<M, RateComposeTimeEval<RA, TE>>>
    where
        RA: Rate<TE::OutputTimeMetric>;

    #[must_use]
    fn apply_rate<RA>(mut self, rate: RA) -> Self::Output<RA>
    where
        RA: Rate<TE::OutputTimeMetric>,
    {
        self.map(|_, timeline_state| IndeterminedTimelineState {
            mobject: timeline_state.mobject.clone(),
            time_eval: Arc::new(RateComposeTimeEval {
                rate,
                time_eval: timeline_state.time_eval.clone(),
            }),
        })
    }
}

impl<'a3, 'a2, 'a1, 'a0, L, M, TE> ApplyUpdate<TE::OutputTimeMetric, M>
    for AliveTimeline<'a3, 'a2, 'a1, 'a0, LayerRenderableState<L>, IndeterminedTimelineState<M, TE>>
where
    L: Layer,
    M: Mobject,
    TE: TimeEval,
{
    type Output<U> =
        AliveTimeline<'a3, 'a2, 'a1, 'a0, LayerRenderableState<L>, UpdateTimelineState<M, TE, U>>
    where
        U: Update<TE::OutputTimeMetric, M>;

    #[must_use]
    fn apply_update<U>(mut self, update: U) -> Self::Output<U>
    where
        U: Update<TE::OutputTimeMetric, M>,
    {
        self.map(|_, timeline_state| UpdateTimelineState {
            mobject: timeline_state.mobject,
            time_eval: timeline_state.time_eval,
            update: Arc::new(update),
        })
    }
}

impl<'a3, 'a2, 'a1, 'a0, L, M> ApplyUpdate<NormalizedTimeMetric, M>
    for AliveTimeline<'a3, 'a2, 'a1, 'a0, LayerRenderableState<L>, CollapsedTimelineState<M>>
where
    L: Layer,
    M: Mobject,
{
    type Output<U> =
        AliveTimeline<'a3, 'a2, 'a1, 'a0, LayerRenderableState<L>, CollapsedTimelineState<M>>
    where
        U: Update<NormalizedTimeMetric, M>;

    #[must_use]
    fn apply_update<U>(mut self, update: U) -> Self::Output<U>
    where
        U: Update<NormalizedTimeMetric, M>,
    {
        self.map(|_, timeline_state| {
            let mut mobject = Arc::unwrap_or_clone(timeline_state.mobject);
            update.update(NormalizedTimeMetric(1.0), &mut mobject);
            CollapsedTimelineState {
                mobject: Arc::new(mobject),
            }
        })
    }
}

impl<'a3, 'a2, 'a1, 'a0, L, M, TE> ApplyConstruct<L, M>
    for AliveTimeline<'a3, 'a2, 'a1, 'a0, LayerRenderableState<L>, IndeterminedTimelineState<M, TE>>
where
    L: Layer,
    M: Mobject,
    TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
{
    type Output<C> =
        AliveTimeline<'a3, 'a2, 'a1, 'a0, LayerRenderableState<L>, ConstructTimelineState<C::OutputMobject, TE>>
    where
        C: Construct<L, M>;

    #[must_use]
    fn apply_construct<C>(mut self, construct: C) -> Self::Output<C>
    where
        C: Construct<L, M>,
    {
        self.map(|alive_context, timeline_state| {
            let child_root_context =
                AliveRootContext::new(alive_context.alive_context().alive_context().config());
            let child_root = child_root_context.start(());
            let child_renderable = child_root.start(LayerRenderableState {
                layer: alive_context.archive_state().layer.clone(),
            });
            let child_timeline = child_renderable.start(CollapsedTimelineState {
                mobject: timeline_state.mobject.clone(),
            });
            let child_output_timeline =
                construct.construct(&child_root, &child_renderable, child_timeline);
            ConstructTimelineState {
                mobject: child_output_timeline.archive_state().mobject.clone(),
                time_eval: timeline_state.time_eval.clone(),
                root_archive: child_root_context.into_archive(),
            }
        })
        // let timeline_state = self.end();
        // let child_supervisor = Supervisor::new(supervisor.config);
        // let output_timeline_state = child_supervisor
        //     .end(&construct.construct(
        //         child_supervisor.start(AliveContent {
        //             spawn_time: child_supervisor.time.borrow().clone(),
        //             timeline_state: CollapsedTimelineState {
        //                 mobject: self.mobject,
        //             },
        //         }),
        //         &child_supervisor,
        //     ))
        //     .timeline_state;
        // let children_time_interval = child_supervisor.time_interval();
        // timeline_entries_sink.extend(child_supervisor.iter_timeline_entries().map(
        //     |mut timeline_entry| {
        //         timeline_entry.time_interval = time_interval.start
        //             + (time_interval.end - time_interval.start)
        //                 * *self.time_eval.time_eval(
        //                     timeline_entry.time_interval.start,
        //                     children_time_interval.clone(),
        //                 )
        //             ..time_interval.start
        //                 + (time_interval.end - time_interval.start)
        //                     * *self.time_eval.time_eval(
        //                         timeline_entry.time_interval.end,
        //                         children_time_interval.clone(),
        //                     );
        //         timeline_entry
        //     },
        // ));
        // output_timeline_state
        // self.supervisor
        //     .start(
        //         self.supervisor
        //             .end(&self)
        //             .map(|output_timeline_state| ConstructTimelineState {
        //                 mobject: output_timeline_state.mobject,
        //                 time_eval: output_timeline_state.time_eval,
        //                 construct,
        //             }),
        //     )
    }
}

trait QuantizeExt: Quantize {
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

trait CollapseExt: Collapse {
    fn play(self, delta_time: Time) -> Self::Output;
}

impl<TS> CollapseExt for TS
where
    TS: Collapse,
{
    #[must_use]
    fn play(self, delta_time: Time) -> Self::Output {
        self.supervisor.wait(delta_time);
        self.collapse()
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
