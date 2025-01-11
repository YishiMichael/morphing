use std::ops::Range;

use super::super::mobjects::mobject::Mobject;

pub trait Timeline: 'static {
    type Presentation: Present;

    fn presentation(self) -> Self::Presentation;
}

pub trait Present: 'static {
    fn present(&self, time: f32, time_interval: Range<f32>);
}

struct SupervisorData {
    timestamps: Vec<f32>,
    presentations: Vec<(Range<usize>, Box<dyn Present>)>,
}

impl SupervisorData {
    fn supervisor_present(&self, time: f32) {
        let timestamp_index = self
            .timestamps
            .partition_point(|&timestamp| timestamp < time);
        for (timestamp_index_range, present) in &self.presentations {
            if timestamp_index_range.contains(&timestamp_index) {
                present.present(
                    time,
                    self.timestamps[timestamp_index_range.start]
                        ..self.timestamps[timestamp_index_range.end],
                );
            }
        }
    }
}

pub struct Supervisor(parking_lot::Mutex<SupervisorData>);

impl Supervisor {
    fn new() -> Self {
        Self(parking_lot::Mutex::new(SupervisorData {
            timestamps: vec![0.0],
            presentations: Vec::new(),
        }))
    }

    fn into_data(self) -> SupervisorData {
        self.0.into_inner()
    }

    pub fn spawn<M>(&self, mobject: M) -> Alive<steady::SteadyTimeline<M>>
    where
        M: Mobject,
    {
        Alive::new(steady::SteadyTimeline { mobject }, self)
    }

    pub fn wait(&self, delta_time: f32) {
        assert!(!delta_time.is_sign_negative());
        let mut guard = self.0.lock();
        let time = guard.timestamps.last().unwrap() + delta_time;
        guard.timestamps.push(time);
    }
}

pub struct Alive<'a, T>
where
    T: Timeline,
{
    spawn_timestamp_index: usize,
    timeline: T,
    supervisor: &'a Supervisor,
}

impl<'a, T> Alive<'a, T>
where
    T: Timeline,
{
    fn new(timeline: T, supervisor: &'a Supervisor) -> Self {
        let guard = supervisor.0.lock();
        Self {
            spawn_timestamp_index: guard.timestamps.len() - 1,
            timeline,
            supervisor,
        }
    }

    fn archive<F, O>(self, f: F) -> O
    where
        F: FnOnce(&T::Presentation, &'a Supervisor, Range<f32>) -> O,
    {
        let mut guard = self.supervisor.0.lock();
        let timestamp_index_interval = self.spawn_timestamp_index..guard.timestamps.len() - 1;
        let presentation = self.timeline.presentation();
        let output = f(
            &presentation,
            self.supervisor,
            guard.timestamps[timestamp_index_interval.start]
                ..guard.timestamps[timestamp_index_interval.end],
        );
        if !timestamp_index_interval.is_empty() {
            guard
            .presentations
            .push((timestamp_index_interval, Box::new(presentation)));
        }
        output
    }

pub mod steady {
    use std::ops::Range;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::act::Act;
    use super::super::act::ApplyAct;
    use super::Alive;
    use super::Present;
    use super::Timeline;

    pub struct SteadyTimeline<M> {
        pub(crate) mobject: M,
    }

    impl<M> Timeline for SteadyTimeline<M>
    where
        M: Mobject,
    {
        type Presentation = Self;

        fn presentation(self) -> Self::Presentation {
            self
        }
    }

    impl<M> Present for SteadyTimeline<M>
    where
        M: Mobject,
    {
        fn present(&self, _time: f32, _time_interval: Range<f32>) {
            self.mobject.render();
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
                Alive::new(
                    SteadyTimeline {
                        mobject: mobject.apply_diff(act.act(mobject)),
                    },
                    supervisor,
                )
            })
        }
    }
}

pub mod dynamic {
    use std::ops::Range;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::act::Act;
    use super::super::act::ApplyAct;
    use super::super::construct::ApplyConstruct;
    use super::super::construct::Construct;
    use super::super::rates::ApplyRate;
    use super::super::rates::IdentityRate;
    use super::super::rates::Rate;
    use super::super::update::ApplyUpdate;
    use super::super::update::Update;
    use super::action::ActionTimelineContent;
    use super::continuous::ContinuousTimelineContent;
    use super::discrete::DiscreteTimelineContent;
    use super::steady::SteadyTimeline;
    use super::Alive;
    use super::Present;
    use super::Timeline;

