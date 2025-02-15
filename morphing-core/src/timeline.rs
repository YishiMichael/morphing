use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::Range;
use std::sync::Arc;
use std::sync::Weak;

use super::config::Config;
use super::traits::Act;
use super::traits::Construct;
use super::traits::Mobject;
use super::traits::MobjectBuilder;
use super::traits::MobjectDiff;
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

trait TimelineNode<S>:
    'static + Send + Sync + Debug + serde_traitobject::Deserialize + serde_traitobject::Serialize
where
    S: Storage,
{
    type MobjectTimelineEntries: Iterator<Item = MobjectTimelineEntry<S>>;

    fn collect_entries(
        self,
        time_interval: Range<f32>,
        storage: &S,
    ) -> Self::MobjectTimelineEntries;

    // type Descendants: Iterator<Item = Arc<dyn MobjectTimelineEntry<S>>>;
    // type Presentation: Send;

    // fn presentation(&self, device: &wgpu::Device) -> Self::Presentation;
    // fn prepare(
    //     &self,
    //     id: TimelineId,
    //     time_interval: Range<f32>,
    //     time: f32,
    //     // presentation: &mut Self::Presentation,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     storage: &mut iced::widget::shader::Storage,
    //     bounds: &iced::Rectangle,
    //     viewport: &iced::widget::shader::Viewport,
    // );
    // fn render(
    //     &self,
    //     id: TimelineId,
    //     // time_interval: Range<f32>,
    //     // time: f32,
    //     // presentation: &Self::Presentation,
    //     encoder: &mut wgpu::CommandEncoder,
    //     storage: &iced::widget::shader::Storage,
    //     target: &wgpu::TextureView,
    //     clip_bounds: &iced::Rectangle<u32>,
    // );
}

trait TimelineLeaf<S>:
    'static + Send + Sync + Debug + serde_traitobject::Deserialize + serde_traitobject::Serialize
where
    S: Storage,
{
    fn generate_key(&self, storage: &S) -> S::Key;
    fn dyn_prepare(
        &self,
        time: f32,
        storage: &mut S,
        key: &S::Key,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    );
}

trait TimelinePrepare {
    type MobjectPresentation: MobjectPresentation;
    // type Rate: Rate;

    fn prepare(
        &self,
        time: f32,
        presentation: &mut Self::MobjectPresentation,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    );
}

struct MobjectTimeline<T, M> {
    timeline: T,
    mobject: M,
    // rate: R,
    // mobject_storage_key: K,
}

impl<T, S> TimelineNode<S> for T
where
    T: TimelineLeaf<S>,
    S: Storage,
{
    type MobjectTimelineEntries = std::iter::Once<MobjectTimelineEntry<S>>;

    fn collect_entries(
        self,
        time_interval: Range<f32>,
        storage: &S,
    ) -> Self::MobjectTimelineEntries {
        std::iter::once(MobjectTimelineEntry {
            time_interval,
            storage_key: self.generate_key(storage),
            timeline: Arc::new(self),
        })
    }
}

