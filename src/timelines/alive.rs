use std::ops::Range;
use std::sync::Arc;

use super::super::mobjects::mobject::Mobject;
use super::super::mobjects::mobject::MobjectBuilder;
use super::super::toplevel::scene::Supervisor;
use super::act::Act;
use super::act::ApplyAct;
use super::act::ComposeDiff;
use super::act::Diff;
use super::construct::ApplyConstruct;
use super::construct::Construct;
use super::rates::ApplyRate;
use super::rates::ComposeRate;
use super::rates::IdentityRate;
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

pub struct Alive<'a, T>
where
    T: Timeline,
{
    spawn_time: Arc<f32>,
    timeline: T,
    supervisor: &'a Supervisor,
}

impl<'a, T> Alive<'a, T>
where
    T: Timeline,
{
    pub(crate) fn archive<F, O>(self, f: F) -> O
    where
        F: FnOnce(&T, &'a Supervisor, Range<f32>) -> O,
    {
        let time_interval = self.spawn_time..self.supervisor.get_time();
        let output = f(
            &self.timeline,
            self.supervisor,
            *time_interval.start..*time_interval.end,
        );
        self.supervisor
            .archive_presentation(time_interval, self.timeline.presentation());
        output
    }
}

impl Supervisor {
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

impl<'a, M> ApplyAct<M> for Alive<'_, SteadyTimeline<M>>
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
            act.act(&mobject).apply(&mut mobject);
            supervisor.launch_timeline(SteadyTimeline { mobject })
        })
    }
}

pub struct DynamicTimelineBuilder<'a, M, ME, R>
where
    M: Mobject,
{
    steady_mobject: Alive<'a, SteadyTimeline<M>>,
    metric: ME,
    rate: R,
}

impl<'a, M, ME, R> ApplyRate for DynamicTimelineBuilder<'a, M, ME, R>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    type Output<RA> = DynamicTimelineBuilder<'a, M, ME, ComposeRate<RA, R>>
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

impl<'a, M> Alive<'a, SteadyTimeline<M>>
where
    M: Mobject,
{
    pub fn animate(self) -> DynamicTimelineBuilder<'a, M, RelativeTimelineMetric, IdentityRate> {
        DynamicTimelineBuilder {
            steady_mobject: self,
            metric: RelativeTimelineMetric,
            rate: IdentityRate,
        }
    }

    pub fn animating(self) -> DynamicTimelineBuilder<'a, M, AbsoluteTimelineMetric, IdentityRate> {
        DynamicTimelineBuilder {
            steady_mobject: self,
            metric: AbsoluteTimelineMetric,
            rate: IdentityRate,
        }
    }
}

impl<'a, CO, ME, R> Alive<'a, DynamicTimeline<CO, ME, R>>
where
    CO: DynamicTimelineContent,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    pub fn collapse(self) -> Alive<'a, SteadyTimeline<CO::Output>> {
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

impl<'a, M, ME, R> ApplyAct<M> for DynamicTimelineBuilder<'a, M, ME, R>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    type Output<A> = Alive<'a, DynamicTimeline<ActionTimelineContent<M, A::Diff>, ME, R>>
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

impl<'a, M, ME, R> ApplyUpdate<M> for DynamicTimelineBuilder<'a, M, ME, R>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    type Output<U> = Alive<'a, DynamicTimeline<ContinuousTimelineContent<M, U>, ME, R>>
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

impl<'a, M, ME, R> ApplyConstruct<M> for DynamicTimelineBuilder<'a, M, ME, R>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    type Output<C> = Alive<'a, DynamicTimeline<DiscreteTimelineContent<M, C>, ME, R>>
    where
        C: Construct<M>;

    fn apply_construct<C>(self, construct: C) -> Self::Output<C>
    where
        C: Construct<M>,
    {
        self.steady_mobject
            .archive(|SteadyTimeline { mobject }, supervisor, _| {
                supervisor.launch_timeline(DynamicTimeline {
                    content: DiscreteTimelineContent {
                        mobject: mobject.clone(),
                        construct,
                        world: supervisor.world().clone(),
                    },
                    metric: self.metric,
                    rate: self.rate,
                })
            })
    }
}

impl<'a, M, ME, R, D> ApplyAct<M> for Alive<'a, DynamicTimeline<ActionTimelineContent<M, D>, ME, R>>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
    D: Diff<M>,
{
    type Output<A> = Alive<'a, DynamicTimeline<ActionTimelineContent<M, ComposeDiff<A::Diff, D>>, ME, R>> where A: Act<M>;

    fn apply_act<A>(self, act: A) -> Self::Output<A>
    where
        A: Act<M>,
    {
        let mobject = self.timeline.content.mobject.clone();
        let diff = ComposeDiff(act.act(&mobject), self.timeline.content.diff.clone());
        self.supervisor.launch_timeline(DynamicTimeline {
            content: ActionTimelineContent { mobject, diff },
            metric: self.timeline.metric,
            rate: self.timeline.rate,
        })
    }
}