    pub trait ContentPresent: 'static {
        fn content_present(&self, time: f32);
    }

    pub trait Collapse {
        type Output: Mobject;

        fn collapse(&self, time: f32) -> Self::Output;
    }

    pub trait DynamicTimelineContent: 'static {
        type ContentPresentation: ContentPresent + Collapse;

        fn content_presentation(self) -> Self::ContentPresentation;
    }

    pub trait DynamicTimelineMetric: 'static {
        fn eval(&self, time: f32, time_interval: Range<f32>) -> f32;
    }

    pub struct RelativeTimelineMetric;

    impl DynamicTimelineMetric for RelativeTimelineMetric {
        fn eval(&self, time: f32, time_interval: Range<f32>) -> f32 {
            (time - time_interval.start) / (time_interval.end - time_interval.start)
        }
    }

    pub struct AbsoluteTimelineMetric;

    impl DynamicTimelineMetric for AbsoluteTimelineMetric {
        fn eval(&self, time: f32, time_interval: Range<f32>) -> f32 {
            time - time_interval.start
        }
    }

    pub struct DynamicTimeline<CO, ME, R> {
        pub(crate) content: CO,
        metric: ME,
        rate: R,
    }

    impl<CO, ME, R> Timeline for DynamicTimeline<CO, ME, R>
    where
        CO: DynamicTimelineContent,
        ME: DynamicTimelineMetric,
        R: Rate,
    {
        type Presentation = DynamicTimelinePresentation<CO::ContentPresentation, ME, R>;

        fn presentation(self) -> Self::Presentation {
            DynamicTimelinePresentation {
                content_presentation: self.content.content_presentation(),
                metric: self.metric,
                rate: self.rate,
            }
        }
    }

    pub struct DynamicTimelinePresentation<CP, ME, R> {
        content_presentation: CP,
        metric: ME,
        rate: R,
    }

    impl<CP, ME, R> Present for DynamicTimelinePresentation<CP, ME, R>
    where
        CP: ContentPresent,
        ME: DynamicTimelineMetric,
        R: Rate,
    {
        fn present(&self, time: f32, time_interval: Range<f32>) {
            self.content_presentation
                .content_present(self.rate.eval(self.metric.eval(time, time_interval)));
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

    impl<'a, M, ME, R> ApplyRate<R> for DynamicTimelineBuilder<'a, M, ME, R>
    where
        M: Mobject,
        ME: DynamicTimelineMetric,
        R: Rate,
    {
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
        pub fn animate(
            self,
        ) -> DynamicTimelineBuilder<'a, M, RelativeTimelineMetric, IdentityRate> {
            DynamicTimelineBuilder {
                steady_mobject: self,
                metric: RelativeTimelineMetric,
                rate: IdentityRate,
            }
        }

        pub fn animating(
            self,
        ) -> DynamicTimelineBuilder<'a, M, AbsoluteTimelineMetric, IdentityRate> {
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
                    Alive::new(
                        SteadyTimeline {
                            mobject: content_presentation
                                .collapse(rate.eval(metric.eval(time_interval.end, time_interval))),
                        },
                        supervisor,
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
                    let diff = act.act(&source_mobject);
                    let target_mobject = source_mobject.apply_diff(diff.clone());
                    Alive::new(
                        DynamicTimeline {
                            content: ActionTimelineContent {
                                source_mobject,
                                target_mobject,
                                diff,
                            },
                            metric: self.metric,
                            rate: self.rate,
                        },
                        supervisor,
                    )
                })
        }
    }

    impl<'a, M, ME, R, U> ApplyUpdate<M, U> for DynamicTimelineBuilder<'a, M, ME, R>
    where
        M: Mobject,
        ME: DynamicTimelineMetric,
        R: Rate,
        U: Update<M>,
    {
        type Output = Alive<'a, DynamicTimeline<ContinuousTimelineContent<M, U>, ME, R>>;

        fn apply_update(self, update: U) -> Self::Output {
            self.steady_mobject
                .archive(|SteadyTimeline { mobject }, supervisor, _| {
                    let mobject = mobject.clone();
                    Alive::new(
                        DynamicTimeline {
                            content: ContinuousTimelineContent { mobject, update },
                            metric: self.metric,
                            rate: self.rate,
                        },
                        supervisor,
                    )
                })
        }
    }

    impl<'a, M, ME, R, C> ApplyConstruct<M, C> for DynamicTimelineBuilder<'a, M, ME, R>
    where
        M: Mobject,
        ME: DynamicTimelineMetric,
        R: Rate,
        C: Construct<M>,
    {
        type Output = Alive<'a, DynamicTimeline<DiscreteTimelineContent<M, C>, ME, R>>;

        fn apply_construct(self, construct: C) -> Self::Output {
            self.steady_mobject
                .archive(|SteadyTimeline { mobject }, supervisor, _| {
                    let mobject = mobject.clone();
                    Alive::new(
                        DynamicTimeline {
                            content: DiscreteTimelineContent { mobject, construct },
                            metric: self.metric,
                            rate: self.rate,
                        },
                        supervisor,
                    )
                })
        }
    }
}