impl<T, M, S> TimelineLeaf<S> for MobjectTimeline<T, M>
where
    T: TimelinePrepare<MobjectPresentation = M::MobjectPresentation>,
    M: Mobject,
    // R: Rate,
    S: Storage,
{
    fn generate_key(&self, storage: &S) -> S::Key {
        storage.generate_key(&self.mobject) // TODO: serde_traitobject::Arc? timeline?
    }

    fn dyn_prepare(
        &self,
        time: f32,
        storage: &mut S,
        key: &S::Key,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        let presentation = storage.get_mut_or_insert(key, || self.mobject.presentation(device));
        self.timeline
            .prepare(time, presentation, device, queue, format);
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct MobjectTimelineEntry<S>
where
    S: Storage,
{
    time_interval: Range<f32>,
    storage_key: S::Key,
    timeline: Arc<dyn TimelineLeaf<S>>,
}

struct CompositeTimeline

impl<I, S> TimelineNode<S> for I
where
    I: Iterator<Item = MobjectTimelineEntry<S>>,
{
    type MobjectTimelineEntries = Self;

    fn collect_entries(
        self,
        time_interval: Range<f32>,
        storage: &S,
    ) -> Self::MobjectTimelineEntries {
    }
}

// trait DynTimeline:
//     Send + Sync + Debug + serde_traitobject::Deserialize + serde_traitobject::Serialize
// {
//     fn dyn_prepare(
//         &self,
//         hash: u64,
//         time_interval: Range<f32>,
//         time: f32,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         storage: &mut iced::widget::shader::Storage,
//         bounds: &iced::Rectangle,
//         viewport: &iced::widget::shader::Viewport,
//     );
//     fn dyn_render(
//         &self,
//         hash: u64,
//         time_interval: Range<f32>,
//         time: f32,
//         encoder: &mut wgpu::CommandEncoder,
//         storage: &iced::widget::shader::Storage,
//         target: &wgpu::TextureView,
//         clip_bounds: &iced::Rectangle<u32>,
//     );
// }

// impl<T> DynTimeline for T
// where
//     T: Timeline,
// {
//     fn dyn_prepare(
//         &self,
//         hash: u64,
//         time_interval: Range<f32>,
//         time: f32,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//         storage: &mut iced::widget::shader::Storage,
//         bounds: &iced::Rectangle,
//         viewport: &iced::widget::shader::Viewport,
//     ) {

//         self.prepare(
//             time_interval,
//             time,
//             &mut presentation,
//             device,
//             queue,
//             format,
//             storage,
//             bounds,
//             viewport,
//         );
//     }

//     fn dyn_render(
//         &self,
//         hash: u64,
//         time_interval: Range<f32>,
//         time: f32,
//         encoder: &mut wgpu::CommandEncoder,
//         storage: &iced::widget::shader::Storage,
//         target: &wgpu::TextureView,
//         clip_bounds: &iced::Rectangle<u32>,
//     ) {
//         let presentation_map = storage
//             .get::<dashmap::DashMap<u64, T::Presentation>>()
//             .unwrap();
//         let presentation = presentation_map.get(&hash).unwrap();
//         self.render(
//             time_interval,
//             time,
//             &presentation,
//             encoder,
//             storage,
//             target,
//             clip_bounds,
//         );
//     }
// }

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct TimelineEntry {
    id: TimelineId,
    time_interval: Range<f32>,
    timeline: serde_traitobject::Arc<dyn Timeline>,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct TimelineEntries(Arc<Vec<TimelineEntry>>);

impl TimelineEntries {
    fn prepare(
        &self,
        time: f32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut iced::widget::shader::Storage,
        bounds: &iced::Rectangle,
        viewport: &iced::widget::shader::Viewport,
    ) {
        for timeline_entry in &*self.0 {
            if timeline_entry.time_interval.contains(&time) {
                timeline_entry.timeline.prepare(
                    timeline_entry.id,
                    timeline_entry.time_interval.clone(),
                    time,
                    device,
                    queue,
                    format,
                    storage,
                    bounds,
                    viewport,
                );
            }
        }
    }

    fn render(
        &self,
        time: f32,
        encoder: &mut wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        for timeline_entry in &*self.0 {
            if timeline_entry.time_interval.contains(&time) {
                timeline_entry.timeline.render(
                    timeline_entry.id,
                    // timeline_entry.time_interval.clone(),
                    // time,
                    encoder,
                    storage,
                    target,
                    clip_bounds,
                );
            }
        }
    }
}

fn new_render_pass<'ce>(
    encoder: &'ce mut wgpu::CommandEncoder,
    target: &'ce wgpu::TextureView,
    // clip_bounds: &iced::Rectangle<u32>,
    load: wgpu::LoadOp<wgpu::Color>,
) -> wgpu::RenderPass<'ce> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target,
            resolve_target: None,
            ops: wgpu::Operations {
                load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    })
}

#[derive(Debug)]
pub struct ScenePrimitive {
    time: f32,
    timeline_entries: TimelineEntries,
    background_color: wgpu::Color,
}

impl iced::widget::shader::Primitive for ScenePrimitive {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut iced::widget::shader::Storage,
        bounds: &iced::Rectangle,
        viewport: &iced::widget::shader::Viewport,
    ) {
        // TODO: clean up storage for older scenes
        self.timeline_entries
            .prepare(self.time, device, queue, format, storage, bounds, viewport);
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        new_render_pass(
            encoder,
            target,
            clip_bounds,
            wgpu::LoadOp::Clear(self.background_color),
        );
        self.timeline_entries
            .render(self.time, encoder, storage, target, clip_bounds);
    }
}

pub struct Supervisor<'c> {
    config: &'c Config,
    time: RefCell<Arc<f32>>,
    timeline_items: RefCell<Vec<(Option<Range<f32>>, Option<Arc<dyn Timeline>>)>>,
}

