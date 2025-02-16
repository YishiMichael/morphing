use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::Range;
use std::sync::Arc;
use std::sync::Weak;

use super::config::Config;
use super::traits::Act;
use super::traits::Construct;
use super::traits::IncreasingRate;
use super::traits::Mobject;
use super::traits::MobjectBuilder;
use super::traits::MobjectPresentation;
use super::traits::Rate;
use super::traits::Storage;
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

trait Timeline<S>:
    'static + Send + Sync + Debug + serde_traitobject::Deserialize + serde_traitobject::Serialize
where
    S: Storage,
{
    fn prepare(
        &self,
        time: f32,
        time_interval: Range<f32>,
        key: &S::Key,
        storage: &mut S,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    );
    fn render(
        &self,
        key: &S::Key,
        storage: &S,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    );
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SteadyTimeline<M> {
    mobject: Arc<M>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DynamicTimeline<M, TM, R, U> {
    mobject: Arc<M>,
    time_metric: TM,
    rate: R,
    update: U,
}

impl<M, S> Timeline<S> for SteadyTimeline<M>
where
    M: Mobject,
    S: Storage,
{
    fn prepare(
        &self,
        _time: f32,
        _time_interval: Range<f32>,
        key: &S::Key,
        storage: &mut S,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) {
        storage.get_mut_or_insert(key, || self.mobject.presentation(device));
    }

    fn render(
        &self,
        key: &S::Key,
        storage: &S,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        let presentation = storage.get_unwrap::<M::MobjectPresentation>(key);
        // let mut render_pass = new_render_pass(encoder, target, wgpu::LoadOp::Load);
        presentation.render(encoder, target);
    }
}

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

impl<M, TM, R, U, S> Timeline<S> for DynamicTimeline<M, TM, R, U>
where
    M: Mobject,
    TM: TimeMetric,
    R: Rate<TM>,
    U: Update<M, TM>,
    S: Storage,
{
    fn prepare(
        &self,
        time: f32,
        time_interval: Range<f32>,
        key: &S::Key,
        storage: &mut S,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        let mobject_presentation =
            storage.get_mut_or_insert(key, || self.mobject.presentation(device));
        self.update.update_presentation(
            mobject_presentation,
            &self.mobject,
            self.time_metric.eval(&self.rate, time, time_interval),
            device,
            queue,
            format,
        );
    }

    fn render(
        &self,
        key: &S::Key,
        storage: &S,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        let presentation = storage.get_unwrap::<M::MobjectPresentation>(key);
        // let mut render_pass = new_render_pass(encoder, target, wgpu::LoadOp::Load);
        presentation.render(encoder, target);
    }
}

trait TimelineState<S>: 'static + Debug
where
    S: Storage,
{
    fn into_timelines(
        self,
        time_interval: Range<f32>,
        children_timelines: Vec<(Range<f32>, Arc<dyn Timeline<S>>)>,
    ) -> Vec<(Range<f32>, Arc<dyn Timeline<S>>)>;
}

#[derive(Clone, Debug)]
pub struct CollapsedTimelineState<M> {
    mobject: Arc<M>,
}

#[derive(Clone, Debug)]
pub struct IndeterminedTimelineState<M, TM, R> {
    mobject: Arc<M>,
    time_metric: TM,
    rate: R,
}

#[derive(Clone, Debug)]
pub struct ActionTimelineState<M, TM, R, U> {
    mobject: Arc<M>,
    time_metric: TM,
    rate: R,
    update: U,
}

#[derive(Clone, Debug)]
pub struct UpdateTimelineState<M, TM, R, U> {
    mobject: Arc<M>,
    time_metric: TM,
    rate: R,
    update: U,
}

#[derive(Clone, Debug)]
pub struct ConstructTimelineState<M, R> {
    mobject: Arc<M>,
    time_metric: NormalizedTimeMetric,
    rate: R,
    time_interval: Range<f32>,
    // timelines: Vec<(Range<f32>, Arc<dyn Timeline<S>>)>,
}

impl<S, M> TimelineState<S> for CollapsedTimelineState<M>
where
    S: Storage,
    M: Mobject,
{
    fn into_timelines(
        self,
        time_interval: Range<f32>,
        _children_timelines: Vec<(Range<f32>, Arc<dyn Timeline<S>>)>,
    ) -> Vec<(Range<f32>, Arc<dyn Timeline<S>>)> {
        Vec::from([(
            time_interval,
            Arc::new(SteadyTimeline {
                mobject: self.mobject,
            }) as Arc<dyn Timeline<S>>,
        )])
    }
}

impl<S, M, TM, R> TimelineState<S> for IndeterminedTimelineState<M, TM, R>
where
    S: Storage,
    M: Mobject,
    TM: TimeMetric,
    R: Rate<TM>,
{
    fn into_timelines(
        self,
        time_interval: Range<f32>,
        _children_timelines: Vec<(Range<f32>, Arc<dyn Timeline<S>>)>,
    ) -> Vec<(Range<f32>, Arc<dyn Timeline<S>>)> {
        Vec::from([(
            time_interval,
            Arc::new(SteadyTimeline {
                mobject: self.mobject,
            }) as Arc<dyn Timeline<S>>,
        )])
    }
}

impl<S, M, TM, R, U> TimelineState<S> for ActionTimelineState<M, TM, R, U>
where
    S: Storage,
    M: Mobject,
    TM: TimeMetric,
    R: Rate<TM>,
    U: Update<M, TM>,
{
    fn into_timelines(
        self,
        time_interval: Range<f32>,
        _children_timelines: Vec<(Range<f32>, Arc<dyn Timeline<S>>)>,
    ) -> Vec<(Range<f32>, Arc<dyn Timeline<S>>)> {
        Vec::from([(
            time_interval,
            Arc::new(DynamicTimeline {
                mobject: self.mobject,
                time_metric: self.time_metric,
                rate: self.rate,
                update: self.update,
            }) as Arc<dyn Timeline<S>>,
        )])
    }
}

impl<S, M, TM, R, U> TimelineState<S> for UpdateTimelineState<M, TM, R, U>
where
    S: Storage,
    M: Mobject,
    TM: TimeMetric,
    R: Rate<TM>,
    U: Update<M, TM>,
{
    fn into_timelines(
        self,
        time_interval: Range<f32>,
        _children_timelines: Vec<(Range<f32>, Arc<dyn Timeline<S>>)>,
    ) -> Vec<(Range<f32>, Arc<dyn Timeline<S>>)> {
        Vec::from([(
            time_interval,
            Arc::new(DynamicTimeline {
                mobject: self.mobject,
                time_metric: self.time_metric,
                rate: self.rate,
                update: self.update,
            }) as Arc<dyn Timeline<S>>,
        )])
    }
}

impl<S, M, R> TimelineState<S> for ConstructTimelineState<M, R>
where
    S: Storage,
    M: Mobject,
    R: IncreasingRate<NormalizedTimeMetric>,
{
    fn into_timelines(
        self,
        time_interval: Range<f32>,
        children_timelines: Vec<(Range<f32>, Arc<dyn Timeline<S>>)>,
    ) -> Vec<(Range<f32>, Arc<dyn Timeline<S>>)> {
        let rescale_time = |time| {
            time_interval.start
                + (time_interval.end - time_interval.start)
                    * self
                        .time_metric
                        .eval(&self.rate, time, self.time_interval.clone())
        };
        children_timelines
            .into_iter()
            .map(|(child_time_interval, child_timeline)| {
                (
                    rescale_time(child_time_interval.start)..rescale_time(child_time_interval.end),
                    child_timeline,
                )
            })
            .collect()
    }
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

pub trait TimeMetric:
    'static + Clone + Send + Sync + Debug + serde::de::DeserializeOwned + serde::Serialize
{
    fn eval<R>(&self, rate: &R, time: f32, time_interval: Range<f32>) -> f32
    where
        R: Rate<Self>;
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct NormalizedTimeMetric;

impl TimeMetric for NormalizedTimeMetric {
    fn eval<R>(&self, rate: &R, time: f32, time_interval: Range<f32>) -> f32
    where
        R: Rate<Self>,
    {
        rate.eval((time - time_interval.start) / (time_interval.end - time_interval.start))
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DenormalizedTimeMetric;

impl TimeMetric for DenormalizedTimeMetric {
    fn eval<R>(&self, rate: &R, time: f32, time_interval: Range<f32>) -> f32
    where
        R: Rate<Self>,
    {
        rate.eval(time - time_interval.start)
    }
}

pub trait Quantize: Sized {
    type Output<TM>
    where
        TM: TimeMetric;

    fn quantize<TM>(self, time_metric: TM) -> Self::Output<TM>
    where
        TM: TimeMetric;
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

pub trait ApplyAct<M, TM>: Sized
where
    M: Mobject,
    TM: TimeMetric,
{
    type Output<A>
    where
        A: Act<M, TM>;

    fn apply_act<A>(self, act: A) -> Self::Output<A>
    where
        A: Act<M, TM>;
}

pub trait ApplyUpdate<M, TM>: Sized
where
    M: Mobject,
    TM: TimeMetric,
{
    type Output<U>
    where
        U: Update<M, TM>;

    fn apply_update<U>(self, update: U) -> Self::Output<U>
    where
        U: Update<M, TM>;
}

pub trait ApplyConstruct<S, M>: Sized
where
    S: Storage,
    M: Mobject,
{
    type Output<C>
    where
        C: Construct<S, M>;

    fn apply_construct<C>(self, construct: C) -> Self::Output<C>
    where
        C: Construct<S, M>;
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct TimelineEntry<S>
where
    S: Storage,
{
    key: S::Key,
    time_interval: Range<f32>,
    timeline: serde_traitobject::Arc<dyn Timeline<S>>,
}

enum TimelineSlot<S> {
    Unarchived(
        Arc<dyn TimelineState<S>>,
        Vec<(Range<f32>, Arc<dyn Timeline<S>>)>,
    ),
    Archived(Vec<(Range<f32>, Arc<dyn Timeline<S>>)>),
}

pub struct Supervisor<'c, 's, S>
where
    S: Storage,
{
    config: &'c Config,
    storage: &'s S,
    time: RefCell<Arc<f32>>,
    timeline_slots: RefCell<Vec<TimelineSlot<S>>>,
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

    fn collect_timelines(self) -> Vec<(Range<f32>, Arc<dyn Timeline<S>>)> {
        self.timeline_slots
            .into_inner()
            .into_iter()
            .flat_map(|timeline_slot| match timeline_slot {
                TimelineSlot::Unarchived(..) => unreachable!(),
                TimelineSlot::Archived(timelines) => timelines,
            })
            .collect()
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
    ) -> Alive<'sv, 'c, 's, S, CollapsedTimelineState<MB::Instantiation>>
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
            Vec::new(),
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
    TS: Clone + TimelineState<S>,
{
    supervisor: &'sv Supervisor<'c, 's, S>,
    spawn_time: Arc<f32>,
    weak_timeline_state: Weak<TS>,
    index: usize,
}

impl<'c, 's, 'sv, S, TS> Alive<'c, 's, 'sv, S, TS>
where
    S: Storage,
    TS: Clone + TimelineState<S>,
{
    fn new(
        supervisor: &'sv Supervisor<'c, 's, S>,
        spawn_time: Arc<f32>,
        timeline_state: TS,
        children_timelines: Vec<(Range<f32>, Arc<dyn Timeline<S>>)>,
    ) -> Self {
        let timeline_state = Arc::new(timeline_state);
        let weak_timeline_state = Arc::downgrade(&timeline_state);
        let mut timeline_slots = supervisor.timeline_slots.borrow_mut();
        let index = timeline_slots.len();
        timeline_slots.push(TimelineSlot::Unarchived(timeline_state, children_timelines));
        Self {
            supervisor,
            spawn_time,
            weak_timeline_state,
            index,
        }
    }

    fn archive<F, FO>(&mut self, f: F) -> FO
    where
        F: FnOnce(&'sv Supervisor<'c, 's, S>, Arc<f32>, TS) -> FO,
    {
        let supervisor = self.supervisor;
        let timeline_slot = &mut supervisor.timeline_slots.borrow_mut()[self.index];
        let TimelineSlot::Unarchived(_, children_timelines) =
            std::mem::replace(timeline_slot, TimelineSlot::Archived(Vec::new()))
        else {
            unreachable!()
        };
        let archive_time = supervisor.arc_time();
        let spawn_time = std::mem::replace(&mut self.spawn_time, archive_time.clone());
        let timeline_state = Arc::try_unwrap(self.weak_timeline_state.upgrade().unwrap()).unwrap();
        // let timeline = Arc::new(self.timeline.take().unwrap());
        *timeline_slot = TimelineSlot::Archived(if Arc::ptr_eq(&spawn_time, &archive_time) {
            Vec::new()
        } else {
            timeline_state
                .clone()
                .into_timelines(*spawn_time..*archive_time, children_timelines)
        });
        f(supervisor, archive_time.clone(), timeline_state)
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
    TS: Clone + TimelineState<S>,
{
    fn drop(&mut self) {
        if self.weak_timeline_state.strong_count() != 0 {
            self.archive(|_, _, _| ());
        }
    }
}

impl<'sv, 'c, 's, S, M> Quantize for Alive<'sv, 'c, 's, S, CollapsedTimelineState<M>>
where
    S: Storage,
    M: Mobject,
{
    type Output<TM> = Alive<'sv, 'c, 's, S, IndeterminedTimelineState<M, TM, IdentityRate>> where TM: TimeMetric;

    #[must_use]
    fn quantize<TM>(mut self, time_metric: TM) -> Self::Output<TM>
    where
        TM: TimeMetric,
    {
        self.archive(|supervisor, archive_time, timeline_state| {
            Alive::new(
                supervisor,
                archive_time,
                IndeterminedTimelineState {
                    mobject: timeline_state.mobject.clone(),
                    time_metric,
                    rate: IdentityRate,
                },
                Vec::new(),
            )
        })
    }
}

impl<'sv, 'c, 's, S, M, TM, R, U> Collapse
    for Alive<'sv, 'c, 's, S, ActionTimelineState<M, TM, R, U>>
where
    S: Storage,
    M: Mobject,
    TM: TimeMetric,
    R: Rate<TM>,
    U: Update<M, TM>,
{
    type Output = Alive<'sv, 'c, 's, S, CollapsedTimelineState<M>>;

    #[must_use]
    fn collapse(mut self) -> Self::Output {
        let spawn_time = self.spawn_time.clone();
        self.archive(|supervisor, archive_time, timeline_state| {
            let time_interval = *spawn_time..*archive_time;
            let mut mobject = Arc::unwrap_or_clone(timeline_state.mobject);
            timeline_state.update.update(
                &mut mobject,
                timeline_state.time_metric.eval(
                    &timeline_state.rate,
                    time_interval.end - time_interval.start,
                    time_interval,
                ),
            );
            Alive::new(
                supervisor,
                archive_time,
                CollapsedTimelineState {
                    mobject: Arc::new(mobject),
                },
                Vec::new(),
            )
        })
    }
}

impl<'sv, 'c, 's, S, M, TM, R, U> Collapse
    for Alive<'sv, 'c, 's, S, UpdateTimelineState<M, TM, R, U>>
where
    S: Storage,
    M: Mobject,
    TM: TimeMetric,
    R: Rate<TM>,
    U: Update<M, TM>,
{
    type Output = Alive<'sv, 'c, 's, S, CollapsedTimelineState<M>>;

    #[must_use]
    fn collapse(mut self) -> Self::Output {
        let spawn_time = self.spawn_time.clone();
        self.archive(|supervisor, archive_time, timeline_state| {
            let time_interval = *spawn_time..*archive_time;
            let mut mobject = Arc::unwrap_or_clone(timeline_state.mobject);
            timeline_state.update.update(
                &mut mobject,
                timeline_state.time_metric.eval(
                    &timeline_state.rate,
                    time_interval.end - time_interval.start,
                    time_interval,
                ),
            );
            Alive::new(
                supervisor,
                archive_time,
                CollapsedTimelineState {
                    mobject: Arc::new(mobject),
                },
                Vec::new(),
            )
        })
    }
}

impl<'sv, 'c, 's, S, M, R> Collapse for Alive<'sv, 'c, 's, S, ConstructTimelineState<M, R>>
where
    S: Storage,
    M: Mobject,
    R: IncreasingRate<NormalizedTimeMetric>,
{
    type Output = Alive<'sv, 'c, 's, S, CollapsedTimelineState<M>>;

    #[must_use]
    fn collapse(mut self) -> Self::Output {
        self.archive(|supervisor, archive_time, timeline_state| {
            Alive::new(
                supervisor,
                archive_time,
                CollapsedTimelineState {
                    mobject: timeline_state.mobject,
                },
                Vec::new(),
            )
        })
    }
}

impl<'sv, 'c, 's, S, M, TM, R> ApplyRate<R::OutputMetric>
    for Alive<'sv, 'c, 's, S, IndeterminedTimelineState<M, TM, R>>
where
    S: Storage,
    M: Mobject,
    TM: TimeMetric,
    R: Rate<TM>,
{
    type Output<RA> = Alive<'sv, 'c, 's, S, IndeterminedTimelineState<M, TM, ComposeRate<RA, R>>>
    where
        RA: Rate<R::OutputMetric>;

    #[must_use]
    fn apply_rate<RA>(mut self, rate: RA) -> Self::Output<RA>
    where
        RA: Rate<R::OutputMetric>,
    {
        self.archive(|supervisor, archive_time, timeline_state| {
            Alive::new(
                supervisor,
                archive_time,
                IndeterminedTimelineState {
                    mobject: timeline_state.mobject,
                    time_metric: timeline_state.time_metric,
                    rate: ComposeRate(rate, timeline_state.rate),
                },
                Vec::new(),
            )
        })
    }
}

impl<'sv, 'c, 's, S, M> ApplyAct<M, NormalizedTimeMetric>
    for Alive<'sv, 'c, 's, S, CollapsedTimelineState<M>>
where
    S: Storage,
    M: Mobject,
{
    type Output<A> = Alive<'sv, 'c, 's, S, CollapsedTimelineState<M>> where A: Act<M, NormalizedTimeMetric>;

    #[must_use]
    fn apply_act<A>(mut self, act: A) -> Self::Output<A>
    where
        A: Act<M, NormalizedTimeMetric>,
    {
        self.archive(|supervisor, archive_time, timeline_state| {
            let mut mobject = Arc::unwrap_or_clone(timeline_state.mobject);
            act.act(&mobject).update(&mut mobject, 1.0);
            Alive::new(
                supervisor,
                archive_time,
                CollapsedTimelineState {
                    mobject: Arc::new(mobject),
                },
                Vec::new(),
            )
        })
    }
}

impl<'sv, 'c, 's, S, M, TM, R> ApplyAct<M, TM>
    for Alive<'sv, 'c, 's, S, IndeterminedTimelineState<M, TM, R>>
where
    S: Storage,
    M: Mobject,
    TM: TimeMetric,
    R: Rate<TM>,
{
    type Output<A> = Alive<'sv, 'c, 's, S, ActionTimelineState<M, TM, R, A::Update>>
    where
        A: Act<M, TM>;

    #[must_use]
    fn apply_act<A>(mut self, act: A) -> Self::Output<A>
    where
        A: Act<M, TM>,
    {
        self.archive(|supervisor, archive_time, timeline_state| {
            let update = act.act(&timeline_state.mobject);
            Alive::new(
                supervisor,
                archive_time,
                ActionTimelineState {
                    mobject: timeline_state.mobject,
                    time_metric: timeline_state.time_metric,
                    rate: timeline_state.rate,
                    update,
                },
                Vec::new(),
            )
        })
    }
}

impl<'sv, 'c, 's, S, M, TM, R, U> ApplyAct<M, TM>
    for Alive<'sv, 'c, 's, S, ActionTimelineState<M, TM, R, U>>
where
    S: Storage,
    M: Mobject,
    TM: TimeMetric,
    R: Rate<TM>,
    U: Update<M, TM>,
{
    type Output<A> = Alive<'sv, 'c, 's, S, ActionTimelineState<M, TM, R, ComposeUpdate<A::Update, U>>>
    where
        A: Act<M, TM>;

    #[must_use]
    fn apply_act<A>(mut self, act: A) -> Self::Output<A>
    where
        A: Act<M, TM>,
    {
        self.archive(|supervisor, archive_time, timeline_state| {
            let update = ComposeUpdate(act.act(&timeline_state.mobject), timeline_state.update);
            Alive::new(
                supervisor,
                archive_time,
                ActionTimelineState {
                    mobject: timeline_state.mobject,
                    time_metric: timeline_state.time_metric,
                    rate: timeline_state.rate,
                    update,
                },
                Vec::new(),
            )
        })
    }
}

impl<'sv, 'c, 's, S, M, TM, R> ApplyUpdate<M, TM>
    for Alive<'sv, 'c, 's, S, IndeterminedTimelineState<M, TM, R>>
where
    S: Storage,
    M: Mobject,
    TM: TimeMetric,
    R: Rate<TM>,
{
    type Output<U> = Alive<'sv, 'c, 's, S, UpdateTimelineState<M, TM, R, U>>
    where
        U: Update<M, TM>;

    #[must_use]
    fn apply_update<U>(mut self, update: U) -> Self::Output<U>
    where
        U: Update<M, TM>,
    {
        self.archive(|supervisor, archive_time, timeline_state| {
            Alive::new(
                supervisor,
                archive_time,
                UpdateTimelineState {
                    mobject: timeline_state.mobject,
                    time_metric: timeline_state.time_metric,
                    rate: timeline_state.rate,
                    update,
                },
                Vec::new(),
            )
        })
    }
}

impl<'sv, 'c, 's, S, M, R> ApplyConstruct<S, M>
    for Alive<'sv, 'c, 's, S, IndeterminedTimelineState<M, NormalizedTimeMetric, R>>
where
    S: Storage,
    M: Mobject,
    R: IncreasingRate<NormalizedTimeMetric>,
{
    type Output<C> = Alive<'sv, 'c, 's, S, ConstructTimelineState<C::Output, R>>
    where
        C: Construct<S, M>;

    #[must_use]
    fn apply_construct<C>(mut self, construct: C) -> Self::Output<C>
    where
        C: Construct<S, M>,
    {
        self.archive(|supervisor, archive_time, timeline_state| {
            let child_supervisor = Supervisor::new(supervisor.config, supervisor.storage);
            let child_time_start = child_supervisor.time();
            let child_timeline_state = construct
                .construct(
                    Alive::new(
                        &child_supervisor,
                        child_supervisor.arc_time(),
                        CollapsedTimelineState {
                            mobject: timeline_state.mobject,
                        },
                        Vec::new(),
                    ),
                    &child_supervisor,
                )
                .archive(|_, _, child_timeline_state| child_timeline_state);
            Alive::new(
                supervisor,
                archive_time,
                ConstructTimelineState {
                    mobject: child_timeline_state.mobject,
                    time_metric: timeline_state.time_metric,
                    rate: timeline_state.rate,
                    time_interval: child_time_start..child_supervisor.time(),
                },
                child_supervisor.collect_timelines(), // DynamicTimeline {
                                                      //     content: {
                                                      //         let supervisor = Supervisor::new(&supervisor.config);
                                                      //         let mobject = construct
                                                      //             .construct(
                                                      //                 Alive::new(
                                                      //                     &supervisor,
                                                      //                     supervisor.arc_time(),
                                                      //                     SteadyTimeline {
                                                      //                         mobject: dynamic_timeline.content.mobject,
                                                      //                     },
                                                      //                 ),
                                                      //                 &supervisor,
                                                      //             )
                                                      //             .archive(|_, _, steady_timeline| steady_timeline.mobject);
                                                      //         DiscreteTimelineContent {
                                                      //             mobject,
                                                      //             timeline_entries: supervisor.into_timeline_entries(),
                                                      //         }
                                                      //     },
                                                      //     metric: dynamic_timeline.metric,
                                                      //     rate: dynamic_timeline.rate,
                                                      // },
            )
            // Supervisor::visit(
            //     supervisor.config(),
            //     |supervisor| {
            //         construct
            //             .construct(
            //                 Alive::new(
            //                     supervisor,
            //                     supervisor.time(),
            //                     SteadyTimeline {
            //                         mobject: dynamic_timeline.content.mobject,
            //                     },
            //                 ),
            //                 supervisor,
            //             )
            //             .archive(|_, _, steady_timeline| steady_timeline.mobject)
            //     },
            //     |_, timeline_entries, mobject| {
            //         Alive::new(
            //             supervisor,
            //             archive_time,
            //             DynamicTimeline {
            //                 content: DiscreteTimelineContent {
            //                     mobject,
            //                     timeline_entries,
            //                 },
            //                 metric: dynamic_timeline.metric,
            //                 rate: dynamic_timeline.rate,
            //             },
            //         )
            //     },
            // )
            // let child_supervisor = Supervisor::new(supervisor.world());
            // let input_mobject = timeline.content.mobject;
            // let output_mobject = construct
            //     .construct(
            //         Alive::new(
            //             &child_supervisor,
            //             SteadyTimeline {
            //                 mobject: input_mobject,
            //             },
            //         ),
            //         &child_supervisor,
            //     )
            //     .archive(|_, _, steady_timeline| steady_timeline.mobject);
            // Alive::new(
            //     supervisor,
            //     DynamicTimeline {
            //         content: DiscreteTimelineContent {
            //             mobject: output_mobject,
            //             timeline_entries: child_supervisor.into_timeline_entries(),
            //         },
            //         metric: timeline.metric,
            //         rate: timeline.rate,
            //     },
            // )
        })
    }
}

trait QuantizeExt: Quantize {
    fn animate(self) -> Self::Output<NormalizedTimeMetric>;
    fn animating(self) -> Self::Output<DenormalizedTimeMetric>;
}

impl<'sv, 'c, 's, S, TS> QuantizeExt for Alive<'sv, 'c, 's, S, TS>
where
    S: Storage,
    TS: Clone + TimelineState<S>,
    Self: Quantize,
{
    #[must_use]
    fn animate(self) -> Self::Output<NormalizedTimeMetric> {
        self.quantize(NormalizedTimeMetric)
    }

    #[must_use]
    fn animating(self) -> Self::Output<DenormalizedTimeMetric> {
        self.quantize(DenormalizedTimeMetric)
    }
}

trait CollapseExt: Collapse {
    fn play(self, delta_time: f32) -> Self::Output;
}

impl<'sv, 'c, 's, S, TS> CollapseExt for Alive<'sv, 'c, 's, S, TS>
where
    S: Storage,
    TS: Clone + TimelineState<S>,
    Self: Collapse,
{
    #[must_use]
    fn play(self, delta_time: f32) -> Self::Output {
        self.supervisor.wait(delta_time);
        self.collapse()
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct IdentityRate;

impl<TM> Rate<TM> for IdentityRate
where
    TM: TimeMetric,
{
    type OutputMetric = TM;

    fn eval(&self, t: f32) -> f32 {
        t
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ComposeRate<R0, R1>(R0, R1);

impl<R0, R1, TM> Rate<TM> for ComposeRate<R0, R1>
where
    R0: Rate<R1::OutputMetric>,
    R1: Rate<TM>,
    TM: TimeMetric,
{
    type OutputMetric = R0::OutputMetric;

    fn eval(&self, t: f32) -> f32 {
        self.0.eval(self.1.eval(t))
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ComposeUpdate<U0, U1>(U0, U1);

impl<M, U0, U1, TM> Update<M, TM> for ComposeUpdate<U0, U1>
where
    M: Mobject,
    U0: Update<M, TM>,
    U1: Update<M, TM>,
    TM: TimeMetric,
{
    fn update(&self, mobject: &mut M, t: f32) {
        self.1.update(mobject, t);
        self.0.update(mobject, t);
    }

    fn update_presentation(
        &self,
        mobject_presentation: &mut M::MobjectPresentation,
        reference_mobject: &M,
        t: f32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        self.1.update_presentation(
            mobject_presentation,
            reference_mobject,
            t,
            device,
            queue,
            format,
        );
        self.0.update_presentation(
            mobject_presentation,
            reference_mobject,
            t,
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