pub mod action {
    use super::super::super::mobjects::mobject::Mobject;
    use super::super::act::Act;
    use super::super::act::ApplyAct;
    use super::super::rates::Rate;
    use super::dynamic::Collapse;
    use super::dynamic::ContentPresent;
    use super::dynamic::DynamicTimeline;
    use super::dynamic::DynamicTimelineContent;
    use super::dynamic::DynamicTimelineMetric;
    use super::Alive;

    pub struct ActionTimelineContent<M>
    where
        M: Mobject,
    {
        pub(crate) source_mobject: M,
        pub(crate) target_mobject: M,
        pub(crate) diff: M::Diff,
    }

    impl<M> DynamicTimelineContent for ActionTimelineContent<M>
    where
        M: Mobject,
    {
        type ContentPresentation = Self;

        fn content_presentation(self) -> Self::ContentPresentation {
            self
        }
    }

    impl<M> ContentPresent for ActionTimelineContent<M>
    where
        M: Mobject,
    {
        fn content_present(&self, time: f32) {
            self.collapse(time).render();
        }
    }

    impl<M> Collapse for ActionTimelineContent<M>
    where
        M: Mobject,
    {
        type Output = M;

        fn collapse(&self, time: f32) -> Self::Output {
            let mut diff = self.diff.clone();
            diff *= time;
            self.source_mobject.apply_diff(diff)
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
            let diff = act.act(&content.target_mobject);
            content.target_mobject = content.target_mobject.apply_diff(diff.clone());
            content.diff += diff;
            self
        }
    }
}

pub mod continuous {
    use super::super::super::mobjects::mobject::Mobject;
    use super::super::update::Update;
    use super::dynamic::Collapse;
    use super::dynamic::ContentPresent;
    use super::dynamic::DynamicTimelineContent;

    pub struct ContinuousTimelineContent<M, U> {
        pub(crate) mobject: M,
        pub(crate) update: U,
    }

    impl<M, U> DynamicTimelineContent for ContinuousTimelineContent<M, U>
    where
        M: Mobject,
        U: Update<M>,
    {
        type ContentPresentation = Self;

        fn content_presentation(self) -> Self::ContentPresentation {
            self
        }
    }

    impl<M, U> ContentPresent for ContinuousTimelineContent<M, U>
    where
        M: Mobject,
        U: Update<M>,
    {
        fn content_present(&self, time: f32) {
            self.collapse(time).render();
        }
    }

    impl<M, U> Collapse for ContinuousTimelineContent<M, U>
    where
        M: Mobject,
        U: Update<M>,
    {
        type Output = M;

        fn collapse(&self, time: f32) -> Self::Output {
            self.update.update(&self.mobject, time)
        }
    }
}

pub mod discrete {
    use super::super::super::mobjects::mobject::Mobject;
    use super::super::construct::Construct;
    use super::dynamic::Collapse;
    use super::dynamic::ContentPresent;
    use super::dynamic::DynamicTimelineContent;
    use super::steady::SteadyTimeline;
    use super::Alive;
    use super::Supervisor;
    use super::SupervisorData;

    pub struct DiscreteTimelineContent<M, C> {
        pub(crate) mobject: M,
        pub(crate) construct: C,
    }

    impl<M, C> DynamicTimelineContent for DiscreteTimelineContent<M, C>
    where
        M: Mobject,
        C: Construct<M>,
    {
        type ContentPresentation = DiscreteTimelineContentPresentation<C::Output>;

        fn content_presentation(self) -> Self::ContentPresentation {
            let supervisor = Supervisor::new();
            let mobject = self
                .construct
                .construct(
                    Alive::new(
                        SteadyTimeline {
                            mobject: self.mobject,
                        },
                        &supervisor,
                    ),
                    &supervisor,
                )
                .archive(|steady_timeline, _, _| steady_timeline.mobject.clone());
            DiscreteTimelineContentPresentation {
                mobject,
                supervisor_data: supervisor.into_data(),
            }
        }
    }

    pub struct DiscreteTimelineContentPresentation<M> {
        mobject: M,
        supervisor_data: SupervisorData,
    }

    impl<M> ContentPresent for DiscreteTimelineContentPresentation<M>
    where
        M: Mobject,
    {
        fn content_present(&self, time: f32) {
            self.supervisor_data.supervisor_present(time);
        }
    }

    impl<M> Collapse for DiscreteTimelineContentPresentation<M>
    where
        M: Mobject,
    {
        type Output = M;

        fn collapse(&self, _time: f32) -> Self::Output {
            self.mobject.clone()
        }
    }
}