impl<'c> Supervisor<'c> {
    pub(crate) fn new(config: &'c Config) -> Self {
        Self {
            config,
            time: RefCell::new(Arc::new(0.0)),
            timeline_items: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn into_timeline_entries(self) -> TimelineEntries {
        TimelineEntries(Arc::new(
            self.timeline_items
                .into_inner()
                .into_iter()
                .filter_map(|(time_interval, timeline)| {
                    time_interval
                        .zip(timeline)
                        .map(|(time_interval, timeline)| {
                            let timeline = serde_traitobject::Arc::from(timeline);
                            TimelineEntry {
                                id: TimelineId::new(&timeline),
                                time_interval,
                                timeline,
                            }
                        })
                })
                .collect(),
        ))
    }

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
    pub fn spawn<MB>(&'c self, mobject_builder: MB) -> Alive<'c, SteadyTimeline<MB::Instantiation>>
    where
        MB: MobjectBuilder,
    {
        Alive::new(
            self,
            self.arc_time(),
            SteadyTimeline {
                mobject: mobject_builder.instantiate(&self.config),
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

pub struct Alive<'s, T>
where
    T: Clone + Timeline,
{
    supervisor: &'s Supervisor<'s>,
    spawn_time: Arc<f32>,
    weak_timeline: Weak<T>,
    index: usize,
}

impl<'s, T> Alive<'s, T>
where
    T: Clone + Timeline,
{
    fn new(supervisor: &'s Supervisor<'s>, spawn_time: Arc<f32>, timeline: T) -> Self {
        let timeline = Arc::new(timeline);
        let weak_timeline = Arc::downgrade(&timeline);
        let mut timeline_items = supervisor.timeline_items.borrow_mut();
        let index = timeline_items.len();
        timeline_items.push((None, Some(timeline)));
        Self {
            supervisor,
            spawn_time,
            weak_timeline,
            index,
        }
    }

    fn archive<F, FO>(&mut self, f: F) -> FO
    where
        F: FnOnce(&'s Supervisor<'s>, Arc<f32>, T) -> FO,
    {
        let supervisor = self.supervisor;
        let (time_interval, timeline) = &mut supervisor.timeline_items.borrow_mut()[self.index];
        let archive_time = supervisor.arc_time();
        let spawn_time = std::mem::replace(&mut self.spawn_time, archive_time.clone());
        // let timeline = Arc::new(self.timeline.take().unwrap());
        if Arc::ptr_eq(&spawn_time, &archive_time) {
            let _ = timeline.take();
            // let index = timeline_entries
            //     .iter()
            //     .rposition(|timeline_entry| {
            //         std::ptr::eq(
            //             &*timeline as *const T as *const (),
            //             &timeline_entry.timeline.into() as *const dyn DynTimeline as *const (),
            //         )
            //     })
            //     .unwrap();
            // timeline_entries.remove(index);
            // supervisor
            //     .push(*self.spawn_time..*archive_time, timeline.clone());
        } else {
            let _ = time_interval.insert(*spawn_time..*archive_time);
        }
        f(
            supervisor,
            archive_time,
            Arc::unwrap_or_clone(self.weak_timeline.upgrade().unwrap()),
        )
    }

    // fn archive(mut self) -> Self {
    //     self.archive();
    //     self
    // }

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

impl<T> Drop for Alive<'_, T>
where
    T: Clone + Timeline,
{
    fn drop(&mut self) {
        if self.weak_timeline.strong_count() != 0 {
            self.archive(|_, _, _| ());
        }
    }
}

impl<'s, M> Alive<'s, SteadyTimeline<M>>
where
    M: Mobject,
{
    #[must_use]
    pub fn quantize<ME>(
        mut self,
        metric: ME,
    ) -> Alive<'s, DynamicTimeline<IndeterminedTimelineContent<M>, ME, IdentityRate>>
    where
        ME: DynamicTimelineMetric,
    {
        self.archive(|supervisor, archive_time, timeline| {
            Alive::new(
                supervisor,
                archive_time,
                DynamicTimeline {
                    content: IndeterminedTimelineContent {
                        mobject: timeline.mobject,
                    },
                    metric,
                    rate: IdentityRate,
                },
            )
        })
    }

    #[must_use]
    pub fn animate(
        self,
    ) -> Alive<
        's,
        DynamicTimeline<IndeterminedTimelineContent<M>, RelativeTimelineMetric, IdentityRate>,
    > {
        self.quantize(RelativeTimelineMetric)
    }

    #[must_use]
    pub fn animating(
        self,
    ) -> Alive<
        's,
        DynamicTimeline<IndeterminedTimelineContent<M>, AbsoluteTimelineMetric, IdentityRate>,
    > {
        self.quantize(AbsoluteTimelineMetric)
    }
}

impl<'s, CO, ME, R> Alive<'s, DynamicTimeline<CO, ME, R>>
where
    CO: DynamicTimelineContent,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    #[must_use]
    pub fn collapse(mut self) -> Alive<'s, SteadyTimeline<CO::CollapseOutput>> {
        let spawn_time = self.spawn_time.clone();
        self.archive(|supervisor, archive_time, timeline| {
            let time_interval = *spawn_time..*archive_time;
            Alive::new(
                supervisor,
                archive_time,
                SteadyTimeline {
                    mobject: timeline.content.content_collapse(
                        timeline
                            .rate
                            .eval(timeline.metric.eval(time_interval.end, time_interval)),
                    ),
                },
            )
        })
    }

    #[must_use]
    pub fn play(self, delta_time: f32) -> Alive<'s, SteadyTimeline<CO::CollapseOutput>> {
        self.supervisor.wait(delta_time);
        self.collapse()
    }
}

impl<'s, M, ME, R> ApplyRate for Alive<'s, DynamicTimeline<IndeterminedTimelineContent<M>, ME, R>>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    type Output<RA> = Alive<'s, DynamicTimeline<IndeterminedTimelineContent<M>, ME, ComposeRate<RA, R>>>
    where
        RA: Rate;

    #[must_use]
    fn apply_rate<RA>(mut self, rate: RA) -> Self::Output<RA>
    where
        RA: Rate,
    {
        self.archive(|supervisor, archive_time, timeline| {
            Alive::new(
                supervisor,
                archive_time,
                DynamicTimeline {
                    content: IndeterminedTimelineContent {
                        mobject: timeline.content.mobject,
                    },
                    metric: timeline.metric,
                    rate: ComposeRate(rate, timeline.rate),
                },
            )
        })
    }
}

impl<M> ApplyAct<M> for Alive<'_, SteadyTimeline<M>>
where
    M: Mobject,
{
    type Output<A> = Self where A: Act<M>;

    #[must_use]
    fn apply_act<A>(mut self, act: A) -> Self::Output<A>
    where
        A: Act<M>,
    {
        self.archive(|supervisor, archive_time, timeline| {
            let mut mobject = timeline.mobject;
            act.act(&mobject).apply(&mut mobject, 1.0);
            Alive::new(supervisor, archive_time, SteadyTimeline { mobject })
        })
    }
}

impl<'s, M, ME, R> ApplyAct<M> for Alive<'s, DynamicTimeline<IndeterminedTimelineContent<M>, ME, R>>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    type Output<A> = Alive<'s, DynamicTimeline<ActionTimelineContent<M, A::Diff>, ME, R>>
    where
        A: Act<M>;

    #[must_use]
    fn apply_act<A>(mut self, act: A) -> Self::Output<A>
    where
        A: Act<M>,
    {
        self.archive(|supervisor, archive_time, timeline| {
            let mobject = timeline.content.mobject;
            let diff = act.act(&mobject);
            Alive::new(
                supervisor,
                archive_time,
                DynamicTimeline {
                    content: ActionTimelineContent { mobject, diff },
                    metric: timeline.metric,
                    rate: timeline.rate,
                },
            )
        })
    }
}

impl<'s, M, ME, R, MD> ApplyAct<M>
    for Alive<'s, DynamicTimeline<ActionTimelineContent<M, MD>, ME, R>>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
    MD: MobjectDiff<M>,
{
    type Output<A> = Alive<'s, DynamicTimeline<ActionTimelineContent<M, ComposeMobjectDiff<A::Diff, MD>>, ME, R>>
    where
        A: Act<M>;

    #[must_use]
    fn apply_act<A>(mut self, act: A) -> Self::Output<A>
    where
        A: Act<M>,
    {
        self.archive(|supervisor, archive_time, timeline| {
            let mobject = timeline.content.mobject;
            let diff = ComposeMobjectDiff(act.act(&mobject), timeline.content.diff);
            Alive::new(
                supervisor,
                archive_time,
                DynamicTimeline {
                    content: ActionTimelineContent { mobject, diff },
                    metric: timeline.metric,
                    rate: timeline.rate,
                },
            )
        })
    }
}

impl<'s, M, ME, R> ApplyUpdate<M>
    for Alive<'s, DynamicTimeline<IndeterminedTimelineContent<M>, ME, R>>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    type Output<U> = Alive<'s, DynamicTimeline<ContinuousTimelineContent<M, U>, ME, R>>
    where
        U: Update<M>;

    #[must_use]
    fn apply_update<U>(mut self, update: U) -> Self::Output<U>
    where
        U: Update<M>,
    {
        self.archive(|supervisor, archive_time, timeline| {
            Alive::new(
                supervisor,
                archive_time,
                DynamicTimeline {
                    content: ContinuousTimelineContent {
                        mobject: timeline.content.mobject,
                        update,
                    },
                    metric: timeline.metric,
                    rate: timeline.rate,
                },
            )
        })
    }
}

impl<'s, M, ME, R> ApplyConstruct<M>
    for Alive<'s, DynamicTimeline<IndeterminedTimelineContent<M>, ME, R>>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    type Output<C> = Alive<'s, DynamicTimeline<DiscreteTimelineContent<C::Output>, ME, R>>
    where
        C: Construct<M>;

    #[must_use]
    fn apply_construct<C>(mut self, construct: C) -> Self::Output<C>
    where
        C: Construct<M>,
    {
        self.archive(|supervisor, archive_time, dynamic_timeline| {
            Alive::new(
                supervisor,
                archive_time,
                DynamicTimeline {
                    content: {
                        let supervisor = Supervisor::new(&supervisor.config);
                        let mobject = construct
                            .construct(
                                Alive::new(
                                    &supervisor,
                                    supervisor.arc_time(),
                                    SteadyTimeline {
                                        mobject: dynamic_timeline.content.mobject,
                                    },
                                ),
                                &supervisor,
                            )
                            .archive(|_, _, steady_timeline| steady_timeline.mobject);
                        DiscreteTimelineContent {
                            mobject,
                            timeline_entries: supervisor.into_timeline_entries(),
                        }
                    },
                    metric: dynamic_timeline.metric,
                    rate: dynamic_timeline.rate,
                },
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

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct IdentityRate;

impl Rate for IdentityRate {
    fn eval(&self, t: f32) -> f32 {
        t
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ComposeRate<R0, R1>(R0, R1);

impl<R0, R1> Rate for ComposeRate<R0, R1>
where
    R0: Rate,
    R1: Rate,
{
    fn eval(&self, t: f32) -> f32 {
        self.0.eval(self.1.eval(t))
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ComposeMobjectDiff<MD0, MD1>(MD0, MD1);

impl<M, MD0, MD1> MobjectDiff<M> for ComposeMobjectDiff<MD0, MD1>
where
    M: Mobject,
    MD0: MobjectDiff<M>,
    MD1: MobjectDiff<M>,
{
    fn apply(&self, mobject: &mut M, alpha: f32) {
        self.1.apply(mobject, alpha);
        self.0.apply(mobject, alpha);
    }

    fn apply_presentation(
        &self,
        mobject_presentation: &mut M::MobjectPresentation,
        reference_mobject: &M,
        alpha: f32,
        queue: &wgpu::Queue,
    ) {
        self.1
            .apply_presentation(mobject_presentation, reference_mobject, alpha, queue);
        self.0
            .apply_presentation(mobject_presentation, reference_mobject, alpha, queue);
    }
}

// steady

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct SteadyTimeline<M> {
    mobject: M,
}

impl<M> Timeline for SteadyTimeline<M>
where
    M: Mobject,
{
    fn prepare(
        &self,
        id: TimelineId,
        _time_interval: Range<f32>,
        _time: f32,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        storage: &mut iced::widget::shader::Storage,
        _bounds: &iced::Rectangle,
        _viewport: &iced::widget::shader::Viewport,
    ) {
        id.prepare_presentation(storage, device, &self.mobject, |_| ());
    }

    fn render(
        &self,
        id: TimelineId,
        _time_interval: Range<f32>,
        _time: f32,
        encoder: &mut wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        id.render_presentation::<M, _>(storage, |presentation| {
            let mut render_pass = new_render_pass(encoder, target, clip_bounds, wgpu::LoadOp::Load);
            presentation.draw(&mut render_pass);
        });
    }

    // type Presentation = M::MobjectPresentation;

    // fn presentation(&self, device: &wgpu::Device) -> Self::Presentation {
    //     self.mobject.presentation(device)
    // }

    // fn prepare(
    //     &self,
    //     _time_interval: Range<f32>,
    //     _time: f32,
    //     _device: &wgpu::Device,
    //     _queue: &wgpu::Queue,
    //     _format: wgpu::TextureFormat,
    //     _presentation: &mut Self::Presentation,
    //     _bounds: &iced::Rectangle,
    //     _viewport: &iced::widget::shader::Viewport,
    // ) {
    // }

    // fn render(
    //     &self,
    //     _time_interval: Range<f32>,
    //     _time: f32,
    //     encoder: &mut wgpu::CommandEncoder,
    //     presentation: &Self::Presentation,
    //     target: &wgpu::TextureView,
    //     clip_bounds: &iced::Rectangle<u32>,
    // ) {
    //     let mut render_pass = new_render_pass(
    //         encoder,
    //         target,
    //         clip_bounds,
    //         wgpu::LoadOp::Load,
    //     );
    //     presentation.draw(&mut render_pass);
    // }
}

// dynamic

pub trait ApplyRate: Sized {
    type Output<R>
    where
        R: Rate;

    fn apply_rate<R>(self, rate: R) -> Self::Output<R>
    where
        R: Rate;
}

pub trait DynamicTimelineContent:
    'static + Clone + Send + Sync + Debug + serde::de::DeserializeOwned + serde::Serialize
{
    // type ContentPresentation: Send;
    type CollapseOutput: Mobject;

    // fn content_presentation(
    //     &self,
    //     device: &wgpu::Device,
    // ) -> Self::ContentPresentation;
    fn content_prepare(
        &self,
        id: TimelineId,
        time: f32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut iced::widget::shader::Storage,
        bounds: &iced::Rectangle,
        viewport: &iced::widget::shader::Viewport,
    );
    fn content_render(
        &self,
        id: TimelineId,
        time: f32,
        encoder: &mut wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    );
    fn content_collapse(self, time: f32) -> Self::CollapseOutput;
}

pub trait DynamicTimelineMetric:
    'static + Clone + Send + Sync + Debug + serde::de::DeserializeOwned + serde::Serialize
{
    fn eval(&self, time: f32, time_interval: Range<f32>) -> f32;
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct RelativeTimelineMetric;

impl DynamicTimelineMetric for RelativeTimelineMetric {
    fn eval(&self, time: f32, time_interval: Range<f32>) -> f32 {
        (time - time_interval.start) / (time_interval.end - time_interval.start)
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AbsoluteTimelineMetric;

impl DynamicTimelineMetric for AbsoluteTimelineMetric {
    fn eval(&self, time: f32, time_interval: Range<f32>) -> f32 {
        time - time_interval.start
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DynamicTimeline<CO, ME, R> {
    content: CO,
    metric: ME,
    rate: R,
}

impl<CO, ME, R> Timeline for DynamicTimeline<CO, ME, R>
where
    CO: DynamicTimelineContent,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    // type Presentation = CO::ContentPresentation;

    // fn presentation(&self, device: &wgpu::Device) -> Self::Presentation {
    //     self.content.content_presentation(device)
    // }

    fn prepare(
        &self,
        id: TimelineId,
        time_interval: Range<f32>,
        time: f32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut iced::widget::shader::Storage,
        bounds: &iced::Rectangle,
        viewport: &iced::widget::shader::Viewport,
    ) {
        self.content.content_prepare(
            id,
            self.rate.eval(self.metric.eval(time, time_interval)),
            device,
            queue,
            format,
            storage,
            bounds,
            viewport,
        );
    }

    fn render(
        &self,
        id: TimelineId,
        time_interval: Range<f32>,
        time: f32,
        encoder: &mut wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        self.content.content_render(
            id,
            self.rate.eval(self.metric.eval(time, time_interval)),
            encoder,
            storage,
            target,
            clip_bounds,
        );
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct IndeterminedTimelineContent<M> {
    mobject: M,
}

impl<M> DynamicTimelineContent for IndeterminedTimelineContent<M>
where
    M: Mobject,
{
    // type ContentPresentation = M::MobjectPresentation;
    type CollapseOutput = M;

    fn content_prepare(
        &self,
        id: TimelineId,
        _time: f32,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        storage: &mut iced::widget::shader::Storage,
        _bounds: &iced::Rectangle,
        _viewport: &iced::widget::shader::Viewport,
    ) {
        id.prepare_presentation(storage, device, &self.mobject, |_| ());
    }

    fn content_render(
        &self,
        id: TimelineId,
        _time: f32,
        encoder: &mut wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        id.render_presentation::<M, _>(storage, |presentation| {
            let mut render_pass = new_render_pass(encoder, target, clip_bounds, wgpu::LoadOp::Load);
            presentation.draw(&mut render_pass)
        });
    }

    // fn content_presentation(
    //     &self,
    //     device: &wgpu::Device,
    // ) -> Self::ContentPresentation {
    //     self.mobject.presentation(device)
    // }

    // fn content_prepare(
    //     &self,
    //     _time: f32,
    //     _device: &wgpu::Device,
    //     _queue: &wgpu::Queue,
    //     _format: wgpu::TextureFormat,
    //     _presentation: &mut Self::ContentPresentation,
    //     _bounds: &iced::Rectangle,
    //     _viewport: &iced::widget::shader::Viewport,
    // ) {
    // }

    // fn content_render(
    //     &self,
    //     _time: f32,
    //     encoder: &mut wgpu::CommandEncoder,
    //     presentation: &Self::ContentPresentation,
    //     target: &wgpu::TextureView,
    //     clip_bounds: &iced::Rectangle<u32>,
    // ) {
    //     let mut render_pass = new_render_pass(
    //         encoder,
    //         target,
    //         clip_bounds,
    //         wgpu::LoadOp::Load,
    //     );
    //     presentation.draw(&mut render_pass);
    // }

    fn content_collapse(self, _time: f32) -> Self::CollapseOutput {
        self.mobject
    }
}

// action

pub trait ApplyAct<M>: Sized
where
    M: Mobject,
{
    type Output<A>
    where
        A: Act<M>;

    fn apply_act<A>(self, act: A) -> Self::Output<A>
    where
        A: Act<M>;
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ActionTimelineContent<M, D> {
    mobject: M,
    diff: D,
}

impl<M, MD> DynamicTimelineContent for ActionTimelineContent<M, MD>
where
    M: Mobject,
    MD: MobjectDiff<M>,
{
    // type ContentPresentation = M::MobjectPresentation;
    type CollapseOutput = M;

    fn content_prepare(
        &self,
        id: TimelineId,
        time: f32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        storage: &mut iced::widget::shader::Storage,
        _bounds: &iced::Rectangle,
        _viewport: &iced::widget::shader::Viewport,
    ) {
        id.prepare_presentation(storage, device, &self.mobject, |presentation| {
            self.diff
                .apply_presentation(presentation, &self.mobject, time, queue);
        });
    }

    fn content_render(
        &self,
        id: TimelineId,
        _time: f32,
        encoder: &mut wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        id.render_presentation::<M, _>(storage, |presentation| {
            let mut render_pass = new_render_pass(encoder, target, clip_bounds, wgpu::LoadOp::Load);
            presentation.draw(&mut render_pass)
        });
    }

    // fn content_presentation(
    //     &self,
    //     device: &wgpu::Device,
    // ) -> Self::ContentPresentation {
    //     self.mobject.presentation(device)
    // }

    // fn content_prepare(
    //     &self,
    //     time: f32,
    //     _device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     _format: wgpu::TextureFormat,
    //     presentation: &mut Self::ContentPresentation,
    //     _bounds: &iced::Rectangle,
    //     _viewport: &iced::widget::shader::Viewport,
    // ) {
    //     self.diff
    //         .apply_presentation(presentation, &self.mobject, time, queue);
    // }

    // fn content_render(
    //     &self,
    //     _time: f32,
    //     encoder: &mut wgpu::CommandEncoder,
    //     presentation: &Self::ContentPresentation,
    //     target: &wgpu::TextureView,
    //     clip_bounds: &iced::Rectangle<u32>,
    // ) {
    //     let mut render_pass = new_render_pass(
    //         encoder,
    //         target,
    //         clip_bounds,
    //         wgpu::LoadOp::Load,
    //     );
    //     presentation.draw(&mut render_pass);
    // }

    fn content_collapse(self, time: f32) -> Self::CollapseOutput {
        let mut mobject = self.mobject;
        self.diff.apply(&mut mobject, time);
        mobject
    }
}

// continuous

pub trait ApplyUpdate<M>: Sized
where
    M: Mobject,
{
    type Output<U>
    where
        U: Update<M>;

    fn apply_update<U>(self, update: U) -> Self::Output<U>
    where
        U: Update<M>;
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ContinuousTimelineContent<M, U> {
    mobject: M,
    update: U,
}

impl<M, U> DynamicTimelineContent for ContinuousTimelineContent<M, U>
where
    M: Mobject,
    U: Update<M>,
{
    // type ContentPresentation = M::MobjectPresentation;
    type CollapseOutput = M;

    fn content_prepare(
        &self,
        id: TimelineId,
        time: f32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        storage: &mut iced::widget::shader::Storage,
        _bounds: &iced::Rectangle,
        _viewport: &iced::widget::shader::Viewport,
    ) {
        id.prepare_presentation(storage, device, &self.mobject, |presentation| {
            self.update
                .update_presentation(presentation, &self.mobject, time, device, queue);
        });
    }

    fn content_render(
        &self,
        id: TimelineId,
        _time: f32,
        encoder: &mut wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        id.render_presentation::<M, _>(storage, |presentation| {
            let mut render_pass = new_render_pass(encoder, target, clip_bounds, wgpu::LoadOp::Load);
            presentation.draw(&mut render_pass);
        });
    }

    // fn content_presentation(
    //     &self,
    //     device: &wgpu::Device,
    // ) -> Self::ContentPresentation {
    //     self.mobject.presentation(device)
    // }

    // fn content_prepare(
    //     &self,
    //     time: f32,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     _format: wgpu::TextureFormat,
    //     presentation: &mut Self::ContentPresentation,
    //     _bounds: &iced::Rectangle,
    //     _viewport: &iced::widget::shader::Viewport,
    // ) {
    //     self.update
    //         .update_presentation(presentation, &self.mobject, time, device, queue);
    // }

    // fn content_render(
    //     &self,
    //     _time: f32,
    //     encoder: &mut wgpu::CommandEncoder,
    //     presentation: &Self::ContentPresentation,
    //     target: &wgpu::TextureView,
    //     clip_bounds: &iced::Rectangle<u32>,
    // ) {
    //     let mut render_pass = new_render_pass(
    //         encoder,
    //         target,
    //         clip_bounds,
    //         wgpu::LoadOp::Load,
    //     );
    //     presentation.draw(&mut render_pass);
    // }

    fn content_collapse(self, time: f32) -> Self::CollapseOutput {
        let mut mobject = self.mobject;
        self.update.update(&mut mobject, time);
        mobject
    }
}

// discrete

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

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DiscreteTimelineContent<M> {
    mobject: M,
    timeline_entries: TimelineEntries,
}

impl<M> DynamicTimelineContent for DiscreteTimelineContent<M>
where
    M: Mobject,
{
    // type ContentPresentation = iced::widget::shader::Storage;
    type CollapseOutput = M;

    fn content_prepare(
        &self,
        _id: TimelineId,
        time: f32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut iced::widget::shader::Storage,
        bounds: &iced::Rectangle,
        viewport: &iced::widget::shader::Viewport,
    ) {
        self.timeline_entries
            .prepare(time, device, queue, format, storage, bounds, viewport);
    }

    fn content_render(
        &self,
        _id: TimelineId,
        time: f32,
        encoder: &mut wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        self.timeline_entries
            .render(time, encoder, storage, target, clip_bounds);
    }

    // fn content_presentation(
    //     &self,
    //     _device: &wgpu::Device,
    // ) -> Self::ContentPresentation {
    //     iced::widget::shader::Storage::default()
    // }

    // fn content_prepare(
    //     &self,
    //     time: f32,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    //     presentation: &mut Self::ContentPresentation,
    //     bounds: &iced::Rectangle,
    //     viewport: &iced::widget::shader::Viewport,
    // ) {
    //     self.timeline_entries
    //         .prepare(time, device, queue, format, presentation, bounds, viewport);
    // }

    // fn content_render(
    //     &self,
    //     time: f32,
    //     encoder: &mut wgpu::CommandEncoder,
    //     presentation: &Self::ContentPresentation,
    //     target: &wgpu::TextureView,
    //     clip_bounds: &iced::Rectangle<u32>,
    // ) {
    //     self.timeline_entries
    //         .render(time, encoder, presentation, target, clip_bounds);
    // }

    fn content_collapse(self, _time: f32) -> Self::CollapseOutput {
        self.mobject
    }
}
