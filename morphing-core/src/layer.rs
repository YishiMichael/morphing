use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::Range;
use std::sync::Arc;

use crate::timeline::Timeline;

use super::alive::Alive;
use super::alive::AliveRoot;
use super::alive::ArchiveState;
use super::alive::IntoArchiveState;
use super::config::Config;
use super::storage::Allocated;
use super::storage::PresentationStorage;
use super::storage::ReadWriteStorage;
use super::storage::Storable;
use super::storage::StorablePrimitive;
use super::storage::StorageTypeMap;
use super::timeline::IncreasingTimeEval;
use super::timeline::NormalizedTimeMetric;
use super::timeline::PreallocatedTimeline;
use super::timeline::TimelineEntry;
use super::timer::Time;
use super::timer::Timer;

pub struct LayerField<'lf, MP> {
    config: &'lf Config,
    timer: &'lf Timer,
    timeline_entries:
        RefCell<Vec<TimelineEntry<Box<dyn PreallocatedTimeline<MobjectPresentation = MP>>>>>,
}

impl<MP> LayerField<'_, MP> {
    pub fn new(config: &Config, timer: &Timer) -> Self {
        Self {
            config,
            timer,
            timeline_entries: RefCell::new(Vec::new()),
        }
    }

    pub fn inherit(&self, timer: &Timer) -> Self {
        Self::new(&self.config, timer)
    }

    pub fn reclaim<TE>(&self, time_interval: Range<Time>, child: Self, time_eval: &TE)
    where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    {
        self.timeline_entries.borrow_mut().extend(
            child
                .timeline_entries
                .into_inner()
                .into_iter()
                .map(|timeline_entry| TimelineEntry {
                    time_interval: time_interval.start
                        + (time_interval.end - time_interval.start)
                            * *time_eval.time_eval(
                                timeline_entry.time_interval.start,
                                0.0..child.timer.time,
                            )
                        ..time_interval.start
                            + (time_interval.end - time_interval.start)
                                * *time_eval.time_eval(
                                    timeline_entry.time_interval.end,
                                    0.0..child.timer.time,
                                ),
                    timeline: timeline_entry.timeline,
                }),
        );
    }

    pub fn collect(
        self,
    ) -> Vec<TimelineEntry<Box<dyn PreallocatedTimeline<MobjectPresentation = MP>>>> {
        self.timeline_entries.into_inner()
    }

    pub(crate) fn push_timeline<T>(&self, time_interval: Range<Time>, timeline: T)
    where
        T: Timeline<MobjectPresentation = MP>,
    {
        self.timeline_entries.borrow_mut().push(TimelineEntry {
            time_interval,
            timeline: Box::new(timeline),
        });
    }
}

pub trait Layer:
    'static + Clone + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
{
    type Preallocated: PreallocatedLayer;

    fn new(config: &Config, timer: &Timer) -> Self;
    fn inherit(&self, timer: &Timer) -> Self;
    fn reclaim<TE>(&self, time_interval: Range<Time>, child: Self, time_eval: &TE)
    where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>;
    fn collect(self) -> Self::Preallocated;
}

pub trait PreallocatedLayer {
    fn allocate(
        self: Box<Self>,
        storage_type_map: &mut StorageTypeMap,
    ) -> Box<dyn AllocatedLayer>;
}

pub trait AllocatedLayer {
    fn prepare(
        &self,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    );
    fn render()
}

