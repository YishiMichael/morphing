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
use super::traits::TimeMetric;
use super::traits::Update;

// #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
// pub struct TimelineId(u64);

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

// trait Prepare<TM, M>:
//     'static + Send + Sync + Debug + serde::de::DeserializeOwned + serde::Serialize
// where
//     TM: TimeMetric,
//     M: Mobject,
// {
//     fn prepare(
//         &self,
//         time_metric: TM,
//         mobject: &M,
//         mobject_presentation: &mut M::MobjectPresentation,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     );
// }

trait Timeline<S>:
    'static + Send + Sync + Debug + serde_traitobject::Deserialize + serde_traitobject::Serialize
where
    S: Storage,
{
    fn allocate(&self, storage: &S) -> S::Key;
    fn prepare(
        &self,
        time: f32,
        time_interval: Range<f32>,
        storage: &mut S,
        storage_key: S::Key,
        // mobject: &M,
        // mobject_presentation: &mut M::MobjectPresentation,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    );
    fn render(
        &self,
        storage: &S,
        storage_key: &S::Key,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    );
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct StaticTimeline<M> {
    mobject: Arc<M>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct DynamicTimeline<M, TE, U> {
    mobject: Arc<M>,
    time_eval: Arc<TE>,
    update: Arc<U>,
}

impl<S, M> Timeline<S> for StaticTimeline<M>
where
    S: Storage,
    M: Mobject,
{
    fn allocate(&self, storage: &S) -> S::Key {
        storage.static_allocate(&self.mobject)
    }

    fn prepare(
        &self,
        _time: f32,
        _time_interval: Range<f32>,
        storage: &mut S,
        storage_key: S::Key,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) {
        storage
            .static_entry(storage_key)
            .or_insert_with(|| Box::new(self.mobject.presentation(device)));
    }

    fn render(
        &self,
        storage: &S,
        storage_key: &S::Key,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        let mobject_presentation = storage.static_get(storage_key).unwrap();
        mobject_presentation.render(encoder, target);
    }
}

impl<S, M, TE, U> Timeline<S> for DynamicTimeline<M, TE, U>
where
    S: Storage,
    M: Mobject,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, M>,
{
    fn allocate(&self, storage: &S) -> S::Key {
        storage.dynamic_allocate(&self.mobject, &self.update)
    }

    fn prepare(
        &self,
        time: f32,
        time_interval: Range<f32>,
        storage: &mut S,
        storage_key: S::Key,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        let mobject_presentation = (storage
            .dynamic_entry(storage_key)
            .or_insert_with(|| Box::new(self.mobject.presentation(device)))
            as &mut dyn Any)
            .downcast_mut::<M::MobjectPresentation>()
            .unwrap();
        self.update.update_presentation(
            self.time_eval.time_eval(time, time_interval),
            &self.mobject,
            mobject_presentation,
            device,
            queue,
            format,
        );
    }

    fn render(
        &self,
        storage: &S,
        storage_key: &S::Key,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        let mobject_presentation = storage.dynamic_get(storage_key).unwrap();
        mobject_presentation.render(encoder, target);
    }
}

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub struct SteadyTimeline;

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub struct DynamicTimeline<U> {
//     // time_eval: TE,
//     update: U,
// }

// impl<TM, M> Timeline<TM, M> for SteadyTimeline
// where
//     TM: TimeMetric,
//     M: Mobject,
// {
//     fn prepare(
//         &self,
//         _time_metric: TM,
//         _mobject: &M,
//         _mobject_presentation: &mut M::MobjectPresentation,
//         _device: &wgpu::Device,
//         _queue: &wgpu::Queue,
//         _format: wgpu::TextureFormat,
//     ) {
//     }

//     fn render(
//         &self,
//         mobject_presentation: &M::MobjectPresentation,
//         encoder: &mut wgpu::CommandEncoder,
//         target: &wgpu::TextureView,
//     ) {
//         mobject_presentation.render(encoder, target);
//     }
// }

// impl<TM, M, U> Timeline<TM, M> for DynamicTimeline<U>
// where
//     TM: TimeMetric,
//     M: Mobject,
//     U: Update<TM, M>,
// {
//     fn prepare(
//         &self,
//         time_metric: TM,
//         mobject: &M,
//         mobject_presentation: &mut M::MobjectPresentation,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) {
//         self.update.update_presentation(
//             time_metric,
//             mobject,
//             mobject_presentation,
//             device,
//             queue,
//             format,
//         );
//     }

//     fn render(
//         &self,
//         mobject_presentation: &M::MobjectPresentation,
//         encoder: &mut wgpu::CommandEncoder,
//         target: &wgpu::TextureView,
//     ) {
//         mobject_presentation.render(encoder, target);
//     }
//     // fn prepare(
//     //     &self,
//     //     _time: f32,
//     //     _time_interval: Range<f32>,
//     //     key: &S::Key,
//     //     storage: &mut S,
//     //     device: &wgpu::Device,
//     //     _queue: &wgpu::Queue,
//     //     _format: wgpu::TextureFormat,
//     // ) {
//     //     storage.get_mut_or_insert(key, || self.mobject.presentation(device));
//     // }

//     // fn render(
//     //     &self,
//     //     key: &S::Key,
//     //     storage: &S,
//     //     encoder: &mut wgpu::CommandEncoder,
//     //     target: &wgpu::TextureView,
//     // ) {
//     //     let presentation = storage.get_unwrap::<M::MobjectPresentation>(key);
//     //     // let mut render_pass = new_render_pass(encoder, target, wgpu::LoadOp::Load);
//     //     presentation.render(encoder, target);
//     // }
// }

trait TimeEval:
    'static + Clone + Send + Sync + Debug + serde::de::DeserializeOwned + serde::Serialize
{
    type OutputTimeMetric: TimeMetric;

    fn time_eval(&self, time: f32, time_interval: Range<f32>) -> Self::OutputTimeMetric;
}

trait IncreasingTimeEval: TimeEval {}

// impl<M, TM, R, S> Timeline<S> for IndeterminedTimelineNode<M, TM, R>
// where
//     M: Mobject,
//     TM: TimeMetric,
//     R: Rate<TM>,
//     S: Storage,
// {
//     fn prepare(
//         &self,
//         time: f32,
//         time_interval: Range<f32>,
//         key: &S::Key,
//         storage: &mut S,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) {
//         storage.get_mut_or_insert(key, || self.mobject.presentation(device));
//     }

//     fn render(
//         &self,
//         key: &S::Key,
//         storage: &S,
//         encoder: &mut wgpu::CommandEncoder,
//         target: &wgpu::TextureView,
//     ) {
//         let presentation = storage.get_unwrap::<M::MobjectPresentation>(key);
//         // let mut render_pass = new_render_pass(encoder, target, wgpu::LoadOp::Load);
//         presentation.render(encoder, target);
//     }
// }

// trait DynTimeEntry {
//     fn prepare(
//         &self,
//         time: f32,
//         storage: &mut dyn Storage,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) {
//         self.update.update_presentation(
//             self.time_transform.eval(time, time_interval),
//             mobject,
//             mobject_presentation,
//             device,
//             queue,
//             format,
//         );
//     }

//     fn render(
//         &self,
//         storage: &dyn Storage,
//         encoder: &mut wgpu::CommandEncoder,
//         target: &wgpu::TextureView,
//     ) {
//         mobject_presentation.render(encoder, target);
//     }
// }

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct TimelineEntry<S>
where
    S: Storage,
{
    time_interval: Range<f32>,
    storage_key: S::Key,
    timeline: serde_traitobject::Box<dyn Timeline<S>>,
}

impl<S> TimelineEntry<S>
where
    S: Storage,
{
    fn allocate_in<T>(time_interval: Range<f32>, timeline: T, storage: &S) -> Self
    where
        T: Timeline<S>,
    {
        Self {
            time_interval,
            storage_key: timeline.allocate(storage),
            timeline: serde_traitobject::Box::new(timeline),
        }
    }

    fn rescale_time_interval<TE>(
        &mut self,
        time_eval: &TE,
        source_time_interval: &Range<f32>,
        target_time_interval: &Range<f32>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    {
        self.time_interval = target_time_interval.start
            + (target_time_interval.end - target_time_interval.start)
                * *time_eval.time_eval(self.time_interval.start, source_time_interval.clone())
            ..target_time_interval.start
                + (target_time_interval.end - target_time_interval.start)
                    * *time_eval.time_eval(self.time_interval.end, source_time_interval.clone());
    }
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
        time_interval: Range<f32>,
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
    // children_time_interval: Range<f32>,
    // children_timeline_entries: CTE,
}

// #[derive(Clone, Debug)]
// pub struct CollapsedTimelineState<M> {
//     mobject: Arc<M>,
// }

// #[derive(Clone, Debug)]
// pub struct IndeterminedTimelineState<M, TM, R> {
//     mobject: Arc<M>,
//     time_transform: TE,
// }

// #[derive(Clone, Debug)]
// pub struct UpdateTimelineState<M, TM, R, U> {
//     mobject: Arc<M>,
//     time_transform: TE,
//     update: U,
// }

// #[derive(Clone, Debug)]
// pub struct ActionTimelineState<M, TM, R, U> {
//     mobject: Arc<M>,
//     time_transform: TE,
//     update: U,
// }

// #[derive(Clone, Debug)]
// pub struct ConstructTimelineState<M, R> {
//     mobject: Arc<M>,
//     time_transform: TimeTransform<NormalizedTimeMetric, R>,
//     time_interval: Range<f32>,
//     // timelines: Vec<(Range<f32>, Arc<dyn Timeline<S>>)>,
// }

impl<S, M> TimelineState<S> for CollapsedTimelineState<M>
where
    S: Storage,
    M: Mobject,
{
    type OutputTimelineState = CollapsedTimelineState<M>;

    fn allocate_timeline_entries(
        self,
        time_interval: Range<f32>,
        supervisor: &Supervisor<S>,
        mut timeline_entries_sink: TimelineEntriesSink<S>,
    ) -> Self::OutputTimelineState {
        timeline_entries_sink.extend_one(TimelineEntry::allocate_in(
            time_interval,
            StaticTimeline {
                mobject: self.mobject.clone(),
            },
            &supervisor.storage,
        ));
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
        time_interval: Range<f32>,
        supervisor: &Supervisor<S>,
        mut timeline_entries_sink: TimelineEntriesSink<S>,
    ) -> Self::OutputTimelineState {
        timeline_entries_sink.extend_one(TimelineEntry::allocate_in(
            time_interval,
            StaticTimeline {
                mobject: self.mobject.clone(),
            },
            &supervisor.storage,
        ));
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
        time_interval: Range<f32>,
        supervisor: &Supervisor<S>,
        mut timeline_entries_sink: TimelineEntriesSink<S>,
    ) -> Self::OutputTimelineState {
        timeline_entries_sink.extend_one(TimelineEntry::allocate_in(
            time_interval.clone(),
            DynamicTimeline {
                mobject: self.mobject.clone(),
                time_eval: self.time_eval.clone(),
                update: self.update.clone(),
            },
            &supervisor.storage,
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
        time_interval: Range<f32>,
        supervisor: &Supervisor<S>,
        mut timeline_entries_sink: TimelineEntriesSink<S>,
    ) -> Self::OutputTimelineState {
        let child_supervisor = Supervisor::new(supervisor.config, supervisor.storage);
        let child_time_start = child_supervisor.time();
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
            .archive(|_, _, timeline_state| timeline_state);
        let children_time_interval = child_time_start..child_supervisor.time();
        timeline_entries_sink.extend(child_supervisor.iter_timeline_entries().map(
            |mut timeline_entry| {
                timeline_entry.rescale_time_interval(
                    self.time_eval.as_ref(),
                    &children_time_interval,
                    &time_interval,
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

// impl TimeMetric for () {}

#[derive(Clone)]
pub struct NormalizedTimeMetric(f32);

impl Deref for NormalizedTimeMetric {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TimeMetric for NormalizedTimeMetric {}

#[derive(Clone)]
pub struct DenormalizedTimeMetric(f32);

impl Deref for DenormalizedTimeMetric {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TimeMetric for DenormalizedTimeMetric {}

// impl TimeEval for () {
//     type OutputTimeMetric = ();

//     fn time_eval(&self, _time: f32, _time_interval: Range<f32>) -> Self::OutputTimeMetric {}
// }

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct NormalizedTimeEval;

impl TimeEval for NormalizedTimeEval {
    type OutputTimeMetric = NormalizedTimeMetric;

    fn time_eval(&self, time: f32, time_interval: Range<f32>) -> Self::OutputTimeMetric {
        NormalizedTimeMetric(
            (time_interval.end - time_interval.start != 0.0)
                .then(|| (time - time_interval.start) / (time_interval.end - time_interval.start))
                .unwrap_or_default(),
        )
    }
}

impl IncreasingTimeEval for NormalizedTimeEval {}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DenormalizedTimeEval;

impl TimeEval for DenormalizedTimeEval {
    type OutputTimeMetric = DenormalizedTimeMetric;

    fn time_eval(&self, time: f32, time_interval: Range<f32>) -> Self::OutputTimeMetric {
        DenormalizedTimeMetric(time - time_interval.start)
    }
}

impl IncreasingTimeEval for DenormalizedTimeEval {}

// #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
// struct TimeTransform<TM, R> {
//     time_metric: TM,
//     rate: R,
// }

// impl<TM, R> TimeTransform<TM, R>
// where
//     TM: TimeMetric,
//     R: Rate<TM>,
// {
//     fn eval(&self, time: f32, time_interval: Range<f32>) -> f32 {
//         self.rate.eval(self.time_metric.eval(time, time_interval))
//     }
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

// pub trait ApplyAct<TM, M>: Sized
// where
//     TM: TimeMetric,
//     M: Mobject,
// {
//     type Output<A>
//     where
//         A: Act<TM, M>;

//     fn apply_act<A>(self, act: A) -> Self::Output<A>
//     where
//         A: Act<TM, M>;
// }

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

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// struct TimelineEntry<S>
// where
//     S: Storage,
// {
//     key: S::Key,
//     time_interval: Range<f32>,
//     timeline: serde_traitobject::Arc<dyn Timeline>,
// }

// struct UnarchivedTimeline<M, TS> {
//     mobject: Arc<M>,
//     timeline_state: TS,
// }

// enum TimelineSlot<S>
// where
//     S: Storage,
// {
//     Unarchived(Arc<dyn Any>),
//     Archived(Vec<TimelineEntry<S>>),
// }

pub struct Supervisor<'c, 's, S>
where
    S: Storage,
{
    config: &'c Config,
    storage: &'s S,
    time: RefCell<Arc<f32>>,
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

    fn iter_timeline_entries(self) -> impl Iterator<Item = TimelineEntry<S>> {
        self.timeline_slots
            .into_inner()
            .into_iter()
            .flat_map(|(timeline, timeline_entries)| {
                assert!(timeline.is_none());
                timeline_entries
            })
    }

    // pub(crate) fn into_timeline_entries(self) -> Vec<TimelineEntry<S>> {
    //     self.timeline_items
    //         .into_inner()
    //         .into_iter()
    //         .filter_map(|(time_interval, timeline)| {
    //             time_interval
    //                 .zip(timeline)
    //                 .map(|(time_interval, timeline)| {
    //                     let timeline = serde_traitobject::Arc::from(timeline);
    //                     TimelineEntry {
    //                         key: self.storage.generate_key(&timeline),
    //                         time_interval,
    //                         timeline,
    //                     }
    //                 })
    //         })
    //         .collect()
    // }

    // pub(crate) fn visit<V, VO, F, FO>(config: &'c Config, visitor: V, f: F) -> FO
    // where
    //     V: for<'s> FnOnce(&'s Self) -> VO,
    //     F: FnOnce(f32, TimelineEntries, VO) -> FO,
    // {
    //     let supervisor = Self {
    //         config,
    //         time: RefCell::new(Arc::new(0.0)),
    //         timeline_items: RefCell::new(Vec::new()),
    //     };
    //     let visitor_output = visitor(&supervisor);
    //     f(
    //         *supervisor.arc_time(),
    //         ,
    //         visitor_output,
    //     )
    // }

    fn arc_time(&self) -> Arc<f32> {
        self.time.borrow().clone()
    }

    pub fn time(&self) -> f32 {
        *self.arc_time()
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

    pub fn wait(&self, delta_time: f32) {
        assert!(
            delta_time.is_sign_positive(),
            "`Supervisor::wait` expects a non-negative `delta_time`, got {delta_time}",
        );
        let mut time = self.time.borrow_mut();
        *time = Arc::new(**time + delta_time);
    }
}

// trait Prepare {
//     type Mobject: Mobject;

//     fn prepare(
//         &self,
//         time: f32,
//         presentation: &mut <Self::Mobject as Mobject>::MobjectPresentation,
//         reference_mobject: &Self::Mobject,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     );
// }

// trait TimelineNode<S>:
//     'static + Send + Sync + Debug + serde_traitobject::Deserialize + serde_traitobject::Serialize
// where
//     S: Storage,
// {
//     type MobjectTimelineEntries: Iterator<Item = MobjectTimelineEntry<S>>;

//     fn collect_entries(
//         self,
//         time_interval: Range<f32>,
//         storage: &S,
//     ) -> Self::MobjectTimelineEntries;

//     // type Descendants: Iterator<Item = Arc<dyn MobjectTimelineEntry<S>>>;
//     // type Presentation: Send;

//     // fn presentation(&self, device: &wgpu::Device) -> Self::Presentation;
//     // fn prepare(
//     //     &self,
//     //     id: TimelineId,
//     //     time_interval: Range<f32>,
//     //     time: f32,
//     //     // presentation: &mut Self::Presentation,
//     //     device: &wgpu::Device,
//     //     queue: &wgpu::Queue,
//     //     format: wgpu::TextureFormat,
//     //     storage: &mut iced::widget::shader::Storage,
//     //     bounds: &iced::Rectangle,
//     //     viewport: &iced::widget::shader::Viewport,
//     // );
//     // fn render(
//     //     &self,
//     //     id: TimelineId,
//     //     // time_interval: Range<f32>,
//     //     // time: f32,
//     //     // presentation: &Self::Presentation,
//     //     encoder: &mut wgpu::CommandEncoder,
//     //     storage: &iced::widget::shader::Storage,
//     //     target: &wgpu::TextureView,
//     //     clip_bounds: &iced::Rectangle<u32>,
//     // );
// }

// trait TimelineLeaf<S>:
//     'static + Send + Sync + Debug + serde_traitobject::Deserialize + serde_traitobject::Serialize
// where
//     S: Storage,
// {
//     fn generate_key(&self, storage: &S) -> S::Key;
//     fn dyn_prepare(
//         &self,
//         time: f32,
//         storage: &mut S,
//         key: &S::Key,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     );
// }

// trait TimelinePrepare {
//     type MobjectPresentation: MobjectPresentation;
//     // type Rate: Rate;

//     fn prepare(
//         &self,
//         time: f32,
//         presentation: &mut Self::MobjectPresentation,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     );
// }

// struct MobjectTimeline<T, M> {
//     timeline: T,
//     mobject: M,
//     // rate: R,
//     // mobject_storage_key: K,
// }

// impl<T, S> TimelineNode<S> for T
// where
//     T: TimelineLeaf<S>,
//     S: Storage,
// {
//     type MobjectTimelineEntries = std::iter::Once<MobjectTimelineEntry<S>>;

//     fn collect_entries(
//         self,
//         time_interval: Range<f32>,
//         storage: &S,
//     ) -> Self::MobjectTimelineEntries {
//         std::iter::once(MobjectTimelineEntry {
//             time_interval,
//             storage_key: self.generate_key(storage),
//             timeline: Arc::new(self),
//         })
//     }
// }

// impl<T, M, S> TimelineLeaf<S> for MobjectTimeline<T, M>
// where
//     T: TimelinePrepare<MobjectPresentation = M::MobjectPresentation>,
//     M: Mobject,
//     // R: Rate,
//     S: Storage,
// {
//     fn generate_key(&self, storage: &S) -> S::Key {
//         storage.generate_key(&self.mobject) // TODO: serde_traitobject::Arc? timeline?
//     }

//     fn dyn_prepare(
//         &self,
//         time: f32,
//         storage: &mut S,
//         key: &S::Key,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) {
//         let presentation = storage.get_mut_or_insert(key, || self.mobject.presentation(device));
//         self.timeline
//             .prepare(time, presentation, device, queue, format);
//     }
// }

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// struct MobjectTimelineEntry<S>
// where
//     S: Storage,
// {
//     time_interval: Range<f32>,
//     storage_key: S::Key,
//     timeline: Arc<dyn TimelineLeaf<S>>,
// }

// // struct CompositeTimeline

// impl<I, S> TimelineNode<S> for I
// where
//     I: Iterator<Item = MobjectTimelineEntry<S>>,
// {
//     type MobjectTimelineEntries = Self;

//     fn collect_entries(
//         self,
//         time_interval: Range<f32>,
//         storage: &S,
//     ) -> Self::MobjectTimelineEntries {
//     }
// }

// // trait DynTimeline:
// //     Send + Sync + Debug + serde_traitobject::Deserialize + serde_traitobject::Serialize
// // {
// //     fn dyn_prepare(
// //         &self,
// //         hash: u64,
// //         time_interval: Range<f32>,
// //         time: f32,
// //         device: &wgpu::Device,
// //         queue: &wgpu::Queue,
// //         format: wgpu::TextureFormat,
// //         storage: &mut iced::widget::shader::Storage,
// //         bounds: &iced::Rectangle,
// //         viewport: &iced::widget::shader::Viewport,
// //     );
// //     fn dyn_render(
// //         &self,
// //         hash: u64,
// //         time_interval: Range<f32>,
// //         time: f32,
// //         encoder: &mut wgpu::CommandEncoder,
// //         storage: &iced::widget::shader::Storage,
// //         target: &wgpu::TextureView,
// //         clip_bounds: &iced::Rectangle<u32>,
// //     );
// // }

// // impl<T> DynTimeline for T
// // where
// //     T: Timeline,
// // {
// //     fn dyn_prepare(
// //         &self,
// //         hash: u64,
// //         time_interval: Range<f32>,
// //         time: f32,
// //         device: &wgpu::Device,
// //         queue: &wgpu::Queue,
// //         format: wgpu::TextureFormat,
// //         storage: &mut iced::widget::shader::Storage,
// //         bounds: &iced::Rectangle,
// //         viewport: &iced::widget::shader::Viewport,
// //     ) {

// //         self.prepare(
// //             time_interval,
// //             time,
// //             &mut presentation,
// //             device,
// //             queue,
// //             format,
// //             storage,
// //             bounds,
// //             viewport,
// //         );
// //     }

// //     fn dyn_render(
// //         &self,
// //         hash: u64,
// //         time_interval: Range<f32>,
// //         time: f32,
// //         encoder: &mut wgpu::CommandEncoder,
// //         storage: &iced::widget::shader::Storage,
// //         target: &wgpu::TextureView,
// //         clip_bounds: &iced::Rectangle<u32>,
// //     ) {
// //         let presentation_map = storage
// //             .get::<dashmap::DashMap<u64, T::Presentation>>()
// //             .unwrap();
// //         let presentation = presentation_map.get(&hash).unwrap();
// //         self.render(
// //             time_interval,
// //             time,
// //             &presentation,
// //             encoder,
// //             storage,
// //             target,
// //             clip_bounds,
// //         );
// //     }
// // }

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// struct TimelineEntry {
//     id: TimelineId,
//     time_interval: Range<f32>,
//     timeline: serde_traitobject::Arc<dyn Timeline>,
// }

// #[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
// pub struct TimelineEntries(Arc<Vec<TimelineEntry>>);

// impl TimelineEntries {
//     fn prepare(
//         &self,
//         time: f32,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         storage: &mut iced::widget::shader::Storage,
//         bounds: &iced::Rectangle,
//         viewport: &iced::widget::shader::Viewport,
//     ) {
//         for timeline_entry in &*self.0 {
//             if timeline_entry.time_interval.contains(&time) {
//                 timeline_entry.timeline.prepare(
//                     timeline_entry.id,
//                     timeline_entry.time_interval.clone(),
//                     time,
//                     device,
//                     queue,
//                     format,
//                     storage,
//                     bounds,
//                     viewport,
//                 );
//             }
//         }
//     }

//     fn render(
//         &self,
//         time: f32,
//         encoder: &mut wgpu::CommandEncoder,
//         storage: &iced::widget::shader::Storage,
//         target: &wgpu::TextureView,
//         clip_bounds: &iced::Rectangle<u32>,
//     ) {
//         for timeline_entry in &*self.0 {
//             if timeline_entry.time_interval.contains(&time) {
//                 timeline_entry.timeline.render(
//                     timeline_entry.id,
//                     // timeline_entry.time_interval.clone(),
//                     // time,
//                     encoder,
//                     storage,
//                     target,
//                     clip_bounds,
//                 );
//             }
//         }
//     }
// }

// #[derive(Debug)]
// pub struct ScenePrimitive {
//     time: f32,
//     timeline_entries: TimelineEntries,
//     background_color: wgpu::Color,
// }

// impl iced::widget::shader::Primitive for ScenePrimitive {
//     fn prepare(
//         &self,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         storage: &mut iced::widget::shader::Storage,
//         bounds: &iced::Rectangle,
//         viewport: &iced::widget::shader::Viewport,
//     ) {
//         // TODO: clean up storage for older scenes
//         self.timeline_entries
//             .prepare(self.time, device, queue, format, storage, bounds, viewport);
//     }

//     fn render(
//         &self,
//         encoder: &mut wgpu::CommandEncoder,
//         storage: &iced::widget::shader::Storage,
//         target: &wgpu::TextureView,
//         clip_bounds: &iced::Rectangle<u32>,
//     ) {
//         new_render_pass(
//             encoder,
//             target,
//             clip_bounds,
//             wgpu::LoadOp::Clear(self.background_color),
//         );
//         self.timeline_entries
//             .render(self.time, encoder, storage, target, clip_bounds);
//     }
// }

pub struct Alive<'c, 's, 'sv, S, TS>
where
    S: Storage,
    TS: TimelineState<S>,
{
    supervisor: &'sv Supervisor<'c, 's, S>,
    spawn_time: Arc<f32>,
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
        spawn_time: Arc<f32>,
        timeline_state: TS,
        // children_timelines: Vec<(Range<f32>, Arc<dyn Timeline>)>,
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
        F: FnOnce(&'sv Supervisor<'c, 's, S>, Arc<f32>, TS::OutputTimelineState) -> FO,
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
        // let timeline = Arc::new(self.timeline.take().unwrap());
        let output_timeline_state = timeline_state.allocate_timeline_entries(
            *spawn_time..*archive_time,
            supervisor,
            timeline_entries_sink,
        );
        f(supervisor, archive_time, output_timeline_state)
        // let (output, timelines) = if Arc::ptr_eq(&spawn_time, &archive_time) {
        // } else {
        //     (
        //         f(
        //             supervisor,
        //             archive_time.clone(),
        //             unarchived_timeline.mobject.clone(),
        //             unarchived_timeline.time_eval.clone(),
        //             &unarchived_timeline.timeline_state,
        //         ),
        //         unarchived_timeline
        //             .timeline_state
        //             .allocate_timeline_entries(
        //                 *spawn_time..*archive_time,
        //                 unarchived_timeline.mobject,
        //                 unarchived_timeline.time_eval,
        //                 &supervisor.storage,
        //             ),
        //     )
        // };
        // *timeline_slot = TimelineSlot::Archived(timelines);
        // output
        // if Arc::ptr_eq(&spawn_time, &archive_time) {
        //     let _ = timeline.take();
        //     // let index = timeline_entries
        //     //     .iter()
        //     //     .rposition(|timeline_entry| {
        //     //         std::ptr::eq(
        //     //             &*timeline as *const T as *const (),
        //     //             &timeline_entry.timeline.into() as *const dyn DynTimeline as *const (),
        //     //         )
        //     //     })
        //     //     .unwrap();
        //     // timeline_entries.remove(index);
        //     // supervisor
        //     //     .push(*self.spawn_time..*archive_time, timeline.clone());
        // } else {
        //     let _ = time_interval.insert(*spawn_time..*archive_time);
        // }
    }

    // fn archive<F, FO>(self, f: F) -> FO
    // where
    //     T: Clone + Timeline,
    //     F: FnOnce(&'w Supervisor, Range<f32>, T) -> FO,
    // {
    //     let Alive {
    //         supervisor,
    //         spawn_time,
    //         timeline,
    //     } = self;
    //     let arc_time_interval = spawn_time..supervisor.time();
    //     let time_interval = *arc_time_interval.start..*arc_time_interval.end;
    //     if Arc::ptr_eq(&arc_time_interval.start, &arc_time_interval.end) {
    //         f(supervisor, time_interval, timeline)
    //     } else {
    //         let output = f(supervisor, time_interval.clone(), timeline.clone());
    //         supervisor.push(time_interval, timeline);
    //         output
    //     }
    // }
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
                        time_eval: Arc::unwrap_or_clone(output_timeline_state.time_eval),
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

// impl<'c, 's, 'sv, S, M, TM, R> ApplyAct<M, TM>
//     for Alive<'c, 's, 'sv, S, IndeterminedTimelineState<M, TM, R>>
// where
//     M: Mobject,
//     TM: TimeMetric,
//     R: Rate<TM>,
// {
//     type Output<A> = Alive<'c, 's, 'sv, S, ActionTimelineState<M, TM, R, A::Update>>
//     where
//         A: Act<M, TM>;

//     #[must_use]
//     fn apply_act<A>(mut self, act: A) -> Self::Output<A>
//     where
//         A: Act<M, TM>,
//     {
//         self.archive(|supervisor, archive_time, output_timeline_state| {
//             let update = act.act(&timeline_state.mobject);
//             Alive::new(
//                 supervisor,
//                 archive_time,
//                 ActionTimelineState {
//                     mobject: timeline_state.mobject,
//                     time_transform: timeline_state.time_transform,
//                     update,
//                 },
//             )
//         })
//     }
// }

// impl<'c, 's, 'sv, S, M, TM, R, U> ApplyAct<M, TM>
//     for Alive<'c, 's, 'sv, S, ActionTimelineState<M, TM, R, U>>
// where
//     M: Mobject,
//     TM: TimeMetric,
//     R: Rate<TM>,
//     U: Update<M, TM>,
// {
//     type Output<A> = Alive<'c, 's, 'sv, S, ActionTimelineState<M, TM, R, ComposeUpdate<A::Update, U>>>
//     where
//         A: Act<M, TM>;

//     #[must_use]
//     fn apply_act<A>(mut self, act: A) -> Self::Output<A>
//     where
//         A: Act<M, TM>,
//     {
//         self.archive(|supervisor, archive_time, output_timeline_state| {
//             let update = ComposeUpdate(act.act(&timeline_state.mobject), timeline_state.update);
//             Alive::new(
//                 supervisor,
//                 archive_time,
//                 ActionTimelineState {
//                     mobject: timeline_state.mobject,
//                     time_transform: timeline_state.time_transform,
//                     update,
//                 },
//             )
//         })
//     }
// }

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
    fn play(self, delta_time: f32) -> Self::Output;
}

impl<'c, 's, 'sv, S, TS> CollapseExt for Alive<'c, 's, 'sv, S, TS>
where
    S: Storage,
    TS: TimelineState<S>,
    Self: Collapse,
{
    #[must_use]
    fn play(self, delta_time: f32) -> Self::Output {
        self.supervisor.wait(delta_time);
        self.collapse()
    }
}

// #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
// pub struct IdentityRate;

// impl<TM> Rate<TM> for IdentityRate
// where
//     TM: TimeMetric,
// {
//     type OutputMetric = TM;

//     fn eval(&self, time_metric: f32) -> f32 {
//         time_metric
//     }
// }

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct RateComposeTimeEval<R, TE> {
    rate: R,
    time_eval: TE,
}

impl<R, TE> TimeEval for RateComposeTimeEval<R, TE>
where
    R: Rate<TE::OutputTimeMetric>,
    TE: TimeEval,
{
    type OutputTimeMetric = R::OutputTimeMetric;

    fn time_eval(&self, time: f32, time_interval: Range<f32>) -> Self::OutputTimeMetric {
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

// #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
// pub struct IdentityPrepare;

// impl<TM, M> Prepare<TM, M> for IdentityPrepare
// where
//     TM: TimeMetric,
//     M: Mobject,
// {
//     fn prepare(
//         &self,
//         _time_metric: TM,
//         _mobject: &M,
//         _mobject_presentation: &mut M::MobjectPresentation,
//         _device: &wgpu::Device,
//         _queue: &wgpu::Queue,
//         _format: wgpu::TextureFormat,
//     ) {
//     }
// }

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ComposeUpdate<U0, U1>(U0, U1);

impl<M, TM, U0, U1> Update<TM, M> for ComposeUpdate<U0, U1>
where
    M: Mobject,
    TM: TimeMetric,
    U0: Update<TM, M>,
    U1: Update<TM, M>,
{
    fn update(&self, time_metric: TM, mobject: &mut M) {
        self.1.update(time_metric.clone(), mobject);
        self.0.update(time_metric, mobject);
    }

    fn update_presentation(
        &self,
        time_metric: TM,
        mobject: &M,
        mobject_presentation: &mut M::MobjectPresentation,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        self.1.update_presentation(
            time_metric.clone(),
            mobject,
            mobject_presentation,
            device,
            queue,
            format,
        );
        self.0.update_presentation(
            time_metric,
            mobject,
            mobject_presentation,
            device,
            queue,
            format,
        );
    }
}

// // steady

// #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
// pub struct SteadyTimeline<M> {
//     mobject: M,
// }

// impl<M> Timeline for SteadyTimeline<M>
// where
//     M: Mobject,
// {
//     fn prepare(
//         &self,
//         id: TimelineId,
//         _time_interval: Range<f32>,
//         _time: f32,
//         device: &wgpu::Device,
//         _queue: &wgpu::Queue,
//         _format: wgpu::TextureFormat,
//         storage: &mut iced::widget::shader::Storage,
//         _bounds: &iced::Rectangle,
//         _viewport: &iced::widget::shader::Viewport,
//     ) {
//         id.prepare_presentation(storage, device, &self.mobject, |_| ());
//     }

//     fn render(
//         &self,
//         id: TimelineId,
//         _time_interval: Range<f32>,
//         _time: f32,
//         encoder: &mut wgpu::CommandEncoder,
//         storage: &iced::widget::shader::Storage,
//         target: &wgpu::TextureView,
//         clip_bounds: &iced::Rectangle<u32>,
//     ) {
//         id.render_presentation::<M, _>(storage, |presentation| {
//             let mut render_pass = new_render_pass(encoder, target, clip_bounds, wgpu::LoadOp::Load);
//             presentation.draw(&mut render_pass);
//         });
//     }

//     // type Presentation = M::MobjectPresentation;

//     // fn presentation(&self, device: &wgpu::Device) -> Self::Presentation {
//     //     self.mobject.presentation(device)
//     // }

//     // fn prepare(
//     //     &self,
//     //     _time_interval: Range<f32>,
//     //     _time: f32,
//     //     _device: &wgpu::Device,
//     //     _queue: &wgpu::Queue,
//     //     _format: wgpu::TextureFormat,
//     //     _presentation: &mut Self::Presentation,
//     //     _bounds: &iced::Rectangle,
//     //     _viewport: &iced::widget::shader::Viewport,
//     // ) {
//     // }

//     // fn render(
//     //     &self,
//     //     _time_interval: Range<f32>,
//     //     _time: f32,
//     //     encoder: &mut wgpu::CommandEncoder,
//     //     presentation: &Self::Presentation,
//     //     target: &wgpu::TextureView,
//     //     clip_bounds: &iced::Rectangle<u32>,
//     // ) {
//     //     let mut render_pass = new_render_pass(
//     //         encoder,
//     //         target,
//     //         clip_bounds,
//     //         wgpu::LoadOp::Load,
//     //     );
//     //     presentation.draw(&mut render_pass);
//     // }
// }

// // dynamic

// pub trait ApplyRate: Sized {
//     type Output<R>
//     where
//         R: Rate;

//     fn apply_rate<R>(self, rate: R) -> Self::Output<R>
//     where
//         R: Rate;
// }

// pub trait DynamicTimelineContent:
//     'static + Clone + Send + Sync + Debug + serde::de::DeserializeOwned + serde::Serialize
// {
//     // type ContentPresentation: Send;
//     type CollapseOutput: Mobject;

//     // fn content_presentation(
//     //     &self,
//     //     device: &wgpu::Device,
//     // ) -> Self::ContentPresentation;
//     fn content_prepare(
//         &self,
//         id: TimelineId,
//         time: f32,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         storage: &mut iced::widget::shader::Storage,
//         bounds: &iced::Rectangle,
//         viewport: &iced::widget::shader::Viewport,
//     );
//     fn content_render(
//         &self,
//         id: TimelineId,
//         time: f32,
//         encoder: &mut wgpu::CommandEncoder,
//         storage: &iced::widget::shader::Storage,
//         target: &wgpu::TextureView,
//         clip_bounds: &iced::Rectangle<u32>,
//     );
//     fn content_collapse(self, time: f32) -> Self::CollapseOutput;
// }

// #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
// pub struct DynamicTimeline<CO, ME, R> {
//     content: CO,
//     metric: ME,
//     rate: R,
// }

// impl<CO, ME, R> Timeline for DynamicTimeline<CO, ME, R>
// where
//     CO: DynamicTimelineContent,
//     ME: DynamicTimelineMetric,
//     R: Rate,
// {
//     // type Presentation = CO::ContentPresentation;

//     // fn presentation(&self, device: &wgpu::Device) -> Self::Presentation {
//     //     self.content.content_presentation(device)
//     // }

//     fn prepare(
//         &self,
//         id: TimelineId,
//         time_interval: Range<f32>,
//         time: f32,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         storage: &mut iced::widget::shader::Storage,
//         bounds: &iced::Rectangle,
//         viewport: &iced::widget::shader::Viewport,
//     ) {
//         self.content.content_prepare(
//             id,
//             self.rate.eval(self.metric.eval(time, time_interval)),
//             device,
//             queue,
//             format,
//             storage,
//             bounds,
//             viewport,
//         );
//     }

//     fn render(
//         &self,
//         id: TimelineId,
//         time_interval: Range<f32>,
//         time: f32,
//         encoder: &mut wgpu::CommandEncoder,
//         storage: &iced::widget::shader::Storage,
//         target: &wgpu::TextureView,
//         clip_bounds: &iced::Rectangle<u32>,
//     ) {
//         self.content.content_render(
//             id,
//             self.rate.eval(self.metric.eval(time, time_interval)),
//             encoder,
//             storage,
//             target,
//             clip_bounds,
//         );
//     }
// }

// #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
// pub struct IndeterminedTimelineContent<M> {
//     mobject: M,
// }

// impl<M> DynamicTimelineContent for IndeterminedTimelineContent<M>
// where
//     M: Mobject,
// {
//     // type ContentPresentation = M::MobjectPresentation;
//     type CollapseOutput = M;

//     fn content_prepare(
//         &self,
//         id: TimelineId,
//         _time: f32,
//         device: &wgpu::Device,
//         _queue: &wgpu::Queue,
//         _format: wgpu::TextureFormat,
//         storage: &mut iced::widget::shader::Storage,
//         _bounds: &iced::Rectangle,
//         _viewport: &iced::widget::shader::Viewport,
//     ) {
//         id.prepare_presentation(storage, device, &self.mobject, |_| ());
//     }

//     fn content_render(
//         &self,
//         id: TimelineId,
//         _time: f32,
//         encoder: &mut wgpu::CommandEncoder,
//         storage: &iced::widget::shader::Storage,
//         target: &wgpu::TextureView,
//         clip_bounds: &iced::Rectangle<u32>,
//     ) {
//         id.render_presentation::<M, _>(storage, |presentation| {
//             let mut render_pass = new_render_pass(encoder, target, clip_bounds, wgpu::LoadOp::Load);
//             presentation.draw(&mut render_pass)
//         });
//     }

//     // fn content_presentation(
//     //     &self,
//     //     device: &wgpu::Device,
//     // ) -> Self::ContentPresentation {
//     //     self.mobject.presentation(device)
//     // }

//     // fn content_prepare(
//     //     &self,
//     //     _time: f32,
//     //     _device: &wgpu::Device,
//     //     _queue: &wgpu::Queue,
//     //     _format: wgpu::TextureFormat,
//     //     _presentation: &mut Self::ContentPresentation,
//     //     _bounds: &iced::Rectangle,
//     //     _viewport: &iced::widget::shader::Viewport,
//     // ) {
//     // }

//     // fn content_render(
//     //     &self,
//     //     _time: f32,
//     //     encoder: &mut wgpu::CommandEncoder,
//     //     presentation: &Self::ContentPresentation,
//     //     target: &wgpu::TextureView,
//     //     clip_bounds: &iced::Rectangle<u32>,
//     // ) {
//     //     let mut render_pass = new_render_pass(
//     //         encoder,
//     //         target,
//     //         clip_bounds,
//     //         wgpu::LoadOp::Load,
//     //     );
//     //     presentation.draw(&mut render_pass);
//     // }

//     fn content_collapse(self, _time: f32) -> Self::CollapseOutput {
//         self.mobject
//     }
// }

// // action

// pub trait ApplyAct<M>: Sized
// where
//     M: Mobject,
// {
//     type Output<A>
//     where
//         A: Act<M>;

//     fn apply_act<A>(self, act: A) -> Self::Output<A>
//     where
//         A: Act<M>;
// }

// #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
// pub struct ActionTimelineContent<M, D> {
//     mobject: M,
//     diff: D,
// }

// impl<M, MD> DynamicTimelineContent for ActionTimelineContent<M, MD>
// where
//     M: Mobject,
//     MD: MobjectDiff<M>,
// {
//     // type ContentPresentation = M::MobjectPresentation;
//     type CollapseOutput = M;

//     fn content_prepare(
//         &self,
//         id: TimelineId,
//         time: f32,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         _format: wgpu::TextureFormat,
//         storage: &mut iced::widget::shader::Storage,
//         _bounds: &iced::Rectangle,
//         _viewport: &iced::widget::shader::Viewport,
//     ) {
//         id.prepare_presentation(storage, device, &self.mobject, |presentation| {
//             self.diff
//                 .apply_presentation(presentation, &self.mobject, time, queue);
//         });
//     }

//     fn content_render(
//         &self,
//         id: TimelineId,
//         _time: f32,
//         encoder: &mut wgpu::CommandEncoder,
//         storage: &iced::widget::shader::Storage,
//         target: &wgpu::TextureView,
//         clip_bounds: &iced::Rectangle<u32>,
//     ) {
//         id.render_presentation::<M, _>(storage, |presentation| {
//             let mut render_pass = new_render_pass(encoder, target, clip_bounds, wgpu::LoadOp::Load);
//             presentation.draw(&mut render_pass)
//         });
//     }

//     // fn content_presentation(
//     //     &self,
//     //     device: &wgpu::Device,
//     // ) -> Self::ContentPresentation {
//     //     self.mobject.presentation(device)
//     // }

//     // fn content_prepare(
//     //     &self,
//     //     time: f32,
//     //     _device: &wgpu::Device,
//     //     queue: &wgpu::Queue,
//     //     _format: wgpu::TextureFormat,
//     //     presentation: &mut Self::ContentPresentation,
//     //     _bounds: &iced::Rectangle,
//     //     _viewport: &iced::widget::shader::Viewport,
//     // ) {
//     //     self.diff
//     //         .apply_presentation(presentation, &self.mobject, time, queue);
//     // }

//     // fn content_render(
//     //     &self,
//     //     _time: f32,
//     //     encoder: &mut wgpu::CommandEncoder,
//     //     presentation: &Self::ContentPresentation,
//     //     target: &wgpu::TextureView,
//     //     clip_bounds: &iced::Rectangle<u32>,
//     // ) {
//     //     let mut render_pass = new_render_pass(
//     //         encoder,
//     //         target,
//     //         clip_bounds,
//     //         wgpu::LoadOp::Load,
//     //     );
//     //     presentation.draw(&mut render_pass);
//     // }

//     fn content_collapse(self, time: f32) -> Self::CollapseOutput {
//         let mut mobject = self.mobject;
//         self.diff.apply(&mut mobject, time);
//         mobject
//     }
// }

// // continuous

// pub trait ApplyUpdate<M>: Sized
// where
//     M: Mobject,
// {
//     type Output<U>
//     where
//         U: Update<M>;

//     fn apply_update<U>(self, update: U) -> Self::Output<U>
//     where
//         U: Update<M>;
// }

// #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
// pub struct ContinuousTimelineContent<M, U> {
//     mobject: M,
//     update: U,
// }

// impl<M, U> DynamicTimelineContent for ContinuousTimelineContent<M, U>
// where
//     M: Mobject,
//     U: Update<M>,
// {
//     // type ContentPresentation = M::MobjectPresentation;
//     type CollapseOutput = M;

//     fn content_prepare(
//         &self,
//         id: TimelineId,
//         time: f32,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         _format: wgpu::TextureFormat,
//         storage: &mut iced::widget::shader::Storage,
//         _bounds: &iced::Rectangle,
//         _viewport: &iced::widget::shader::Viewport,
//     ) {
//         id.prepare_presentation(storage, device, &self.mobject, |presentation| {
//             self.update
//                 .update_presentation(presentation, &self.mobject, time, device, queue);
//         });
//     }

//     fn content_render(
//         &self,
//         id: TimelineId,
//         _time: f32,
//         encoder: &mut wgpu::CommandEncoder,
//         storage: &iced::widget::shader::Storage,
//         target: &wgpu::TextureView,
//         clip_bounds: &iced::Rectangle<u32>,
//     ) {
//         id.render_presentation::<M, _>(storage, |presentation| {
//             let mut render_pass = new_render_pass(encoder, target, clip_bounds, wgpu::LoadOp::Load);
//             presentation.draw(&mut render_pass);
//         });
//     }

//     // fn content_presentation(
//     //     &self,
//     //     device: &wgpu::Device,
//     // ) -> Self::ContentPresentation {
//     //     self.mobject.presentation(device)
//     // }

//     // fn content_prepare(
//     //     &self,
//     //     time: f32,
//     //     device: &wgpu::Device,
//     //     queue: &wgpu::Queue,
//     //     _format: wgpu::TextureFormat,
//     //     presentation: &mut Self::ContentPresentation,
//     //     _bounds: &iced::Rectangle,
//     //     _viewport: &iced::widget::shader::Viewport,
//     // ) {
//     //     self.update
//     //         .update_presentation(presentation, &self.mobject, time, device, queue);
//     // }

//     // fn content_render(
//     //     &self,
//     //     _time: f32,
//     //     encoder: &mut wgpu::CommandEncoder,
//     //     presentation: &Self::ContentPresentation,
//     //     target: &wgpu::TextureView,
//     //     clip_bounds: &iced::Rectangle<u32>,
//     // ) {
//     //     let mut render_pass = new_render_pass(
//     //         encoder,
//     //         target,
//     //         clip_bounds,
//     //         wgpu::LoadOp::Load,
//     //     );
//     //     presentation.draw(&mut render_pass);
//     // }

//     fn content_collapse(self, time: f32) -> Self::CollapseOutput {
//         let mut mobject = self.mobject;
//         self.update.update(&mut mobject, time);
//         mobject
//     }
// }

// // discrete

// pub trait ApplyConstruct<M>: Sized
// where
//     M: Mobject,
// {
//     type Output<C>
//     where
//         C: Construct<M>;

//     fn apply_construct<C>(self, construct: C) -> Self::Output<C>
//     where
//         C: Construct<M>;
// }

// #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
// pub struct DiscreteTimelineContent<M> {
//     mobject: M,
//     timeline_entries: TimelineEntries,
// }

// impl<M> DynamicTimelineContent for DiscreteTimelineContent<M>
// where
//     M: Mobject,
// {
//     // type ContentPresentation = iced::widget::shader::Storage;
//     type CollapseOutput = M;

//     fn content_prepare(
//         &self,
//         _id: TimelineId,
//         time: f32,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         storage: &mut iced::widget::shader::Storage,
//         bounds: &iced::Rectangle,
//         viewport: &iced::widget::shader::Viewport,
//     ) {
//         self.timeline_entries
//             .prepare(time, device, queue, format, storage, bounds, viewport);
//     }

//     fn content_render(
//         &self,
//         _id: TimelineId,
//         time: f32,
//         encoder: &mut wgpu::CommandEncoder,
//         storage: &iced::widget::shader::Storage,
//         target: &wgpu::TextureView,
//         clip_bounds: &iced::Rectangle<u32>,
//     ) {
//         self.timeline_entries
//             .render(time, encoder, storage, target, clip_bounds);
//     }

//     // fn content_presentation(
//     //     &self,
//     //     _device: &wgpu::Device,
//     // ) -> Self::ContentPresentation {
//     //     iced::widget::shader::Storage::default()
//     // }

//     // fn content_prepare(
//     //     &self,
//     //     time: f32,
//     //     device: &wgpu::Device,
//     //     queue: &wgpu::Queue,
//     //     format: wgpu::TextureFormat,
//     //     presentation: &mut Self::ContentPresentation,
//     //     bounds: &iced::Rectangle,
//     //     viewport: &iced::widget::shader::Viewport,
//     // ) {
//     //     self.timeline_entries
//     //         .prepare(time, device, queue, format, presentation, bounds, viewport);
//     // }

//     // fn content_render(
//     //     &self,
//     //     time: f32,
//     //     encoder: &mut wgpu::CommandEncoder,
//     //     presentation: &Self::ContentPresentation,
//     //     target: &wgpu::TextureView,
//     //     clip_bounds: &iced::Rectangle<u32>,
//     // ) {
//     //     self.timeline_entries
//     //         .render(time, encoder, presentation, target, clip_bounds);
//     // }

//     fn content_collapse(self, _time: f32) -> Self::CollapseOutput {
//         self.mobject
//     }
// }
