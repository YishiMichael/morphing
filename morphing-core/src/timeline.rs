use std::any::Any;
use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::Range;
use std::sync::Arc;
use std::sync::Weak;

use super::config::Config;
use super::traits::Construct;
use super::traits::IncreasingRate;
use super::traits::Mobject;
use super::traits::MobjectBuilder;
use super::traits::Rate;
use super::traits::Storage;
use super::traits::Update;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct StaticTimelineId(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DynamicTimelineId(pub u64);

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

type Time = f32;

pub(crate) trait TimeMetric {}

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
            (time_interval.end - time_interval.start != 0.0)
                .then(|| (time - time_interval.start) / (time_interval.end - time_interval.start))
                .unwrap_or_default(),
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
pub struct RateComposeTimeEval<R, TE> {
    rate: R,
    time_eval: Arc<TE>,
}

impl<R, TE> TimeEval for RateComposeTimeEval<R, TE>
where
    R: Rate<TE::OutputTimeMetric>,
    TE: TimeEval,
{
    type OutputTimeMetric = R::OutputTimeMetric;

    fn time_eval(&self, time: Time, time_interval: Range<Time>) -> Self::OutputTimeMetric {
        self.rate
            .eval(self.time_eval.time_eval(time, time_interval))
    }
}

impl<R, TE> IncreasingTimeEval for RateComposeTimeEval<R, TE>
where
    R: IncreasingRate<TE::OutputTimeMetric>,
    TE: IncreasingTimeEval,
{
}

trait Timeline<S>:
    'static + Debug + Send + Sync + serde_traitobject::Deserialize + serde_traitobject::Serialize
where
    S: Storage,
{
    fn prepare(
        &self,
        time: Time,
        time_interval: Range<Time>,
        storage: &mut S,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        activate: Option<()>,
    );
    fn render(&self, storage: &S, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView);
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct StaticTimeline<M> {
    id: StaticTimelineId,
    mobject: Arc<M>,
}

impl<S, M> Timeline<S> for StaticTimeline<M>
where
    S: Storage,
    M: Mobject,
{
    fn prepare(
        &self,
        _time: Time,
        _time_interval: Range<Time>,
        storage: &mut S,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        activate: Option<()>,
    ) {
        storage
            .static_set(&self.id, activate)
            .map(|option_mobject_presentation| {
                option_mobject_presentation
                    .get_or_insert_with(|| Arc::new(self.mobject.presentation(device)));
            });
    }

    fn render(&self, storage: &S, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        storage.static_get(&self.id).map(|mobject_presentation| {
            mobject_presentation.render(encoder, target);
        });
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct DynamicTimeline<M, TE, U> {
    id: DynamicTimelineId,
    mobject: Arc<M>,
    time_eval: Arc<TE>,
    update: Arc<U>,
}

impl<S, M, TE, U> Timeline<S> for DynamicTimeline<M, TE, U>
where
    S: Storage,
    M: Mobject,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, M>,
{
    fn prepare(
        &self,
        time: Time,
        time_interval: Range<Time>,
        storage: &mut S,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        activate: Option<()>,
    ) {
        storage
            .dynamic_set(&self.id, activate)
            .map(|option_mobject_presentation| {
                self.update.update_presentation(
                    self.time_eval.time_eval(time, time_interval),
                    &self.mobject,
                    (option_mobject_presentation
                        .get_or_insert_with(|| Box::new(self.mobject.presentation(device)))
                        .as_mut() as &mut dyn Any)
                        .downcast_mut::<M::MobjectPresentation>()
                        .unwrap(),
                    device,
                    queue,
                    format,
                );
            });
    }

    fn render(&self, storage: &S, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        storage.dynamic_get(&self.id).map(|mobject_presentation| {
            mobject_presentation.render(encoder, target);
        });
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct TimelineEntry<S>
where
    S: Storage,
{
    time_interval: Range<Time>,
    timeline: serde_traitobject::Box<dyn Timeline<S>>,
}

struct TimelineEntriesSink<'v, S>(Option<&'v mut Vec<TimelineEntry<S>>>)
where
    S: Storage;

impl<S> Extend<TimelineEntry<S>> for TimelineEntriesSink<'_, S>
where
    S: Storage,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = TimelineEntry<S>>,
    {
        if let Some(timeline_entries) = self.0.as_mut() {
            timeline_entries.extend(iter)
        }
    }
}

trait TimelineState<S>: 'static
where
    S: Storage,
{
    type OutputTimelineState: TimelineState<S>;

    fn allocate_timeline_entries(
        self,
        time_interval: Range<Time>,
        supervisor: &Supervisor<S>,
        timeline_entries_sink: TimelineEntriesSink<S>,
    ) -> Self::OutputTimelineState;
}

pub struct CollapsedTimelineState<M> {
    mobject: Arc<M>,
}

pub struct IndeterminedTimelineState<M, TE> {
    mobject: Arc<M>,
    time_eval: Arc<TE>,
}

pub struct UpdateTimelineState<M, TE, U> {
    mobject: Arc<M>,
    time_eval: Arc<TE>,
    update: Arc<U>,
}

pub struct ConstructTimelineState<M, TE, C> {
    mobject: Arc<M>,
    time_eval: Arc<TE>,
    construct: C,
}

impl<S, M> TimelineState<S> for CollapsedTimelineState<M>
where
    S: Storage,
    M: Mobject,
{
    type OutputTimelineState = CollapsedTimelineState<M>;

    fn allocate_timeline_entries(
        self,
        time_interval: Range<Time>,
        supervisor: &Supervisor<S>,
        mut timeline_entries_sink: TimelineEntriesSink<S>,
    ) -> Self::OutputTimelineState {
        timeline_entries_sink
            .extend_one(supervisor.new_static_timeline_entry(time_interval, self.mobject.clone()));
        self
    }
}

impl<S, M, TE> TimelineState<S> for IndeterminedTimelineState<M, TE>
where
    S: Storage,
    M: Mobject,
    TE: TimeEval,
{
    type OutputTimelineState = IndeterminedTimelineState<M, TE>;

    fn allocate_timeline_entries(
        self,
        time_interval: Range<Time>,
        supervisor: &Supervisor<S>,
        mut timeline_entries_sink: TimelineEntriesSink<S>,
    ) -> Self::OutputTimelineState {
        timeline_entries_sink
            .extend_one(supervisor.new_static_timeline_entry(time_interval, self.mobject.clone()));
        self
    }
}

impl<S, M, TE, U> TimelineState<S> for UpdateTimelineState<M, TE, U>
where
    S: Storage,
    M: Mobject,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, M>,
{
    type OutputTimelineState = CollapsedTimelineState<M>;

    fn allocate_timeline_entries(
        self,
        time_interval: Range<Time>,
        supervisor: &Supervisor<S>,
        mut timeline_entries_sink: TimelineEntriesSink<S>,
    ) -> Self::OutputTimelineState {
        timeline_entries_sink.extend_one(supervisor.new_dynamic_timeline_entry(
            time_interval.clone(),
            self.mobject.clone(),
            self.time_eval.clone(),
            self.update.clone(),
        ));
        let mut mobject = Arc::unwrap_or_clone(self.mobject);
        self.update.update(
            self.time_eval.time_eval(time_interval.end, time_interval),
            &mut mobject,
        );
        CollapsedTimelineState {
            mobject: Arc::new(mobject),
        }
    }
}

impl<S, M, TE, C> TimelineState<S> for ConstructTimelineState<M, TE, C>
where
    S: Storage,
    M: Mobject,
    TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    C: Construct<M>,
{
    type OutputTimelineState = CollapsedTimelineState<C::OutputMobject>;

    fn allocate_timeline_entries(
        self,
        time_interval: Range<Time>,
        supervisor: &Supervisor<S>,
        mut timeline_entries_sink: TimelineEntriesSink<S>,
    ) -> Self::OutputTimelineState {
        let child_supervisor = Supervisor::new(supervisor.config, supervisor.storage);
        let timeline_state = self
            .construct
            .construct(
                Alive::new(
                    &child_supervisor,
                    child_supervisor.arc_time(),
                    CollapsedTimelineState {
                        mobject: self.mobject,
                    },
                ),
                &child_supervisor,
            )
            .archive(|_, _, output_timeline_state| output_timeline_state);
        let children_time_interval = child_supervisor.time_interval();
        timeline_entries_sink.extend(child_supervisor.iter_timeline_entries().map(
            |mut timeline_entry| {
                timeline_entry.time_interval = time_interval.start
                    + (time_interval.end - time_interval.start)
                        * *self.time_eval.time_eval(
                            timeline_entry.time_interval.start,
                            children_time_interval.clone(),
                        )
                    ..time_interval.start
                        + (time_interval.end - time_interval.start)
                            * *self.time_eval.time_eval(
                                timeline_entry.time_interval.end,
                                children_time_interval.clone(),
                            );
                timeline_entry
            },
        ));
        timeline_state
    }

    // fn allocate_timeline_entries(
    //     self,
    //     time_interval: Range<f32>,
    //     _mobject: Arc<M>,
    //     _storage: &S,
    // ) -> Vec<TimelineEntry<S>> {
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
    type Output<R>
    where
        R: Rate<TM>;

    fn apply_rate<R>(self, rate: R) -> Self::Output<R>
    where
        R: Rate<TM>;
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

pub struct Supervisor<'c, 's, S>
where
    S: Storage,
{
    config: &'c Config,
    storage: &'s S,
    time: RefCell<Arc<Time>>,
    timeline_slots: RefCell<Vec<(Option<Arc<dyn Any>>, Vec<TimelineEntry<S>>)>>,
}

impl<'c, 's, S> Supervisor<'c, 's, S>
where
    S: Storage,
{
    pub(crate) fn new(config: &'c Config, storage: &'s S) -> Self {
        Self {
            config,
            storage,
            time: RefCell::new(Arc::new(0.0)),
            timeline_slots: RefCell::new(Vec::new()),
        }
    }

    fn new_static_timeline_entry<M>(
        &self,
        time_interval: Range<Time>,
        mobject: Arc<M>,
    ) -> TimelineEntry<S>
    where
        M: Mobject,
    {
        TimelineEntry {
            time_interval,
            timeline: serde_traitobject::Box::new(StaticTimeline {
                id: self.storage.static_allocate(&mobject),
                mobject,
            }),
        }
    }

    fn new_dynamic_timeline_entry<M, TE, U>(
        &self,
        time_interval: Range<Time>,
        mobject: Arc<M>,
        time_eval: Arc<TE>,
        update: Arc<U>,
    ) -> TimelineEntry<S>
    where
        M: Mobject,
        TE: TimeEval,
        U: Update<TE::OutputTimeMetric, M>,
    {
        TimelineEntry {
            time_interval,
            timeline: serde_traitobject::Box::new(DynamicTimeline {
                id: self.storage.dynamic_allocate(&mobject, &update),
                mobject,
                time_eval,
                update,
            }),
        }
    }

    fn iter_timeline_entries(self) -> impl Iterator<Item = TimelineEntry<S>> {
        self.timeline_slots
            .into_inner()
            .into_iter()
            .flat_map(|(timeline, timeline_entries)| {
                assert!(timeline.is_none());
                timeline_entries
            })
    }

    fn arc_time(&self) -> Arc<Time> {
        self.time.borrow().clone()
    }

    pub fn time_interval(&self) -> Range<Time> {
        0.0..*self.arc_time()
    }

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

    #[must_use]
    pub fn spawn<'sv, MB>(
        &'sv self,
        mobject_builder: MB,
    ) -> Alive<'c, 's, 'sv, S, CollapsedTimelineState<MB::Instantiation>>
    where
        MB: MobjectBuilder,
        'sv: 'c,
    {
        Alive::new(
            self,
            self.arc_time(),
            CollapsedTimelineState {
                mobject: Arc::new(mobject_builder.instantiate(&self.config)),
            },
        )
    }

    pub fn wait(&self, delta_time: Time) {
        assert!(
            delta_time.is_sign_positive(),
            "`Supervisor::wait` expects a non-negative `delta_time`, got {delta_time}",
        );
        let mut time = self.time.borrow_mut();
        *time = Arc::new(**time + delta_time);
    }
}

pub struct Alive<'c, 's, 'sv, S, TS>
where
    S: Storage,
    TS: TimelineState<S>,
{
    supervisor: &'sv Supervisor<'c, 's, S>,
    spawn_time: Arc<Time>,
    weak_timeline_state: Weak<TS>,
    index: usize,
}

impl<'c, 's, 'sv, S, TS> Alive<'c, 's, 'sv, S, TS>
where
    S: Storage,
    TS: TimelineState<S>,
{
    fn new(
        supervisor: &'sv Supervisor<'c, 's, S>,
        spawn_time: Arc<Time>,
        timeline_state: TS,
    ) -> Self {
        let timeline_state = Arc::new(timeline_state);
        let weak_timeline_state = Arc::downgrade(&timeline_state);
        let mut timeline_slots = supervisor.timeline_slots.borrow_mut();
        let index = timeline_slots.len();
        timeline_slots.push((Some(timeline_state), Vec::new()));
        Self {
            supervisor,
            spawn_time,
            weak_timeline_state,
            index,
        }
    }

    fn archive<F, FO>(&mut self, f: F) -> FO
    where
        F: FnOnce(&'sv Supervisor<'c, 's, S>, Arc<Time>, TS::OutputTimelineState) -> FO,
    {
        let supervisor = self.supervisor;
        let (any_timeline_state, timeline_entries) =
            &mut supervisor.timeline_slots.borrow_mut()[self.index];
        let timeline_state = match Arc::try_unwrap(self.weak_timeline_state.upgrade().unwrap()) {
            Ok(timeline_state) => timeline_state,
            Err(_) => unreachable!(),
        };
        assert!(any_timeline_state.take().is_some());
        let archive_time = supervisor.arc_time();
        let spawn_time = std::mem::replace(&mut self.spawn_time, archive_time.clone());
        let timeline_entries_sink = TimelineEntriesSink(
            (!Arc::ptr_eq(&spawn_time, &archive_time)).then_some(timeline_entries),
        );
        let output_timeline_state = timeline_state.allocate_timeline_entries(
            *spawn_time..*archive_time,
            supervisor,
            timeline_entries_sink,
        );
        f(supervisor, archive_time, output_timeline_state)
    }
}

impl<S, TS> Drop for Alive<'_, '_, '_, S, TS>
where
    S: Storage,
    TS: TimelineState<S>,
{
    fn drop(&mut self) {
        if self.weak_timeline_state.strong_count() != 0 {
            self.archive(|_, _, _| ());
        }
    }
}

impl<'c, 's, 'sv, S, M> Quantize for Alive<'c, 's, 'sv, S, CollapsedTimelineState<M>>
where
    S: Storage,
    M: Mobject,
{
    type Output<TE> = Alive<'c, 's, 'sv, S, IndeterminedTimelineState<M, TE>> where TE: TimeEval;

    #[must_use]
    fn quantize<TE>(mut self, time_eval: TE) -> Self::Output<TE>
    where
        TE: TimeEval,
    {
        self.archive(|supervisor, archive_time, output_timeline_state| {
            Alive::new(
                supervisor,
                archive_time,
                IndeterminedTimelineState {
                    mobject: output_timeline_state.mobject,
                    time_eval: Arc::new(time_eval),
                },
            )
        })
    }
}

impl<'c, 's, 'sv, S, M, TE, U> Collapse for Alive<'c, 's, 'sv, S, UpdateTimelineState<M, TE, U>>
where
    S: Storage,
    M: Mobject,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, M>,
{
    type Output = Alive<'c, 's, 'sv, S, CollapsedTimelineState<M>>;

    #[must_use]
    fn collapse(mut self) -> Self::Output {
        self.archive(|supervisor, archive_time, output_timeline_state| {
            Alive::new(supervisor, archive_time, output_timeline_state)
        })
    }
}

impl<'c, 's, 'sv, S, M, TE, C> Collapse for Alive<'c, 's, 'sv, S, ConstructTimelineState<M, TE, C>>
where
    S: Storage,
    M: Mobject,
    TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    C: Construct<M>,
{
    type Output = Alive<'c, 's, 'sv, S, CollapsedTimelineState<C::OutputMobject>>;

    #[must_use]
    fn collapse(mut self) -> Self::Output {
        self.archive(|supervisor, archive_time, output_timeline_state| {
            Alive::new(supervisor, archive_time, output_timeline_state)
        })
    }
}

impl<'c, 's, 'sv, S, M, TE> ApplyRate<TE::OutputTimeMetric>
    for Alive<'c, 's, 'sv, S, IndeterminedTimelineState<M, TE>>
where
    S: Storage,
    M: Mobject,
    TE: TimeEval,
{
    type Output<R> = Alive<'c, 's, 'sv, S, IndeterminedTimelineState<M, RateComposeTimeEval<R, TE>>>
    where
        R: Rate<TE::OutputTimeMetric>;

    #[must_use]
    fn apply_rate<R>(mut self, rate: R) -> Self::Output<R>
    where
        R: Rate<TE::OutputTimeMetric>,
    {
        self.archive(|supervisor, archive_time, output_timeline_state| {
            Alive::new(
                supervisor,
                archive_time,
                IndeterminedTimelineState {
                    mobject: output_timeline_state.mobject,
                    time_eval: Arc::new(RateComposeTimeEval {
                        rate,
                        time_eval: output_timeline_state.time_eval,
                    }),
                },
            )
        })
    }
}

impl<'c, 's, 'sv, S, M, TE> ApplyUpdate<TE::OutputTimeMetric, M>
    for Alive<'c, 's, 'sv, S, IndeterminedTimelineState<M, TE>>
where
    S: Storage,
    M: Mobject,
    TE: TimeEval,
{
    type Output<U> = Alive<'c, 's, 'sv, S, UpdateTimelineState<M, TE, U>>
    where
        U: Update<TE::OutputTimeMetric, M>;

    #[must_use]
    fn apply_update<U>(mut self, update: U) -> Self::Output<U>
    where
        U: Update<TE::OutputTimeMetric, M>,
    {
        self.archive(|supervisor, archive_time, output_timeline_state| {
            Alive::new(
                supervisor,
                archive_time,
                UpdateTimelineState {
                    mobject: output_timeline_state.mobject,
                    time_eval: output_timeline_state.time_eval,
                    update: Arc::new(update),
                },
            )
        })
    }
}

impl<'c, 's, 'sv, S, M> ApplyUpdate<NormalizedTimeMetric, M>
    for Alive<'c, 's, 'sv, S, CollapsedTimelineState<M>>
where
    S: Storage,
    M: Mobject,
{
    type Output<U> = Alive<'c, 's, 'sv, S, CollapsedTimelineState<M>> where U: Update<NormalizedTimeMetric, M>;

    #[must_use]
    fn apply_update<U>(mut self, update: U) -> Self::Output<U>
    where
        U: Update<NormalizedTimeMetric, M>,
    {
        self.archive(|supervisor, archive_time, output_timeline_state| {
            let mut mobject = Arc::unwrap_or_clone(output_timeline_state.mobject);
            update.update(NormalizedTimeMetric(1.0), &mut mobject);
            Alive::new(
                supervisor,
                archive_time,
                CollapsedTimelineState {
                    mobject: Arc::new(mobject),
                },
            )
        })
    }
}

impl<'c, 's, 'sv, S, M, TE> ApplyConstruct<M>
    for Alive<'c, 's, 'sv, S, IndeterminedTimelineState<M, TE>>
where
    S: Storage,
    M: Mobject,
    TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
{
    type Output<C> = Alive<'c, 's, 'sv, S, ConstructTimelineState<M, TE, C>>
    where
        C: Construct<M>;

    #[must_use]
    fn apply_construct<C>(mut self, construct: C) -> Self::Output<C>
    where
        C: Construct<M>,
    {
        self.archive(|supervisor, archive_time, output_timeline_state| {
            Alive::new(
                supervisor,
                archive_time,
                ConstructTimelineState {
                    mobject: output_timeline_state.mobject,
                    time_eval: output_timeline_state.time_eval,
                    construct,
                },
            )
        })
    }
}

trait QuantizeExt: Quantize {
    fn animate(self) -> Self::Output<NormalizedTimeEval>;
    fn animating(self) -> Self::Output<DenormalizedTimeEval>;
}

impl<'c, 's, 'sv, S, TS> QuantizeExt for Alive<'c, 's, 'sv, S, TS>
where
    S: Storage,
    TS: TimelineState<S>,
    Self: Quantize,
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

impl<'c, 's, 'sv, S, TS> CollapseExt for Alive<'c, 's, 'sv, S, TS>
where
    S: Storage,
    TS: TimelineState<S>,
    Self: Collapse,
{
    #[must_use]
    fn play(self, delta_time: Time) -> Self::Output {
        self.supervisor.wait(delta_time);
        self.collapse()
    }
}
