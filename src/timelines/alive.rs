use std::ops::Range;
use std::sync::Arc;

use super::super::mobjects::mobject::Mobject;
use super::super::toplevel::scene::Supervisor;
use super::act::Act;
use super::act::ApplyAct;
use super::construct::ApplyConstruct;
use super::construct::Construct;
use super::rates::ApplyRateChain;
use super::rates::IdentityRate;
use super::rates::Rate;
use super::timeline::action::ActionTimelineContent;
use super::timeline::continuous::ContinuousTimelineContent;
use super::timeline::discrete::DiscreteTimelineContent;
use super::timeline::dynamic::AbsoluteTimelineMetric;
use super::timeline::dynamic::Collapse;
use super::timeline::dynamic::DynamicTimeline;
use super::timeline::dynamic::DynamicTimelineContent;
use super::timeline::dynamic::DynamicTimelineMetric;
use super::timeline::dynamic::DynamicTimelinePresentation;
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
        F: FnOnce(&T::Presentation, &'a Supervisor, Range<f32>) -> O,
    {
        let presentation = self.timeline.presentation();
        let time_interval = self.spawn_time..self.supervisor.get_time();
        let output = f(
            &presentation,
            self.supervisor,
            *time_interval.start..*time_interval.end,
        );
        self.supervisor
            .archive_presentation(time_interval, presentation);
        output
    }
}

impl Supervisor {
    pub fn spawn<M>(&self, mobject: M) -> Alive<'_, SteadyTimeline<M>>
    where
        M: Mobject,
    {
        self.launch_timeline(SteadyTimeline { mobject })
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
}

impl<M> Alive<'_, SteadyTimeline<M>>
where
    M: Mobject,
{
    pub fn destroy(self) {
        self.archive(|_, _, _| ())
    }
}

impl<'a, M, A> ApplyAct<M, A> for Alive<'_, SteadyTimeline<M>>
where
    M: Mobject,
    A: Act<M>,
{
    type Output = Self;

    fn apply_act(self, act: A) -> Self::Output {
        self.archive(|SteadyTimeline { mobject }, supervisor, _| {
            let (mobject, _) = act.act(mobject);
            supervisor.spawn(mobject)
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

pub struct DynamicTimelineBuilderPartial<'a, M, ME>
where
    M: Mobject,
{
    steady_mobject: Alive<'a, SteadyTimeline<M>>,
    metric: ME,
}

impl<'a, M, ME, R> ApplyRateChain for DynamicTimelineBuilder<'a, M, ME, R>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    type InRate = R;
    type Partial = DynamicTimelineBuilderPartial<'a, M, ME>;
    type Output<RO> = DynamicTimelineBuilder<'a, M, ME, RO>
        where
            RO: Rate;

    fn split(self) -> (R, Self::Partial) {
        (
            self.rate,
            DynamicTimelineBuilderPartial {
                steady_mobject: self.steady_mobject,
                metric: self.metric,
            },
        )
    }

    fn combine<RO>(rate: RO, partial: Self::Partial) -> Self::Output<RO>
    where
        RO: Rate,
    {
        DynamicTimelineBuilder {
            steady_mobject: partial.steady_mobject,
            metric: partial.metric,
            rate,
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
    pub fn collapse(
        self,
    ) -> Alive<'a, SteadyTimeline<<CO::ContentPresentation as Collapse>::Output>> {
        self.archive(
            |DynamicTimelinePresentation {
                 content_presentation,
                 metric,
                 rate,
             },
             supervisor,
             time_interval| {
                supervisor.spawn(
                    content_presentation
                        .collapse(rate.eval(metric.eval(time_interval.end, time_interval))),
                )
            },
        )
    }
}

impl<'a, M, ME, R, A> ApplyAct<M, A> for DynamicTimelineBuilder<'a, M, ME, R>
where
    M: Mobject,
    ME: DynamicTimelineMetric,
    R: Rate,
    A: Act<M>,
{
    type Output = Alive<'a, DynamicTimeline<ActionTimelineContent<M>, ME, R>>;

    fn apply_act(self, act: A) -> Self::Output
    where
        A: Act<M>,
    {
        self.steady_mobject
            .archive(|SteadyTimeline { mobject }, supervisor, _| {
                let source_mobject = mobject.clone();
                let (target_mobject, diff) = act.act(&source_mobject);
                supervisor.launch_timeline(DynamicTimeline {
                    content: ActionTimelineContent {
                        source_mobject,
                        target_mobject,
                        diff,
                    },
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
                    },
                    metric: self.metric,
                    rate: self.rate,
                })
            })
    }
}

impl<M, A, ME, R> ApplyAct<M, A> for Alive<'_, DynamicTimeline<ActionTimelineContent<M>, ME, R>>
where
    M: Mobject,
    A: Act<M>,
    ME: DynamicTimelineMetric,
    R: Rate,
{
    type Output = Self;

    fn apply_act(mut self, act: A) -> Self::Output {
        let content = &mut self.timeline.content;
        let (target_mobject, diff) = act.act(&content.source_mobject);
        content.target_mobject = target_mobject;
        content.diff += diff;
        self
    }
}