trait Renderable:
    'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize + StorablePrimitive
{
    fn prepare(
        &self,
        time: Time,
        prepare_ref: &mut Option<<Self::PresentationStorage as PresentationStorage>::Target>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    );
    fn render(
        &self,
        render_ref: &Option<<Self::PresentationStorage as PresentationStorage>::Target>,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    );
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct LayerRenderable<L> {
    layer: Arc<L>,
}

impl<L> StorablePrimitive for LayerRenderable<L>
where
    L: Layer,
{
    type PresentationStorage = ReadWriteStorage<L::LayerPresentation>;

    fn storage_id_input(
        &self,
    ) -> <Self::PresentationStorage as PresentationStorage>::StorageIdInput {
        ()
    }
}

impl<L> Renderable for LayerRenderable<L>
where
    L: Layer,
{
    fn prepare(
        &self,
        time: Time,
        prepare_ref: &mut Option<<Self::PresentationStorage as PresentationStorage>::Target>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        let _ = prepare_ref.insert(self.layer.prepare(time, device, queue, format));
    }

    fn render(
        &self,
        render_ref: &Option<<Self::PresentationStorage as PresentationStorage>::Target>,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        self.layer
            .render(render_ref.as_ref().unwrap(), encoder, target);
    }
}

pub trait RenderableErased {
    fn allocate(self, storage_type_map: &mut StorageTypeMap) -> Box<dyn AllocatedRenderableErased>;
}

impl<R> RenderableErased for R
where
    R: Renderable,
{
    fn allocate(self, storage_type_map: &mut StorageTypeMap) -> Box<dyn AllocatedRenderableErased> {
        Box::new(storage_type_map.allocate(self))
    }
}

pub trait AllocatedRenderableErased {
    fn prepare(
        &self,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    );
    fn render(
        &self,
        storage_type_map: &StorageTypeMap,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    );
}

impl<R> AllocatedRenderableErased for Allocated<R>
where
    R: Renderable,
{
    fn prepare(
        &self,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        self.storable_primitive().prepare(
            time,
            storage_type_map.get_mut(self),
            device,
            queue,
            format,
        );
    }

    fn render(
        &self,
        storage_type_map: &StorageTypeMap,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        self.storable_primitive()
            .render(storage_type_map.get_ref(self), encoder, target);
    }
}

pub struct RenderableEntry {
    time_interval: Range<Time>,
    renderable: Box<dyn RenderableErased>,
    timeline_entries: Vec<TimelineEntry>,
}

pub struct LayerRenderableState<L> {
    pub(crate) layer: Arc<L>,
}

impl<L> ArchiveState for LayerRenderableState<L>
where
    L: Layer,
{
    type LocalArchive = RefCell<(Vec<TimelineEntry>, Vec<RenderableEntry>)>;
    type GlobalArchive = RefCell<Vec<RenderableEntry>>;

    fn archive(
        &mut self,
        time_interval: Range<Time>,
        local_archive: Self::LocalArchive,
        global_archive: &Self::GlobalArchive,
    ) {
        let (timeline_entries, renderable_entries) = local_archive.into_inner();
        let mut global_archive_mut = global_archive.borrow_mut();
        global_archive_mut.push(RenderableEntry {
            time_interval,
            renderable: Box::new(LayerRenderable {
                layer: self.layer.clone(),
            }),
            timeline_entries,
        });
        global_archive_mut.extend(renderable_entries);
        // (
        //     time_interval,
        //     Box::new(LayerRenderable {
        //         layer: self.layer.clone(),
        //     }),
        // )
    }
}

impl<LB> IntoArchiveState<AliveRoot<'_>> for LB
where
    LB: LayerBuilder,
{
    type ArchiveState = LayerRenderableState<LB::Instantiation>;

    fn into_archive_state(self, alive_context: &AliveRoot) -> Self::ArchiveState {
        LayerRenderableState {
            layer: Arc::new(self.instantiate(alive_context.config())),
        }
    }
}

pub type AliveRenderable<'a1, 'a0, RAS> = Alive<'a1, AliveRoot<'a0>, RAS>;

// impl AliveRecorder<'_, '_, RenderableStateArchive> {
// fn new_static_timeline_entry<M>(
//     &self,
//     time_interval: Range<Time>,
//     mobject: Arc<M>,
// ) -> TimelineEntry
// where
//     M: Mobject,
// {
//     TimelineEntry {
//         time_interval,
//         timeline: serde_traitobject::Box::new(StaticTimeline {
//             id: self.storage.static_allocate(&mobject),
//             mobject,
//         }),
//     }
// }

// fn new_dynamic_timeline_entry<M, TE, U>(
//     &self,
//     time_interval: Range<Time>,
//     mobject: Arc<M>,
//     time_eval: Arc<TE>,
//     update: Arc<U>,
// ) -> TimelineEntry
// where
//     M: Mobject,
//     TE: TimeEval,
//     U: Update<TE::OutputTimeMetric, M>,
// {
//     TimelineEntry {
//         time_interval,
//         timeline: serde_traitobject::Box::new(DynamicTimeline {
//             id: self.storage.dynamic_allocate(&mobject, &update),
//             mobject,
//             time_eval,
//             update,
//         }),
//     }
// }

// fn iter_timeline_entries(self) -> impl Iterator<Item = Box<dyn AllocatedTimelineErased>> {
//     self.timeline_slots
//         .into_inner()
//         .into_iter()
//         .flat_map(|slot| slot.unwrap())
// }

// fn arc_time(&self) -> Arc<Time> {
//     self.time.borrow().clone()
// }

// fn push<T>(&self, time_interval: Range<f32>, timeline: Arc<T>)
// where
//     T: 'static + Timeline,
// {
//     // Hash `Arc<T>` instead of `T`.
//     // Presentation maps inside `storage` are identified only by `T::Presentation` type, without `T`.
//     let timeline = serde_traitobject::Arc::from(timeline as Arc<dyn Timeline>);
//     let hash = seahash::hash(&ron::ser::to_string(&timeline).unwrap().into_bytes());
//     self.timeline_entries.borrow_mut().push(TimelineEntry {
//         hash,
//         time_interval,
//         timeline,
//     });
// }

// fn start<'sv, TS>(&'sv self, alive_content: AliveContent<TS>) -> Alive<'am, TC, TS>
// where
//     TS: TimelineState,
// {
//     let alive_content = Arc::new(alive_content);
//     // let weak_timeline_state = Arc::downgrade(&timeline_state);
//     let weak = Arc::downgrade(&alive_content);
//     self.timeline_slots.borrow_mut().push(Err(alive_content));
//     Alive {
//         supervisor: self,
//         weak,
//     }
// }

// fn end<'sv, TS>(&'sv self, alive: &Alive<'am, TC, TS>) -> AliveContent<TS::OutputTimelineState>
// where
//     TS: TimelineState,
// {
//     let alive_content = alive.weak.upgrade().unwrap();
//     let mut timeline_slots_ref = self.timeline_slots.borrow_mut();
//     let slot = timeline_slots_ref
//         .iter_mut()
//         .rfind(|slot| {
//             slot.as_ref().is_err_and(|alive_content_ref| {
//                 Arc::ptr_eq(alive_content_ref, &(alive_content.clone() as Arc<dyn Any>))
//             })
//         })
//         .unwrap();
//     *slot = Ok(Vec::new());

//     let AliveContent {
//         spawn_time,
//         timeline_state,
//     } = match Arc::try_unwrap(alive_content) {
//         Ok(alive_content) => alive_content,
//         Err(_) => unreachable!(),
//     };
//     let archive_time = self.time.borrow().clone();
//     let timeline_entries_sink = TimelineEntriesSink(
//         (!Arc::ptr_eq(&spawn_time, &archive_time)).then(|| slot.as_mut().unwrap()),
//     );
//     let output_timeline_state =
//         timeline_state.into_next(self, *spawn_time..*archive_time, timeline_entries_sink);
//     AliveContent {
//         spawn_time: archive_time,
//         timeline_state: output_timeline_state,
//     }

//     // let (any_timeline_state, timeline_entries) =
//     //     &mut supervisor.timeline_slots.borrow_mut()[self.index];

//     // assert!(any_timeline_state.take().is_some());
//     // let archive_time = supervisor.arc_time();
//     // let spawn_time = std::mem::replace(&mut self.spawn_time, archive_time.clone());
//     // let timeline_entries_sink = TimelineEntriesSink(
//     //     (!Arc::ptr_eq(&spawn_time, &archive_time)).then_some(timeline_entries),
//     // );
//     // let output_timeline_state = timeline_state.into_next(
//     //     *spawn_time..*archive_time,
//     //     supervisor,
//     //     timeline_entries_sink,
//     // );
// }

// #[must_use]
// pub fn spawn_layer<LB>(
//     &self,
//     layer_builder: LB,
// ) -> Alive<'_, '_, Supervisor<'_>, LayerRenderableState<LB::Instantiation>>
// where
//     LB: LayerBuilder,
//     // 'sv: 'c,
// {
//     self.start(LayerRenderableState {
//         layer: Arc::new(layer_builder.instantiate(self)),
//     })
// }

// #[must_use]
// pub fn spawn_layer<LB>(
//     &self,
//     layer_builder: LB,
// ) -> Alive<LayerRenderableState<LB::Instantiation>>
// where
//     LB: LayerBuilder,
// {
//     self.start(LayerRenderableState {
//         layer: Arc::new(layer_builder.instantiate(self.config())),
//     })
// }
// }

// impl Alive<'_, AliveContextRoot<'_>, >
