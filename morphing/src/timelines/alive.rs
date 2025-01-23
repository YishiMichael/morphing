use std::cell::RefCell;
use std::ops::Range;
use std::sync::Arc;

use super::super::mobjects::mobject::Mobject;
use super::super::mobjects::mobject::MobjectBuilder;
use super::super::toplevel::world::World;
use super::act::Act;
use super::act::ApplyAct;
use super::act::MobjectDiff;
use super::construct::ApplyConstruct;
use super::construct::Construct;
use super::rates::ApplyRate;
use super::rates::Rate;
use super::timeline::action::ActionTimelineContent;
use super::timeline::continuous::ContinuousTimelineContent;
use super::timeline::discrete::DiscreteTimelineContent;
use super::timeline::dynamic::AbsoluteTimelineMetric;
use super::timeline::dynamic::DynamicTimeline;
use super::timeline::dynamic::DynamicTimelineContent;
use super::timeline::dynamic::DynamicTimelineMetric;
use super::timeline::dynamic::IndeterminedTimelineContent;
use super::timeline::dynamic::RelativeTimelineMetric;
use super::timeline::steady::SteadyTimeline;
use super::timeline::Timeline;
use super::timeline::TimelineEntries;
use super::update::ApplyUpdate;
use super::update::Update;

pub struct Supervisor<'w> {
    world: &'w World,
    time: RefCell<Arc<f32>>,
    timeline_entries: RefCell<TimelineEntries>,
}

impl<'w> Supervisor<'w> {
    pub(crate) fn new(world: &'w World) -> Self {
        Self {
            world,
            time: RefCell::new(Arc::new(0.0)),
            timeline_entries: RefCell::new(TimelineEntries::new()),
        }
    }

    pub(crate) fn get_time(&self) -> Arc<f32> {
        self.time.borrow().clone()
    }

    pub(crate) fn into_timeline_entries(self) -> TimelineEntries {
        self.timeline_entries.into_inner()
    }

    fn launch_timeline<T>(&self, timeline: T) -> Alive<'_, T>
    where
        T: Timeline,
    {
        Alive {
            spawn_time: self.get_time(),
            timeline,
            supervisor: self,
        }
    }

    fn archive_timeline<T>(&self, time_interval: Range<f32>, timeline: T)
    where
        T: Timeline,
    {
        self.timeline_entries
            .borrow_mut()
            .push(time_interval, timeline);
        // if !Arc::<f32>::ptr_eq(&time_interval.start, &time_interval.end) {
        //     let time_interval = *time_interval.start..*time_interval.end;
        //     self.timeline_entries
        //         .borrow_mut()
        //         .push(time_interval, timeline);
        // }
    }

    pub fn spawn<MB>(&self, mobject_builder: MB) -> Alive<'_, SteadyTimeline<MB::Instantiation>>
    where
        MB: MobjectBuilder,
    {
        self.launch_timeline(SteadyTimeline {
            mobject: mobject_builder.instantiate(self.world),
        })
    }

    pub fn wait(&self, delta_time: f32) {
        assert!(
            delta_time.is_sign_positive(),
            "`Supervisor::wait` expects a non-negative argument `delta_time`, got {delta_time}",
        );
        let mut time = self.time.borrow_mut();
        *time = Arc::new(**time + delta_time);
    }
}

pub struct Alive<'w, T>
where
    T: Timeline,
{
    spawn_time: Arc<f32>,
    timeline: T,
    supervisor: &'w Supervisor<'w>,
}

