use std::ops::Range;
use std::sync::Arc;

use super::super::mobjects::mobject::Mobject;
use super::super::mobjects::mobject::MobjectBuilder;
use super::super::mobjects::mobject::MobjectDiff;
use super::super::toplevel::scene::Supervisor;
use super::act::Act;
use super::act::ApplyAct;
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
use super::timeline::dynamic::RelativeTimelineMetric;
use super::timeline::steady::SteadyTimeline;
use super::timeline::Timeline;
use super::update::ApplyUpdate;
use super::update::Update;

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
    T: Timeline,
{
    pub(crate) fn archive<F, O>(self, f: F) -> O
    where
        F: FnOnce(&T, &'w Supervisor, Range<f32>) -> O,
    {
        let time_interval = self.spawn_time..self.supervisor.get_time();
        let output = f(
            &self.timeline,
            self.supervisor,
            *time_interval.start..*time_interval.end,
        );
        self.supervisor
            .archive_timeline(time_interval, self.timeline);
        output
    }
}

impl Supervisor<'_> {
    pub fn spawn<MB>(&self, mobject_builder: MB) -> Alive<'_, SteadyTimeline<MB::Instantiation>>
    where
        MB: MobjectBuilder,
    {
        self.launch_timeline(SteadyTimeline {
            mobject: mobject_builder.instantiate(self.world()),
        })
    }

    pub(crate) fn launch_timeline<T>(&self, timeline: T) -> Alive<'_, T>
    where
        T: Timeline,
    {
        Alive {
            spawn_time: self.get_time(),
            timeline,
            supervisor: self,
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
    type Output<A> = Self where A: Act<M>;

    fn apply_act<A>(self, act: A) -> Self::Output<A>
    where
        A: Act<M>,
    {
        self.archive(|SteadyTimeline { mobject }, supervisor, _| {
            let mut mobject = mobject.clone();
            act.act(&mobject).apply(&mut mobject, 1.0);
            supervisor.launch_timeline(SteadyTimeline { mobject })
        })
    }
}

pub struct DynamicTimelineBuilder<'w, M, ME, R>
where
    M: Mobject,
{
    steady_mobject: Alive<'w, SteadyTimeline<M>>,
    metric: ME,
    rate: R,
}

impl<'w, M, ME, R> ApplyRate for DynamicTimelineBuilder<'w, M, ME, R>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    type Output<RA> = DynamicTimelineBuilder<'w, M, ME, ComposeRate<RA, R>>
    where
        RA: Rate;

    fn apply_rate<RA>(self, rate: RA) -> Self::Output<RA>
    where
        RA: Rate,
    {
        DynamicTimelineBuilder {
            steady_mobject: self.steady_mobject,
            metric: self.metric,
            rate: ComposeRate(rate, self.rate),
        }
    }
}

impl<'w, M> Alive<'w, SteadyTimeline<M>>
where
    M: Mobject,
{
    pub fn animate(self) -> DynamicTimelineBuilder<'w, M, RelativeTimelineMetric, IdentityRate> {
        DynamicTimelineBuilder {
            steady_mobject: self,
            metric: RelativeTimelineMetric,
            rate: IdentityRate,
        }
    }

    pub fn animating(self) -> DynamicTimelineBuilder<'w, M, AbsoluteTimelineMetric, IdentityRate> {
        DynamicTimelineBuilder {
            steady_mobject: self,
            metric: AbsoluteTimelineMetric,
            rate: IdentityRate,
        }
    }
}

impl<'w, CO, ME, R> Alive<'w, DynamicTimeline<CO, ME, R>>
where
    CO: DynamicTimelineContent,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    pub fn collapse(self) -> Alive<'w, SteadyTimeline<CO::Output>> {
        self.archive(
            |DynamicTimeline {
                 content,
                 metric,
                 rate,
             },
             supervisor,
             time_interval| {
                supervisor.launch_timeline(SteadyTimeline {
                    mobject: content
                        .collapse(rate.eval(metric.eval(time_interval.end, time_interval))),
                })
            },
        )
    }
}

impl<'w, M, ME, R> ApplyAct<M> for DynamicTimelineBuilder<'w, M, ME, R>
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
        self.steady_mobject
            .archive(|SteadyTimeline { mobject }, supervisor, _| {
                let mobject = mobject.clone();
                let diff = act.act(&mobject);
                supervisor.launch_timeline(DynamicTimeline {
                    content: ActionTimelineContent { mobject, diff },
                    metric: self.metric,
                    rate: self.rate,
                })
            })
    }
}

impl<'w, M, ME, R> ApplyUpdate<M> for DynamicTimelineBuilder<'w, M, ME, R>
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
        self.steady_mobject
            .archive(|SteadyTimeline { mobject }, supervisor, _| {
                supervisor.launch_timeline(DynamicTimeline {
                    content: ContinuousTimelineContent {
                        mobject: mobject.clone(),
                        update,
                    },
                    metric: self.metric,
                    rate: self.rate,
                })
            })
    }
}

impl<'w, M, ME, R> ApplyConstruct<M> for DynamicTimelineBuilder<'w, M, ME, R>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    type Output<C> = Alive<'w, DynamicTimeline<DiscreteTimelineContent<M>, ME, R>>
    where
        C: Construct<M>;

    fn apply_construct<C>(self, construct: C) -> Self::Output<C>
    where
        C: Construct<M>,
    {
        self.steady_mobject
            .archive(|SteadyTimeline { mobject }, supervisor, _| {
                let child_supervisor = Supervisor::new(supervisor.world());
                let mobject = construct
                    .construct(
                        child_supervisor.launch_timeline(SteadyTimeline {
                            mobject: mobject.clone(),
                        }),
                        &child_supervisor,
                    )
                    .archive(|steady_timeline, _, _| steady_timeline.mobject.clone());
                supervisor.launch_timeline(DynamicTimeline {
                    content: DiscreteTimelineContent {
                        mobject: mobject.clone(),
                        timeline_collection: child_supervisor.into_collection(),
                    },
                    metric: self.metric,
                    rate: self.rate,
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
        let mobject = self.timeline.content.mobject.clone();
        let diff = ComposeMobjectDiff(act.act(&mobject), self.timeline.content.diff.clone());
        self.supervisor.launch_timeline(DynamicTimeline {
            content: ActionTimelineContent { mobject, diff },
            metric: self.timeline.metric,
            rate: self.timeline.rate,
        })
    }
}

struct IdentityRate;

impl Rate for IdentityRate {
    fn eval(&self, t: f32) -> f32 {
        t
    }
}

struct ComposeRate<R0, R1>(R0, R1);

impl<R0, R1> Rate for ComposeRate<R0, R1>
where
    R0: Rate,
    R1: Rate,
{
    fn eval(&self, t: f32) -> f32 {
        self.0.eval(self.1.eval(t))
    }
}

#[derive(Clone)]
pub struct ComposeMobjectDiff<MD0, MD1>(pub(crate) MD0, pub(crate) MD1);

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