impl<'w, T> Alive<'w, T>
where
    T: Timeline + Clone,
{
    pub(crate) fn archive<F, O>(self, f: F) -> O
    where
        F: FnOnce(T, &'w Supervisor, Range<f32>) -> O,
    {
        let arc_time_interval = self.spawn_time..self.supervisor.get_time();
        let time_interval = *arc_time_interval.start..*arc_time_interval.end;
        if Arc::<f32>::ptr_eq(&arc_time_interval.start, &arc_time_interval.end) {
            f(self.timeline, self.supervisor, time_interval)
        } else {
            let output = f(
                self.timeline.clone(),
                self.supervisor,
                time_interval.clone(),
            );
            self.supervisor
                .archive_timeline(time_interval, self.timeline);
            output
        }
    }
}

impl<M> Alive<'_, SteadyTimeline<M>>
where
    M: Mobject,
{
    pub fn destroy(self) {
        self.archive(|_, _, _| ())
    }
}

impl<M> ApplyAct<M> for Alive<'_, SteadyTimeline<M>>
where
    M: Mobject,
{
    type Output<A> = Self where A: Act<M>, A::Diff: Clone;

    fn apply_act<A>(self, act: A) -> Self::Output<A>
    where
        A: Act<M>,
        A::Diff: Clone,
    {
        self.archive(|timeline, supervisor, _| {
            let mut mobject = timeline.mobject;
            act.act(&mobject).apply(&mut mobject, 1.0);
            supervisor.launch_timeline(SteadyTimeline { mobject })
        })
    }
}

// pub struct DynamicTimelineBuilder<'w, M, ME, R>
// where
//     M: Mobject,
// {
//     steady_mobject: Alive<'w, SteadyTimeline<M>>,
//     metric: ME,
//     rate: R,
// }

impl<'w, M, ME, R> ApplyRate for Alive<'w, DynamicTimeline<IndeterminedTimelineContent<M>, ME, R>>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    type Output<RA> = Alive<'w, DynamicTimeline<IndeterminedTimelineContent<M>, ME, ComposeRate<RA, R>>>
    where
        RA: Rate;

    fn apply_rate<RA>(self, rate: RA) -> Self::Output<RA>
    where
        RA: Rate,
    {
        self.archive(|timeline, supervisor, _| {
            supervisor.launch_timeline(DynamicTimeline {
                content: IndeterminedTimelineContent {
                    mobject: timeline.content.mobject,
                },
                metric: timeline.metric,
                rate: ComposeRate(rate, timeline.rate),
            })
        })
    }
}

impl<'w, M> Alive<'w, SteadyTimeline<M>>
where
    M: Mobject,
{
    pub fn animate(
        self,
    ) -> Alive<
        'w,
        DynamicTimeline<IndeterminedTimelineContent<M>, RelativeTimelineMetric, IdentityRate>,
    > {
        self.archive(|timeline, supervisor, _| {
            supervisor.launch_timeline(DynamicTimeline {
                content: IndeterminedTimelineContent {
                    mobject: timeline.mobject,
                },
                metric: RelativeTimelineMetric,
                rate: IdentityRate,
            })
        })
    }

    pub fn animating(
        self,
    ) -> Alive<
        'w,
        DynamicTimeline<IndeterminedTimelineContent<M>, AbsoluteTimelineMetric, IdentityRate>,
    > {
        self.archive(|timeline, supervisor, _| {
            supervisor.launch_timeline(DynamicTimeline {
                content: IndeterminedTimelineContent {
                    mobject: timeline.mobject,
                },
                metric: AbsoluteTimelineMetric,
                rate: IdentityRate,
            })
        })
    }
}

impl<'w, CO, ME, R> Alive<'w, DynamicTimeline<CO, ME, R>>
where
    CO: DynamicTimelineContent,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    pub fn collapse(self) -> Alive<'w, SteadyTimeline<CO::Output>> {
        self.archive(|timeline, supervisor, time_interval| {
            supervisor.launch_timeline(SteadyTimeline {
                mobject: timeline.content.collapse(
                    timeline
                        .rate
                        .eval(timeline.metric.eval(time_interval.end, time_interval)),
                ),
            })
        })
    }
}

impl<'w, M, ME, R> ApplyAct<M> for Alive<'w, DynamicTimeline<IndeterminedTimelineContent<M>, ME, R>>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    type Output<A> = Alive<'w, DynamicTimeline<ActionTimelineContent<M, A::Diff>, ME, R>>
    where
        A: Act<M>;

    fn apply_act<A>(self, act: A) -> Self::Output<A>
    where
        A: Act<M>,
    {
        self.archive(|timeline, supervisor, _| {
            let mobject = timeline.content.mobject;
            let diff = act.act(&mobject);
            supervisor.launch_timeline(DynamicTimeline {
                content: ActionTimelineContent { mobject, diff },
                metric: timeline.metric,
                rate: timeline.rate,
            })
        })
    }
}

impl<'w, M, ME, R, MD> ApplyAct<M>
    for Alive<'w, DynamicTimeline<ActionTimelineContent<M, MD>, ME, R>>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
    MD: MobjectDiff<M>,
{
    type Output<A> = Alive<'w, DynamicTimeline<ActionTimelineContent<M, ComposeMobjectDiff<A::Diff, MD>>, ME, R>> where A: Act<M>;

    fn apply_act<A>(self, act: A) -> Self::Output<A>
    where
        A: Act<M>,
    {
        self.archive(|timeline, supervisor, _| {
            let mobject = timeline.content.mobject;
            let diff = ComposeMobjectDiff(act.act(&mobject), timeline.content.diff);
            supervisor.launch_timeline(DynamicTimeline {
                content: ActionTimelineContent { mobject, diff },
                metric: timeline.metric,
                rate: timeline.rate,
            })
        })
    }
}

impl<'w, M, ME, R> ApplyUpdate<M>
    for Alive<'w, DynamicTimeline<IndeterminedTimelineContent<M>, ME, R>>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    type Output<U> = Alive<'w, DynamicTimeline<ContinuousTimelineContent<M, U>, ME, R>>
    where
        U: Update<M>;

    fn apply_update<U>(self, update: U) -> Self::Output<U>
    where
        U: Update<M>,
    {
        self.archive(|timeline, supervisor, _| {
            supervisor.launch_timeline(DynamicTimeline {
                content: ContinuousTimelineContent {
                    mobject: timeline.content.mobject,
                    update,
                },
                metric: timeline.metric,
                rate: timeline.rate,
            })
        })
    }
}

impl<'w, M, ME, R> ApplyConstruct<M>
    for Alive<'w, DynamicTimeline<IndeterminedTimelineContent<M>, ME, R>>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    type Output<C> = Alive<'w, DynamicTimeline<DiscreteTimelineContent<C::Output>, ME, R>>
    where
        C: Construct<M>;

    fn apply_construct<C>(self, construct: C) -> Self::Output<C>
    where
        C: Construct<M>,
    {
        self.archive(|timeline, supervisor, _| {
            let child_supervisor = Supervisor::new(supervisor.world);
            let input_mobject = timeline.content.mobject;
            let output_mobject = construct
                .construct(
                    child_supervisor.launch_timeline(SteadyTimeline {
                        mobject: input_mobject,
                    }),
                    &child_supervisor,
                )
                .archive(|steady_timeline, _, _| steady_timeline.mobject);
            supervisor.launch_timeline(DynamicTimeline {
                content: DiscreteTimelineContent {
                    mobject: output_mobject,
                    timeline_entries: Arc::new(child_supervisor.into_timeline_entries()),
                },
                metric: timeline.metric,
                rate: timeline.rate,
            })
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

    fn apply_realization(
        &self,
        mobject_realization: &mut M::Realization,
        reference_mobject: &M,
        alpha: f32,
        queue: &wgpu::Queue,
    ) {
        self.1
            .apply_realization(mobject_realization, reference_mobject, alpha, queue);
        self.0
            .apply_realization(mobject_realization, reference_mobject, alpha, queue);
    }
}

// pub mod tuple {
// }
